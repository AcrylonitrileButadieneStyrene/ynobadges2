use std::{collections::HashMap, sync::Arc};

use crate::{
    Badge,
    format::{config::Config, input, output},
};

pub async fn lang(config: Arc<Config>, badges: Arc<[Badge]>) {
    let mut languages: HashMap<String, output::Lang> = config
        .lang
        .list
        .iter()
        .map(|key| {
            let path = format!("ynobadges/lang/{key}.json");
            let contents = std::fs::read(&path).unwrap();
            let lang: output::Lang = serde_json::from_slice(&contents).unwrap();
            (key.clone(), lang)
        })
        .collect();

    for Badge {
        id: badge_id,
        game_id,
        bundle: input::Bundle { lang, .. },
        ..
    } in &*badges
    {
        let base = lang.get(&config.lang.base).unwrap();
        for (language_id, language) in &mut languages {
            let this = lang.get(language_id);

            let (name, name_fallback) = extract(base, this, |x| x.name.clone());
            let (description, desc_fallback) = extract(base, this, |x| x.description.clone());
            let (condition, condition_fallback) = extract(base, this, |x| x.condition.clone());
            let (checkbox, checkbox_fallback) = extract(base, this, |x| x.checkbox.clone());

            let path = format!("lang/{language_id}/{game_id}/{badge_id}");
            check_tab(&path, "name", name.as_ref());
            check_tab(&path, "description", description.as_ref());
            check_tab(&path, "condition", condition.as_ref());
            // the checkbox field can't really be messed up, checking not necessary

            let game_entries = language
                .entry(game_id.clone())
                .or_insert_with(indexmap::IndexMap::new);
            let entry = game_entries
                .entry(badge_id.clone())
                .or_insert(input::Locale {
                    name: name.clone(),
                    description: description.clone(),
                    condition: condition.clone(),
                    checkbox: checkbox.clone(),
                });

            let mut inequal = false;
            inequal |= check_conflict(&entry.name, &name, name_fallback);
            inequal |= check_conflict(&entry.description, &description, desc_fallback);
            inequal |= check_conflict(&entry.condition, &condition, condition_fallback);
            inequal |= check_conflict(&entry.checkbox, &checkbox, checkbox_fallback);
            if inequal {
                log::warn!("Mismatch between locale {language_id}/{game_id}/{badge_id}");
            }
        }
    }

    for (key, locale) in languages {
        let path = format!("ynobadges/lang/{key}.json");
        tokio::fs::write(&path, serde_json::to_string_pretty(&locale).unwrap())
            .await
            .unwrap();
    }
}

fn extract<T, U>(
    base: &input::Locale,
    this: Option<&input::Locale>,
    extractor: U,
) -> (Option<T>, bool)
where
    U: Fn(&input::Locale) -> Option<T>,
{
    this.and_then(&extractor)
        .map_or_else(|| (extractor(base), true), |value| (Some(value), false))
}

fn check_tab(path: &str, field: &str, option: Option<&String>) {
    let has_tab = option.as_ref().is_some_and(|item| item.contains('\t'));
    if has_tab {
        log::warn!("{path}/{field} contains a tab.");
    }
}

fn check_conflict<T: PartialEq>(old: &T, new: &T, silenced: bool) -> bool {
    !silenced && old != new
}

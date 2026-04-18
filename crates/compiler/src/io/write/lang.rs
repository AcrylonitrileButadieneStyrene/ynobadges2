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
        badge_id,
        game_id,
        bundle: input::Bundle { lang, .. },
        ..
    } in &*badges
    {
        let base = lang.get(&config.lang.base).unwrap();
        for (language_id, language) in &mut languages {
            let (lang, is_fallback) = lang
                .get(language_id)
                .map_or((base, true), |entry| (entry, false));
            let game_entries = language
                .entry(game_id.clone())
                .or_insert_with(indexmap::IndexMap::new);

            let entry = game_entries.entry(badge_id.clone()).or_insert(lang.clone());
            if !is_fallback && entry != lang {
                log::warn!("Mismatch between locale {language_id}/{game_id}/{badge_id}");
            }

            let path = format!("lang/{language_id}/{game_id}/{badge_id}");
            check_tab(&path, "name", &lang.name);
            check_tab(&path, "description", &lang.description);
            check_tab(&path, "condition", &lang.condition);
            // the checkbox field can't really be messed up, checking not necessary

            lang.name.as_ref().map(|item| item.contains("\t"));
        }
    }

    for (key, locale) in languages {
        let path = format!("ynobadges/lang/{key}.json");
        tokio::fs::write(&path, serde_json::to_string_pretty(&locale).unwrap())
            .await
            .unwrap();
    }
}

fn check_tab(path: &str, field: &str, option: &Option<String>) {
    let has_tab = option
        .as_ref()
        .map(|item| item.contains('\t'))
        .unwrap_or_default();
    if has_tab {
        log::warn!("{path}/{field} contains a tab.");
    }
}

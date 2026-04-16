use std::{collections::HashMap, num::NonZeroU16, sync::Arc};

use crate::{
    Badge,
    dsl::{
        conditions,
        requirements::{self, Request},
    },
    format::{
        config::Config,
        input,
        output::{self, BadgeReqType},
    },
};

pub async fn badges(config: Arc<Config>, badges: Arc<[Badge]>) {
    for Badge {
        badge_id: id,
        game_id: game,
        batch,
        bundle,
    } in &*badges
    {
        let (map_id, map_x, map_y, map_secret) = match bundle.badge.map {
            input::Map::Plain(id) => (id, None, None, false),
            input::Map::Object { id, x, y, secret } => (id, x, y, secret),
        };

        let Some(reqs) = bundle
            .conditions
            .requirements
            .as_ref()
            .map_or(Some(Request::All), |requirements| {
                requirements::parse(requirements)
            })
        else {
            continue;
        };

        let (req_string, req_strings, req_string_arrays, req_type) = match reqs {
            Request::All => {
                let conditions = bundle.conditions.rest.keys().cloned().collect::<Vec<_>>();
                match conditions.len() {
                    0 => (Some(id.clone()), None, None, BadgeReqType::Tag),
                    1 => (
                        Some(match &**conditions.first().unwrap() {
                            "default" => id.clone(),
                            x => x.to_string(),
                        }),
                        None,
                        None,
                        BadgeReqType::Tag,
                    ),
                    _ => (None, Some(conditions), None, BadgeReqType::Tags),
                }
            }
            Request::Tag(id) => (Some(id), None, None, BadgeReqType::Tag),
            Request::Tags(ids) => (None, Some(ids), None, BadgeReqType::Tags),
            Request::TagArray(ids) => (None, None, Some(ids), BadgeReqType::TagArrays),
        };

        let out = output::Badge {
            animated: bundle.badge.animated,
            art: bundle.badge.art.clone(),
            batch: *batch,
            bp: NonZeroU16::new(bundle.badge.points).map(Into::into), // todo: temporary
            group: bundle.badge.group.clone().or_else(|| {
                config
                    .groups
                    .get(game)
                    .and_then(|group| group.default.clone())
            }),
            hidden: bundle.badge.hidden,
            map: map_id,
            map_order: None,
            map_x,
            map_y,
            order: None,
            overlay_type: None,
            parent: None,
            req_count: None,
            req_int: None,
            req_string,
            req_string_arrays,
            req_strings,
            req_type: Some(req_type),
            secret: bundle.badge.secret,
            secret_condition: bundle.conditions.secret,
            secret_map: map_secret,
        };

        tokio::fs::write(
            format!("ynobadges/badges/{game}/{id}.json"),
            serde_json::to_string_pretty(&out).unwrap(),
        )
        .await
        .unwrap();
    }
}

pub async fn conditions(badges: Arc<[Badge]>) {
    for Badge {
        badge_id,
        game_id,
        bundle: input::Bundle { conditions, .. },
        ..
    } in &*badges
    {
        let conditions = conditions
            .rest
            .iter()
            .filter_map(|(condition_id, condition)| {
                let condition_id = match &**condition_id {
                    "default" => badge_id.clone(),
                    x => x.to_string(),
                };

                conditions::parse(badge_id, condition).map(|condition| (condition_id, condition))
            });

        for (condition_id, condition) in conditions {
            tokio::fs::write(
                format!("ynobadges/conditions/{game_id}/{condition_id}.json"),
                serde_json::to_string_pretty(&condition).unwrap(),
            )
            .await
            .unwrap();
        }
    }
}

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
        }
    }

    for (key, locale) in languages {
        let path = format!("ynobadges/lang/{key}.json");
        tokio::fs::write(&path, serde_json::to_string_pretty(&locale).unwrap())
            .await
            .unwrap();
    }
}

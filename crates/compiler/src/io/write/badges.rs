use std::{num::NonZeroU16, sync::Arc};

use crate::{
    Badge,
    dsl::requirements::{self, Request},
    format::{
        config::Config,
        input,
        output::{self, BadgeReqType},
    },
};

pub async fn badges(config: Arc<Config>, badges: Arc<[Badge]>) {
    for Badge {
        id: badge_id,
        game_id,
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
                let mut conditions = bundle.conditions.rest.keys().cloned().collect::<Vec<_>>();
                match conditions.len() {
                    0 => (Some(badge_id.clone()), None, None, BadgeReqType::Tag),
                    1 => (
                        Some(match &**conditions.first().unwrap() {
                            "default" => badge_id.clone(),
                            x => x.to_string(),
                        }),
                        None,
                        None,
                        BadgeReqType::Tag,
                    ),
                    _ => (
                        None,
                        Some({
                            conditions.sort();
                            conditions
                        }),
                        None,
                        BadgeReqType::Tags,
                    ),
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
                    .get(game_id)
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

        let path = format!("ynobadges/badges/{game_id}/{badge_id}.json");

        if tokio::fs::try_exists(&path).await.unwrap_or_default() {
            let bytes = tokio::fs::read(&path).await.unwrap();
            let original: output::Badge = serde_json::from_slice(&bytes).unwrap();
            if original != out {
                // todo: print a diff
                log::warn!("Desync detected: {batch}/{game_id}/{badge_id} != {path}");
            }
        }

        tokio::fs::write(path, serde_json::to_string_pretty(&out).unwrap())
            .await
            .unwrap();
    }
}

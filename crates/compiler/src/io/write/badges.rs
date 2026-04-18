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
                let mut conditions = bundle.conditions.rest.keys().cloned().collect::<Vec<_>>();
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

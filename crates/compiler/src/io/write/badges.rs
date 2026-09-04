use std::num::NonZeroU16;

use crate::{
    Badge,
    dsl::requirements::{self, Request},
    format::{
        config::Config,
        input,
        output::{self, BadgeReqType},
    },
};

pub async fn badges(config: &Config, badges: &[Badge]) {
    for Badge {
        id: badge_id,
        game_id,
        batch,
        bundle,
    } in badges
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

        let mut req_int = None;
        let (req_string, req_strings, req_string_arrays, mut req_type, req_count) = match reqs {
            Request::All => {
                let mut conditions = bundle.conditions.rest.keys().cloned().collect::<Vec<_>>();
                match conditions.len() {
                    0 => (Some(badge_id.clone()), None, None, BadgeReqType::Tag, None),
                    1 => (
                        Some(match &**conditions.first().unwrap() {
                            "default" => badge_id.clone(),
                            x => x.to_string(),
                        }),
                        None,
                        None,
                        BadgeReqType::Tag,
                        None,
                    ),
                    _ => (
                        None,
                        Some({
                            conditions.sort();
                            conditions
                        }),
                        None,
                        BadgeReqType::Tags,
                        None,
                    ),
                }
            }
            Request::Tag(id) => (Some(id), None, None, BadgeReqType::Tag, None),
            Request::Tags(ids) => (None, Some(ids), None, BadgeReqType::Tags, None),
            Request::TagsCount(ids, count) => {
                (None, Some(ids), None, BadgeReqType::Tags, Some(count))
            }
            Request::TagArray(ids) => (None, None, Some(ids), BadgeReqType::TagArrays, None),
        };

        if let Some(attributes) = &bundle.badge.attributes
            && let Some(time_limit) = attributes.time_limit
        {
            req_type = BadgeReqType::TimeTrial;
            req_int = Some(time_limit);
        }

        let group = config.groups.get(game_id).and_then(|game| {
            let filtered = bundle
                .badge
                .group
                .clone()
                .filter(|group| game.list.contains(group));
            if let Some(group) = &bundle.badge.group
                && filtered.is_none()
            {
                log::warn!("Invalid group {group} for badge {batch}/{game_id}/{badge_id}.toml");
            }

            filtered.or_else(|| game.default.clone())
        });

        let out = output::Badge {
            animated: bundle.badge.animated,
            art: bundle.badge.art.clone(),
            batch: *batch,
            bp: NonZeroU16::new(bundle.badge.points).map(Into::into), // todo: temporary
            group,
            hidden: bundle.badge.hidden,
            map: Some(map_id),
            map_order: None,
            map_x,
            map_y,
            order: bundle.badge.order,
            overlay_type: None,
            parent: None,
            req_count,
            req_int,
            req_string,
            req_string_arrays,
            req_strings,
            req_type: Some(req_type),
            secret: bundle.badge.secret,
            secret_condition: bundle.conditions.secret,
            secret_map: map_secret,
            dev: false,
        };

        let game = format!("ynobadges/badges/{game_id}");
        let path = format!("{game}/{badge_id}.json");

        if tokio::fs::try_exists(&path).await.unwrap_or_default() {
            let bytes = tokio::fs::read(&path).await.unwrap();
            match serde_json::from_slice::<output::Badge>(&bytes) {
                Ok(original) => {
                    if original != out {
                        // todo: print a diff
                        log::warn!("Desync detected: {batch}/{game_id}/{badge_id}.toml != {path}");
                    }
                }
                Err(err) => {
                    log::warn!("Error parsing original badge @ badges/{game_id}/{badge_id}: {err}");
                }
            }
        } else if !tokio::fs::try_exists(&game).await.unwrap_or_default() {
            tokio::fs::create_dir(&game).await.unwrap();
        }

        tokio::fs::write(path, serde_json::to_string_pretty(&out).unwrap())
            .await
            .unwrap();
    }
}

use std::sync::Arc;

use crate::{Badge, dsl::conditions, format::input};

pub async fn conditions(badges: Arc<[Badge]>) {
    for Badge {
        id: badge_id,
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

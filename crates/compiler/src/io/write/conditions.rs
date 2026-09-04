use crate::{
    Badge,
    dsl::conditions,
    format::{input, output},
};

pub async fn conditions(badges: &[Badge]) {
    for Badge {
        batch,
        id: badge_id,
        game_id,
        bundle: input::Bundle { conditions, .. },
        ..
    } in badges
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
            let game = format!("ynobadges/conditions/{game_id}");
            let path = format!("{game}/{condition_id}.json");

            match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    let original: output::Condition = serde_json::from_slice(&bytes).unwrap();
                    if original != condition {
                        log::warn!("Desync detected: {batch}/{game_id}/{badge_id}.toml != {path}");
                    }
                }
                Err(err) if matches!(err.kind(), std::io::ErrorKind::NotFound) => {
                    if !tokio::fs::try_exists(&game).await.unwrap_or_default() {
                        tokio::fs::create_dir(&game).await.unwrap();
                    }
                }
                Err(err) => panic!("{err}"),
            }

            tokio::fs::write(&path, serde_json::to_string_pretty(&condition).unwrap())
                .await
                .unwrap();
        }
    }
}

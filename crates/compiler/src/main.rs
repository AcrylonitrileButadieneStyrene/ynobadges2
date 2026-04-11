pub mod format;

#[derive(Debug)]
struct Badge {
    id: String,
    game: String,
    batch: u16,
    bundle: format::input::Bundle,
}

fn main() {
    tracing_subscriber::fmt().without_time().init();

    let config: format::config::Config =
        toml::from_slice(&std::fs::read("./options.toml").unwrap()).unwrap();

    let Some(badges) = collect() else {
        return;
    };

    dbg!(badges);
}

fn collect() -> Option<Vec<Badge>> {
    Some(
        std::fs::read_dir("badges")
            .inspect_err(|err| log::error!("Failed to read `badges`: {err}"))
            .ok()?
            .filter_map(|entry| {
                entry
                    .inspect_err(|err| log::warn!("Failed to read batch: {err}"))
                    .ok()
            })
            .filter_map(|batch_entry| {
                let batch = batch_entry
                    .file_name()
                    .into_string()
                    .inspect_err(|err| log::warn!("Batch contains invalid unicode: {err:?}"))
                    .ok()?
                    .parse::<u16>()
                    .inspect_err(|err| log::warn!("Batch is not a number: {err}"))
                    .ok()?;
                let game_entries = std::fs::read_dir(batch_entry.path())
                    .inspect_err(|err| log::warn!("Failed to read `badges/:batch`: {err}"))
                    .ok()?;
                Some((batch, game_entries))
            })
            .flat_map(|(batch, game_entries)| {
                game_entries
                    .filter_map(|game_entry| {
                        game_entry
                            .inspect_err(|err| log::warn!("Failed to read game: {err}"))
                            .ok()
                    })
                    .filter_map(|game_entry| {
                        let game = game_entry
                            .file_name()
                            .into_string()
                            .inspect_err(|err| log::warn!("Game contains invalid unicode: {err:?}"))
                            .ok()?;
                        let badge_entries = std::fs::read_dir(game_entry.path())
                            .inspect_err(|err| {
                                log::warn!("Failed to read `badges/:batch/:game`: {err}")
                            })
                            .ok()?;
                        Some((game, badge_entries))
                    })
                    .flat_map(move |(game, badge_entries)| {
                        badge_entries
                            .filter_map(|badge_entry| {
                                badge_entry
                                    .inspect_err(|err| log::warn!("Failed to read badge: {err}"))
                                    .ok()
                            })
                            .filter_map(move |badge_entry| {
                                let id = badge_entry
                                    .path()
                                    .file_prefix()
                                    .unwrap()
                                    .to_os_string()
                                    .into_string()
                                    .inspect_err(|err| {
                                        log::warn!("Badge contained invalid unicode: {err:?}")
                                    })
                                    .ok()?;
                                let contents = std::fs::read(badge_entry.path())
                                    .inspect_err(|err| log::warn!("Failed to read badge: {err}"))
                                    .ok()?;
                                let bundle = toml::from_slice(&contents)
                                    .inspect_err(|err| log::warn!("Failed to parse badge: {err}"))
                                    .ok()?;

                                Some(Badge {
                                    id,
                                    batch,
                                    game: game.clone(),
                                    bundle,
                                })
                            })
                    })
            })
            .collect(),
    )
}

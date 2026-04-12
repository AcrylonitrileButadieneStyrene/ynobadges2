use std::num::NonZeroU16;

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

    unsafe {
        git2::opts::set_verify_owner_validation(false).unwrap();
    }

    let repo = git2::Repository::open("ynobadges")
        .or_else(|_| {
            log::info!("Cloning ynobadges...");
            git2::Repository::clone("https://github.com/ynoproject/ynobadges", "ynobadges")
        })
        .unwrap();
    git_reset(&repo);

    // step 1: write out badge files
    for Badge {
        id,
        game,
        batch,
        bundle,
    } in badges
    {
        let (map_id, map_x, map_y, map_secret) = match bundle.badge.map {
            format::input::Map::Plain(id) => (id, None, None, false),
            format::input::Map::Object { id, x, y, secret } => (id, Some(x), Some(y), secret),
        };

        let out = format::output::Badge {
            animated: bundle.badge.animated,
            art: bundle.badge.art,
            batch,
            bp: NonZeroU16::new(bundle.badge.points).map(|x| x.into()),
            group: bundle.badge.group.or_else(|| {
                config
                    .groups
                    .get(&game)
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
            req_string: None,
            req_string_arrays: None,
            req_strings: None,
            req_type: None,
            secret: bundle.badge.secret,
            secret_condition: bundle.conditions.secret,
            secret_map: map_secret,
        };

        std::fs::write(
            format!("ynobadges/badges/{game}/{id}.json"),
            serde_json::to_string_pretty(&out).unwrap(),
        )
        .unwrap();
    }
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
                                    .inspect_err(|err| {
                                        log::warn!("Failed to parse badge `{id}`:\n{err}")
                                    })
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

fn git_reset(repo: &git2::Repository) {
    // git fetch
    repo.find_remote("origin")
        .unwrap()
        .fetch(&["master"], None, None)
        .unwrap();

    // git reset --hard
    repo.reset(
        repo.find_reference("refs/remotes/origin/master")
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .as_object(),
        git2::ResetType::Hard,
        None,
    )
    .unwrap();

    // git clean -fd
    for entry in repo
        .statuses(Some(git2::StatusOptions::new().include_untracked(true)))
        .unwrap()
        .iter()
    {
        if entry.status().contains(git2::Status::WT_NEW) {
            let path = std::path::PathBuf::from("ynobadges").join(entry.path().unwrap());
            if path.is_dir() {
                std::fs::remove_dir_all(path).unwrap();
            } else {
                std::fs::remove_file(path).unwrap();
            }
        }
    }
}

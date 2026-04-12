#![feature(iterator_try_collect)]
#![feature(bool_to_result)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::{collections::HashMap, num::NonZeroU16};

use format::output::BadgeReqType;

pub mod dsl;
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

    write_badges(&config, &badges);
    write_conditions(&badges);
    write_lang(&config, &badges);
}

fn write_badges(config: &format::config::Config, badges: &[Badge]) {
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

        let Some(reqs) = bundle
            .conditions
            .requirements
            .as_ref()
            .map_or(Some(dsl::requirements::Request::All), |requirements| {
                dsl::requirements::parse(requirements)
            })
        else {
            continue;
        };

        let (req_string, req_strings, req_string_arrays, req_type) = match reqs {
            dsl::requirements::Request::All => {
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
            dsl::requirements::Request::Tag(id) => (Some(id), None, None, BadgeReqType::Tag),
            dsl::requirements::Request::Tags(ids) => (None, Some(ids), None, BadgeReqType::Tags),
            dsl::requirements::Request::TagArray(ids) => {
                (None, None, Some(ids), BadgeReqType::TagArrays)
            }
        };

        let out = format::output::Badge {
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

        std::fs::write(
            format!("ynobadges/badges/{game}/{id}.json"),
            serde_json::to_string_pretty(&out).unwrap(),
        )
        .unwrap();
    }
}

fn write_conditions(badges: &[Badge]) {
    for Badge {
        id: badge_id,
        game,
        bundle: format::input::Bundle { conditions, .. },
        ..
    } in badges
    {
        conditions
            .rest
            .iter()
            .filter_map(|(condition_id, condition)| {
                let condition_id = match &**condition_id {
                    "default" => badge_id.clone(),
                    x => x.to_string(),
                };

                dsl::conditions::parse(condition).map(|condition| (condition_id, condition))
            })
            .for_each(|(condition_id, condition)| {
                std::fs::write(
                    format!("ynobadges/conditions/{game}/{condition_id}.json"),
                    serde_json::to_string_pretty(&condition).unwrap(),
                )
                .unwrap();
            });
    }
}

fn write_lang(config: &format::config::Config, badges: &[Badge]) {
    let mut locales: HashMap<String, format::output::Lang> = config
        .lang
        .list
        .iter()
        .map(|key| {
            let path = format!("ynobadges/lang/{key}.json");
            let contents = std::fs::read(&path).unwrap();
            let lang: format::output::Lang = serde_json::from_slice(&contents).unwrap();
            (key.clone(), lang)
        })
        .collect();

    for Badge {
        id,
        game,
        bundle: format::input::Bundle { lang, .. },
        ..
    } in badges
    {
        let base = lang.get(&config.lang.base).unwrap();
        for (key, locale) in &mut locales {
            let lang = lang.get(key).unwrap_or(base);
            locale
                .entry(game.clone())
                .or_insert_with(indexmap::IndexMap::new)
                .insert(id.clone(), lang.clone());
        }
    }

    for (key, locale) in locales {
        let path = format!("ynobadges/lang/{key}.json");
        std::fs::write(&path, serde_json::to_string_pretty(&locale).unwrap()).unwrap();
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
                    .inspect_err(|err| {
                        log::warn!("Batch contains invalid unicode: {}", err.display());
                    })
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
                            .inspect_err(|err| {
                                log::warn!("Game contains invalid unicode: {}", err.display());
                            })
                            .ok()?;
                        let badge_entries = std::fs::read_dir(game_entry.path())
                            .inspect_err(|err| {
                                log::warn!("Failed to read `badges/:batch/:game`: {err}");
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
                                        log::warn!(
                                            "Badge contained invalid unicode: {}",
                                            err.display()
                                        );
                                    })
                                    .ok()?;
                                let contents = std::fs::read(badge_entry.path())
                                    .inspect_err(|err| log::warn!("Failed to read badge: {err}"))
                                    .ok()?;
                                let bundle = toml::from_slice(&contents)
                                    .inspect_err(|err| {
                                        log::warn!("Failed to parse badge `{id}`:\n{err}");
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

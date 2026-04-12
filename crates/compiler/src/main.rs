#![feature(iterator_try_collect)]
#![feature(bool_to_result)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::sync::Arc;

mod collect;
pub mod dsl;
pub mod format;
mod write;

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

    let Some(badges) = collect::badges() else {
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

    tokio::runtime::Builder::new_multi_thread()
        .build()
        .unwrap()
        .block_on(async_main(Arc::new(config), Arc::from(badges)));
}

async fn async_main(config: Arc<format::config::Config>, badges: Arc<[Badge]>) {
    let mut set = tokio::task::JoinSet::new();
    set.spawn(write::badges(config.clone(), badges.clone()));
    set.spawn(write::conditions(badges.clone()));
    set.spawn(write::lang(config, badges));
    set.join_all().await;
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

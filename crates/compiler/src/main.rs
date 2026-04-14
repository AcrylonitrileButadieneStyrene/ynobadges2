#![feature(iterator_try_collect)]
#![feature(bool_to_result)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::sync::Arc;

pub mod dsl;
pub mod format;
mod io;

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

    let Some(badges) = io::collect::badges() else {
        return;
    };

    let repo = io::git::open_or_init().unwrap();
    io::git::reset(&repo);

    tokio::runtime::Builder::new_multi_thread()
        .build()
        .unwrap()
        .block_on(async_main(Arc::new(config), Arc::from(badges)));

    io::git::fix_staging(&repo);
}

async fn async_main(config: Arc<format::config::Config>, badges: Arc<[Badge]>) {
    let mut set = tokio::task::JoinSet::new();
    set.spawn(io::write::badges(config.clone(), badges.clone()));
    set.spawn(io::write::conditions(badges.clone()));
    set.spawn(io::write::lang(config, badges));
    set.join_all().await;
}

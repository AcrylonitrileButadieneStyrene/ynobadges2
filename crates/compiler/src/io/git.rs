pub fn open_or_init() -> Result<git2::Repository, (git2::Error, git2::Error)> {
    unsafe {
        git2::opts::set_verify_owner_validation(false).unwrap();
    }

    match git2::Repository::open("ynobadges") {
        Ok(repo) => Ok(repo),
        Err(left) => {
            log::info!("Cloning ynobadges...");
            git2::Repository::clone("https://github.com/ynoproject/ynobadges", "ynobadges")
                .map_err(|right| (left, right))
        }
    }
}

pub fn reset(repo: &git2::Repository) {
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

// todo: config
pub fn fix_staging(repo: &git2::Repository) {
    let Ok(mut fork) = repo.find_remote("fork") else {
        return;
    };

    // git fetch fork/master
    fork.fetch(&["master"], None, None).unwrap();

    let head = repo
        .find_reference("refs/remotes/fork/master")
        .unwrap()
        .peel_to_commit()
        .unwrap();
    let head = head.as_object();

    // git reset --soft
    repo.reset(head, git2::ResetType::Soft, None).unwrap();

    let mut index = repo.index().unwrap();
    // git add .
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    // git restore --staged images
    repo.reset_default(Some(head), ["images"]).unwrap();
}

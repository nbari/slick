#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

mod common;

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_repo_discovery() {
    let (_dir, repo) = common::create_test_repo();
    let path = repo.path().parent().unwrap();

    let found = git2::Repository::discover(path);
    assert!(found.is_ok());
}

#[test]
fn test_non_git_directory() {
    let dir = TempDir::new().unwrap();
    let result = git2::Repository::discover(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_get_branch_name() {
    let (_dir, repo) = common::create_test_repo();

    fs::write(repo.path().parent().unwrap().join("test.txt"), "test").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
    index.write().unwrap();
    common::create_commit(&repo, "initial commit");

    let head = repo.head().unwrap();
    let branch = head.shorthand().unwrap();

    assert!(branch == "master" || branch == "main");
}

#[test]
fn test_detached_head() {
    let (_dir, repo) = common::create_test_repo();

    fs::write(repo.path().parent().unwrap().join("test.txt"), "test").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
    index.write().unwrap();
    common::create_commit(&repo, "initial commit");

    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    repo.set_head_detached(commit.id()).unwrap();

    let head = repo.head().unwrap();
    assert!(!head.is_branch());
}

#[test]
fn test_file_status_detection() {
    let (dir, repo) = common::create_test_repo();

    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "initial content").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();
    common::create_commit(&repo, "initial commit");

    fs::write(&file_path, "modified content").unwrap();

    let statuses = repo.statuses(None).unwrap();
    assert!(!statuses.is_empty());

    let status = statuses.get(0).unwrap();
    assert!(status.status().is_wt_modified());
}

#[test]
fn test_staged_changes() {
    let (dir, repo) = common::create_test_repo();

    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "initial").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();
    common::create_commit(&repo, "initial commit");

    fs::write(&file_path, "modified").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();

    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();
    let diff = repo.diff_tree_to_index(Some(&tree), None, None).unwrap();
    let stats = diff.stats().unwrap();

    assert!(stats.files_changed() > 0);
}

#[test]
fn test_untracked_files() {
    let (dir, repo) = common::create_test_repo();

    fs::write(dir.path().join("untracked.txt"), "content").unwrap();

    let statuses = repo.statuses(None).unwrap();
    assert_eq!(statuses.len(), 1);

    let status = statuses.get(0).unwrap();
    assert!(status.status().is_wt_new());
}

#[test]
fn test_get_user_name_from_config() {
    let (_dir, repo) = common::create_test_repo();

    let config = repo.config().unwrap();
    let name = config.get_string("user.name");

    assert!(name.is_ok());
    assert_eq!(name.unwrap(), "Test User");
}

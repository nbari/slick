#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

mod common;

use slick::git;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn stage_paths(repo: &git2::Repository, paths: &[&str]) {
    let mut index = repo.index().unwrap();
    for path in paths {
        index.add_path(Path::new(path)).unwrap();
    }
    index.write().unwrap();
}

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

#[test]
fn test_unborn_main_reports_branch_staged_and_added() {
    let (dir, repo) = common::create_test_repo();
    repo.set_head("refs/heads/main").unwrap();

    fs::write(dir.path().join("staged.txt"), "staged content\n").unwrap();
    stage_paths(&repo, &["staged.txt"]);

    let prompt = git::build_prompt_fast(&repo);
    assert_eq!(prompt.branch, "main");
    assert!(prompt.staged);
    assert_eq!(git::get_status(&repo).unwrap(), "A 1");
}

#[test]
fn test_staged_rename_reports_rename() {
    let (dir, repo) = common::create_test_repo();
    fs::write(
        dir.path().join("before.txt"),
        "content preserved by the rename\n",
    )
    .unwrap();
    stage_paths(&repo, &["before.txt"]);
    common::create_commit(&repo, "initial commit");

    fs::rename(dir.path().join("before.txt"), dir.path().join("after.txt")).unwrap();
    let mut index = repo.index().unwrap();
    index.remove_path(Path::new("before.txt")).unwrap();
    index.add_path(Path::new("after.txt")).unwrap();
    index.write().unwrap();

    assert_eq!(git::get_status(&repo).unwrap(), "R 1");
}

#[test]
fn test_worktree_rename_into_new_nested_directory_reports_rename() {
    let (dir, repo) = common::create_test_repo();
    fs::write(
        dir.path().join("before.txt"),
        "content moved into a nested directory\n",
    )
    .unwrap();
    stage_paths(&repo, &["before.txt"]);
    common::create_commit(&repo, "initial commit");

    let nested_dir = dir.path().join("new").join("nested");
    fs::create_dir_all(&nested_dir).unwrap();
    fs::rename(dir.path().join("before.txt"), nested_dir.join("after.txt")).unwrap();

    assert_eq!(git::get_status(&repo).unwrap(), "R 1");
}

#[test]
fn test_staged_rename_with_worktree_modification_reports_rename() {
    let (dir, repo) = common::create_test_repo();
    fs::write(
        dir.path().join("before.txt"),
        "content staged as part of the rename\n",
    )
    .unwrap();
    stage_paths(&repo, &["before.txt"]);
    common::create_commit(&repo, "initial commit");

    fs::rename(dir.path().join("before.txt"), dir.path().join("after.txt")).unwrap();
    let mut index = repo.index().unwrap();
    index.remove_path(Path::new("before.txt")).unwrap();
    index.add_path(Path::new("after.txt")).unwrap();
    index.write().unwrap();
    drop(index);

    fs::write(
        dir.path().join("after.txt"),
        "content staged as part of the rename\nmodified after staging\n",
    )
    .unwrap();

    assert_eq!(git::get_status(&repo).unwrap(), "R 1");
}

#[test]
fn test_worktree_rename_with_staged_modification_reports_rename() {
    let (dir, repo) = common::create_test_repo();
    fs::write(dir.path().join("before.txt"), "original content\n").unwrap();
    stage_paths(&repo, &["before.txt"]);
    common::create_commit(&repo, "initial commit");

    fs::write(
        dir.path().join("before.txt"),
        "original content\nstaged modification\n",
    )
    .unwrap();
    stage_paths(&repo, &["before.txt"]);
    fs::rename(dir.path().join("before.txt"), dir.path().join("after.txt")).unwrap();

    assert_eq!(git::get_status(&repo).unwrap(), "R 1");
}

#[test]
fn test_unrelated_staged_add_and_delete_remain_distinct() {
    let (dir, repo) = common::create_test_repo();
    fs::write(
        dir.path().join("removed.txt"),
        "old deletion payload\n".repeat(100),
    )
    .unwrap();
    stage_paths(&repo, &["removed.txt"]);
    common::create_commit(&repo, "initial commit");

    fs::remove_file(dir.path().join("removed.txt")).unwrap();
    fs::write(
        dir.path().join("added.txt"),
        "completely unrelated addition\n".repeat(100),
    )
    .unwrap();
    let mut index = repo.index().unwrap();
    index.remove_path(Path::new("removed.txt")).unwrap();
    index.add_path(Path::new("added.txt")).unwrap();
    index.write().unwrap();

    assert_eq!(git::get_status(&repo).unwrap(), "D 1 A 1");
}

#[cfg(unix)]
#[test]
fn test_get_status_is_deterministic_for_mixed_states() {
    use git2::build::CheckoutBuilder;
    use std::os::unix::fs::symlink;

    let (dir, repo) = common::create_test_repo();
    for (path, contents) in [
        ("conflicted.txt", "common ancestor\n"),
        ("modified.txt", "initial modified file\n"),
        ("deleted.txt", "old deletion payload\n"),
        ("rename-before.txt", "rename payload remains identical\n"),
        ("typechanged.txt", "regular file before type change\n"),
        ("modified_modified.txt", "initial twice-modified file\n"),
    ] {
        fs::write(dir.path().join(path), contents).unwrap();
    }
    stage_paths(
        &repo,
        &[
            "conflicted.txt",
            "modified.txt",
            "deleted.txt",
            "rename-before.txt",
            "typechanged.txt",
            "modified_modified.txt",
        ],
    );
    common::create_commit(&repo, "initial commit");

    let main_ref = repo.head().unwrap().name().unwrap().to_owned();
    let base = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("status-conflict", &base, false).unwrap();

    fs::write(dir.path().join("conflicted.txt"), "ours\n").unwrap();
    stage_paths(&repo, &["conflicted.txt"]);
    common::create_commit(&repo, "ours");

    repo.set_head("refs/heads/status-conflict").unwrap();
    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .unwrap();
    fs::write(dir.path().join("conflicted.txt"), "theirs\n").unwrap();
    stage_paths(&repo, &["conflicted.txt"]);
    common::create_commit(&repo, "theirs");
    let theirs_id = repo.head().unwrap().target().unwrap();

    repo.set_head(&main_ref).unwrap();
    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .unwrap();
    let theirs = repo.find_annotated_commit(theirs_id).unwrap();
    repo.merge(&[&theirs], None, None).unwrap();
    assert!(repo.index().unwrap().has_conflicts());

    fs::write(
        dir.path().join("modified.txt"),
        "worktree modification with a different length\n",
    )
    .unwrap();
    fs::remove_file(dir.path().join("deleted.txt")).unwrap();
    fs::rename(
        dir.path().join("rename-before.txt"),
        dir.path().join("rename-after.txt"),
    )
    .unwrap();
    fs::remove_file(dir.path().join("typechanged.txt")).unwrap();
    symlink("modified.txt", dir.path().join("typechanged.txt")).unwrap();
    fs::write(dir.path().join("added_modified.txt"), "staged addition\n").unwrap();
    fs::write(
        dir.path().join("modified_modified.txt"),
        "staged modification\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("added.txt"),
        "new staged payload unlike the deleted file\n",
    )
    .unwrap();

    let mut index = repo.index().unwrap();
    index.remove_path(Path::new("rename-before.txt")).unwrap();
    index.add_path(Path::new("rename-after.txt")).unwrap();
    index.add_path(Path::new("added_modified.txt")).unwrap();
    index.add_path(Path::new("modified_modified.txt")).unwrap();
    index.add_path(Path::new("added.txt")).unwrap();
    index.write().unwrap();
    drop(index);

    fs::write(
        dir.path().join("added_modified.txt"),
        "unstaged change after the staged addition\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("modified_modified.txt"),
        "unstaged change after the staged modification\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("untracked.txt"),
        "untracked payload unlike every tracked file\n",
    )
    .unwrap();

    let expected = "UU 1 AM 1 MM 1 M 1 D 1 R 1 T 1 A 1 ?? 1";
    for _ in 0..32 {
        assert_eq!(git::get_status(&repo).unwrap(), expected);
    }
}

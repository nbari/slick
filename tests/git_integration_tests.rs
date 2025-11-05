use git2::{Repository, Signature};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_repo() -> (TempDir, Repository) {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    // Set up git config
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();

    (dir, repo)
}

fn create_commit(repo: &Repository, message: &str) {
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = Signature::now("Test User", "test@example.com").unwrap();

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    if let Some(parent) = parent_commit {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
            .unwrap();
    } else {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
            .unwrap();
    }
}

#[test]
fn test_repo_discovery() {
    let (_dir, repo) = create_test_repo();
    let path = repo.path().parent().unwrap();

    // Should find the repository
    let found = Repository::discover(path);
    assert!(found.is_ok());
}

#[test]
fn test_non_git_directory() {
    let dir = TempDir::new().unwrap();
    let result = Repository::discover(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_get_branch_name() {
    let (_dir, repo) = create_test_repo();

    // Create initial commit to establish HEAD
    fs::write(repo.path().parent().unwrap().join("test.txt"), "test").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
    index.write().unwrap();
    create_commit(&repo, "initial commit");

    let head = repo.head().unwrap();
    let branch = head.shorthand().unwrap();

    // Default branch is usually "master" or "main"
    assert!(branch == "master" || branch == "main");
}

#[test]
fn test_detached_head() {
    let (_dir, repo) = create_test_repo();

    // Create initial commit
    fs::write(repo.path().parent().unwrap().join("test.txt"), "test").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
    index.write().unwrap();
    create_commit(&repo, "initial commit");

    // Get the commit SHA
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();

    // Detach HEAD
    repo.set_head_detached(commit.id()).unwrap();

    let head = repo.head().unwrap();
    assert!(!head.is_branch());
}

#[test]
fn test_file_status_detection() {
    let (dir, repo) = create_test_repo();

    // Create and commit a file
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "initial content").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();
    create_commit(&repo, "initial commit");

    // Modify the file
    fs::write(&file_path, "modified content").unwrap();

    // Check status
    let statuses = repo.statuses(None).unwrap();
    assert!(!statuses.is_empty());

    let status = statuses.get(0).unwrap();
    assert!(status.status().is_wt_modified());
}

#[test]
fn test_staged_changes() {
    let (dir, repo) = create_test_repo();

    // Create initial commit
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "initial").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();
    create_commit(&repo, "initial commit");

    // Modify and stage
    fs::write(&file_path, "modified").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(PathBuf::from("file.txt").as_path()).unwrap();
    index.write().unwrap();

    // Check if there are staged changes
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();
    let diff = repo.diff_tree_to_index(Some(&tree), None, None).unwrap();
    let stats = diff.stats().unwrap();

    assert!(stats.files_changed() > 0);
}

#[test]
fn test_untracked_files() {
    let (dir, repo) = create_test_repo();

    // Create untracked file
    fs::write(dir.path().join("untracked.txt"), "content").unwrap();

    let statuses = repo.statuses(None).unwrap();
    assert_eq!(statuses.len(), 1);

    let status = statuses.get(0).unwrap();
    assert!(status.status().is_wt_new());
}

#[test]
fn test_get_user_name_from_config() {
    let (_dir, repo) = create_test_repo();

    let config = repo.config().unwrap();
    let name = config.get_string("user.name");

    assert!(name.is_ok());
    assert_eq!(name.unwrap(), "Test User");
}

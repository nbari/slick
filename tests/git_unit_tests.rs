#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use git2::{Repository, Signature};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper functions (copied from git_integration_tests.rs)
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

// Unit tests for src/git.rs functions
mod git_unit_tests {
    use super::*;
    use slick::git; // Import the git module

    #[test]
    fn test_unix_timestamp() {
        let timestamp = git::unix_timestamp();
        assert!(timestamp > 0); // Should be a positive value
        // Cannot assert exact value due to time dependency, but can check it's sensible
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Allow a small delta for execution time
        assert!(timestamp >= now - 5 && timestamp <= now + 5);
    }

    #[test]
    fn test_get_action_no_action() {
        let (_dir, repo) = create_test_repo();
        create_commit(&repo, "initial commit");
        assert_eq!(git::get_action(&repo), None);
    }

    // Test for staged changes
    #[test]
    fn test_is_staged_no_changes() {
        let (_dir, repo) = create_test_repo();
        create_commit(&repo, "initial commit");
        assert!(!git::is_staged(&repo).unwrap());
    }

    #[test]
    fn test_is_staged_with_staged_changes() {
        let (dir, repo) = create_test_repo();
        create_commit(&repo, "initial commit");

        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
        index.write().unwrap();

        assert!(git::is_staged(&repo).unwrap());
    }

    // Test for ahead/behind remote
    #[test]
    fn test_is_ahead_behind_remote_no_remote() {
        let (_dir, repo) = create_test_repo();
        create_commit(&repo, "initial commit");
        let (ahead, behind) = git::is_ahead_behind_remote(&repo);
        assert_eq!(ahead, 0);
        assert_eq!(behind, 0);
    }

    // More tests could be added for specific git actions (rebase, merge etc.)
    // and for ahead/behind when a remote is configured.
}

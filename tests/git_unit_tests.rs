#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

mod common;

use std::fs;
use std::path::PathBuf;

mod git_unit_tests {
    use super::*;
    use slick::git;

    #[test]
    fn test_unix_timestamp() {
        let timestamp = git::unix_timestamp();
        assert!(timestamp > 0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(timestamp >= now - 5 && timestamp <= now + 5);
    }

    #[test]
    fn test_get_action_no_action() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");
        assert_eq!(git::get_action(&repo), None);
    }

    #[test]
    fn test_get_action_merge() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        // Simulate a merge in progress by creating MERGE_HEAD
        let git_dir = repo.path();
        fs::write(
            git_dir.join("MERGE_HEAD"),
            "0000000000000000000000000000000000000000\n",
        )
        .unwrap();

        assert_eq!(git::get_action(&repo), Some(git::ACTION_MERGE.to_owned()));
    }

    #[test]
    fn test_get_action_rebase_merge() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        // Simulate an interactive rebase by creating rebase-merge/
        let git_dir = repo.path();
        fs::create_dir_all(git_dir.join("rebase-merge")).unwrap();

        assert_eq!(
            git::get_action(&repo),
            Some(git::ACTION_REBASE_M.to_owned())
        );
    }

    #[test]
    fn test_get_action_rebase_apply() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        // rebase-apply/ exists but neither rebasing nor applying sub-file → AM_REBASE
        let git_dir = repo.path();
        fs::create_dir_all(git_dir.join("rebase-apply")).unwrap();

        assert_eq!(
            git::get_action(&repo),
            Some(git::ACTION_AM_REBASE.to_owned())
        );
    }

    #[test]
    fn test_get_action_cherry_pick() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        // CHERRY_PICK_HEAD without sequencer/ → ACTION_CHERRY
        let git_dir = repo.path();
        fs::write(
            git_dir.join("CHERRY_PICK_HEAD"),
            "0000000000000000000000000000000000000000\n",
        )
        .unwrap();

        assert_eq!(git::get_action(&repo), Some(git::ACTION_CHERRY.to_owned()));
    }

    #[test]
    fn test_get_action_bisect() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        let git_dir = repo.path();
        fs::write(git_dir.join("BISECT_LOG"), "").unwrap();

        assert_eq!(git::get_action(&repo), Some(git::ACTION_BISECT.to_owned()));
    }

    #[test]
    fn test_is_staged_no_changes() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");
        assert!(!git::is_staged(&repo).unwrap());
    }

    #[test]
    fn test_is_staged_with_staged_changes() {
        let (dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");

        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("test.txt").as_path()).unwrap();
        index.write().unwrap();

        assert!(git::is_staged(&repo).unwrap());
    }

    #[test]
    fn test_is_ahead_behind_remote_no_remote() {
        let (_dir, repo) = common::create_test_repo();
        common::create_commit(&repo, "initial commit");
        let (ahead, behind) = git::is_ahead_behind_remote(&repo);
        assert_eq!(ahead, 0);
        assert_eq!(behind, 0);
    }
}

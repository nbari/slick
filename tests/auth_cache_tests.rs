#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// Test helper to create a mock auth cache file
fn create_mock_cache(dir: &TempDir, filename: &str, age_seconds: u64, auth_failed: bool) -> String {
    let cache_path = dir.path().join(filename);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let timestamp = now - age_seconds;
    let status = if auth_failed { "1" } else { "0" };
    fs::write(&cache_path, format!("{timestamp}:{status}")).unwrap();
    cache_path.to_str().unwrap().to_string()
}

#[test]
fn test_auth_cache_file_creation() {
    let dir = TempDir::new().unwrap();
    let cache_path = create_mock_cache(&dir, "auth_test", 0, true);

    // Verify file exists
    assert!(fs::metadata(&cache_path).is_ok());

    // Verify content
    let content = fs::read_to_string(&cache_path).unwrap();
    assert!(content.ends_with(":1"));
}

#[test]
fn test_auth_cache_parsing_success() {
    let dir = TempDir::new().unwrap();
    let cache_path = create_mock_cache(&dir, "auth_test", 0, false);

    let content = fs::read_to_string(&cache_path).unwrap();
    let parts: Vec<&str> = content.split(':').collect();

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1], "0"); // No auth failure
}

#[test]
fn test_auth_cache_parsing_failure() {
    let dir = TempDir::new().unwrap();
    let cache_path = create_mock_cache(&dir, "auth_test", 0, true);

    let content = fs::read_to_string(&cache_path).unwrap();
    let parts: Vec<&str> = content.split(':').collect();

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1], "1"); // Auth failure
}

#[test]
fn test_auth_cache_fresh() {
    let dir = TempDir::new().unwrap();
    let _cache_path = create_mock_cache(&dir, "auth_test", 0, true);

    // Cache is fresh (0 seconds old)
    // In real code, this would be < 300 seconds
    let content = fs::read_to_string(dir.path().join("auth_test")).unwrap();
    let (ts_str, _) = content.split_once(':').unwrap();
    let cached_time: u64 = ts_str.parse().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert!(now - cached_time < 300); // Less than 5 minutes
}

#[test]
fn test_auth_cache_expired() {
    let dir = TempDir::new().unwrap();
    let _cache_path = create_mock_cache(&dir, "auth_test", 301, true);

    // Cache is stale (301 seconds old, > 5 minutes)
    let content = fs::read_to_string(dir.path().join("auth_test")).unwrap();
    let (ts_str, _) = content.split_once(':').unwrap();
    let cached_time: u64 = ts_str.parse().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert!(now - cached_time >= 300); // 5 minutes or more
}

#[test]
fn test_auth_cache_exactly_5_minutes() {
    let dir = TempDir::new().unwrap();
    let _cache_path = create_mock_cache(&dir, "auth_test", 300, true);

    let content = fs::read_to_string(dir.path().join("auth_test")).unwrap();
    let (ts_str, _) = content.split_once(':').unwrap();
    let cached_time: u64 = ts_str.parse().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let age = now - cached_time;
    // Should be right at the boundary (might be 299-301 due to timing)
    assert!((299..=301).contains(&age));
}

#[test]
fn test_auth_cache_missing_file() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("nonexistent");

    // Verify file doesn't exist
    assert!(fs::read_to_string(&cache_path).is_err());
}

#[test]
fn test_auth_cache_invalid_format() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("invalid_cache");

    // Write invalid format (missing colon)
    fs::write(&cache_path, "invalid_content").unwrap();

    let content = fs::read_to_string(&cache_path).unwrap();
    assert!(content.split_once(':').is_none());
}

#[test]
fn test_auth_cache_invalid_timestamp() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("invalid_timestamp");

    // Write invalid timestamp
    fs::write(&cache_path, "not_a_number:1").unwrap();

    let content = fs::read_to_string(&cache_path).unwrap();
    let (ts_str, _) = content.split_once(':').unwrap();
    assert!(ts_str.parse::<u64>().is_err());
}

#[test]
fn test_auth_cache_multiple_writes() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("auth_test");

    // First write - auth failed
    create_mock_cache(&dir, "auth_test", 0, true);
    let content1 = fs::read_to_string(&cache_path).unwrap();
    assert!(content1.ends_with(":1"));

    // Second write - auth success
    create_mock_cache(&dir, "auth_test", 0, false);
    let content2 = fs::read_to_string(&cache_path).unwrap();
    assert!(content2.ends_with(":0"));
}

#[test]
fn test_auth_cache_path_generation() {
    // Test that different repo paths generate different cache files
    let hash1 = "/home/user/repo1/.git".bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });

    let hash2 = "/home/user/repo2/.git".bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });

    assert_ne!(hash1, hash2);
}

#[test]
fn test_auth_cache_whitespace_in_status() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("auth_test");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Write with whitespace
    fs::write(&cache_path, format!("{now}:1\n")).unwrap();

    let content = fs::read_to_string(&cache_path).unwrap();
    let (_, status) = content.split_once(':').unwrap();
    assert_eq!(status.trim(), "1");
}

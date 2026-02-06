//! Tests for prompt elapsed time handling
//!
//! These tests verify that the prompt command correctly handles
//! elapsed time values, including negative values from clock adjustments.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::process::Command;

fn get_slick_binary() -> String {
    // Try release binary first, then debug
    let release_path = "./target/release/slick";
    let debug_path = "./target/debug/slick";

    if std::path::Path::new(release_path).exists() {
        release_path.to_string()
    } else if std::path::Path::new(debug_path).exists() {
        debug_path.to_string()
    } else {
        panic!("slick binary not found in target/release or target/debug");
    }
}

#[test]
fn test_negative_elapsed_time_does_not_error() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "-3", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(
        output.status.success(),
        "Command should succeed with negative elapsed time. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should not contain error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "Should not have 'unexpected argument' error"
    );
}

#[test]
fn test_negative_elapsed_time_shows_no_elapsed() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "-3", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Negative elapsed should be treated as 0, which won't show elapsed time
    // (default threshold is 5 seconds)
    // The output should NOT contain time format patterns like "s", "m", "h", "d"
    // followed by time indicators in the elapsed time color section
    assert!(
        !stdout.contains("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
        "Should not show elapsed time for negative values"
    );
}

#[test]
fn test_zero_elapsed_time_shows_no_elapsed() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 0 seconds should not show elapsed time (threshold is 5)
    assert!(
        !stdout.contains("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
        "Should not show elapsed time for 0 seconds"
    );
}

#[test]
fn test_positive_elapsed_time_above_threshold_shows_elapsed() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "10", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 10 seconds should show elapsed time (above threshold of 5)
    // Should contain "10s" in the output
    assert!(
        stdout.contains("10s"),
        "Should show '10s' for 10 seconds elapsed"
    );
}

#[test]
fn test_large_negative_elapsed_time() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "-999", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(
        output.status.success(),
        "Command should succeed with large negative elapsed time"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("999"),
        "Should not show negative elapsed time in output"
    );
}

#[test]
fn test_elapsed_time_below_threshold_not_shown() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "3", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 3 seconds should not show elapsed time (below threshold of 5)
    assert!(
        !stdout.contains("3s"),
        "Should not show elapsed time for values below threshold"
    );
}

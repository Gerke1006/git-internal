//! Integration tests for pack_stats function.

mod common;

use git_internal::hash::{HashKind, set_hash_kind_for_test};
use git_internal::internal::pack::stats::pack_stats;

/// Test: decode a small SHA-1 pack file and check statistics.
#[test]
fn test_pack_stats_small_sha1() {
    let _guard = set_hash_kind_for_test(HashKind::Sha1);
    let (path, _guard) = common::download_pack_file("small-sha1.pack");
    let stats = pack_stats(&path.to_string_lossy()).unwrap();

    // Total objects should be greater than 0
    assert!(stats.total > 0, "Total objects should be > 0");

    // Sum of all types should equal total
    let sum = stats.commits + stats.trees + stats.blobs + stats.tags + stats.deltas;
    assert_eq!(stats.total, sum, "Sum of types should equal total");

    println!("small-sha1 stats: {:?}", stats);
}

/// Test: decode a small SHA-256 pack file and check statistics.
#[test]
fn test_pack_stats_small_sha256() {
    let _guard = set_hash_kind_for_test(HashKind::Sha256);
    let (path, _guard) = common::download_pack_file("small-sha256.pack");
    let stats = pack_stats(&path.to_string_lossy()).unwrap();

    assert!(stats.total > 0, "Total objects should be > 0");

    let sum = stats.commits + stats.trees + stats.blobs + stats.tags + stats.deltas;
    assert_eq!(stats.total, sum, "Sum of types should equal total");

    println!("small-sha256 stats: {:?}", stats);
}

/// Test: file not found should return an error (not panic).
#[test]
fn test_pack_stats_file_not_found() {
    let result = pack_stats("nonexistent.pack");
    assert!(result.is_err(), "Should return error for missing file");
}

/// Test: invalid file (not a pack file) should return an error.
#[test]
fn test_pack_stats_invalid_file() {
    let _guard = set_hash_kind_for_test(HashKind::Sha1);
    let (path, _guard) = common::download_pack_file("small-sha1.idx");
    let result = pack_stats(&path.to_string_lossy());
    assert!(result.is_err(), "Should return error for invalid pack file");
}

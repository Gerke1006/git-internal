//! Pack file statistics: count objects and type distribution.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::errors::GitError;
use crate::hash::ObjectHash;
use crate::internal::object::types::ObjectType;
use crate::internal::pack::Pack;

/// Statistics about a Git pack file.
///
/// Contains the total number of objects and a breakdown by type.
#[derive(Debug, Clone)]
pub struct PackStats {
    /// Total number of objects in the pack file.
    pub total: usize,
    /// Number of commit objects.
    pub commits: usize,
    /// Number of tree objects.
    pub trees: usize,
    /// Number of blob objects.
    pub blobs: usize,
    /// Number of tag objects.
    pub tags: usize,
    /// Number of delta objects (OffsetDelta, OffsetZstdelta, HashDelta).
    pub deltas: usize,
}

/// Analyze a pack file and return statistics about its contents.
///
/// This function decodes the pack file and counts each object by type.
/// It reuses the existing `Pack::decode` logic rather than implementing
/// a separate decode path.
///
/// # Arguments
/// * `pack_path` - Path to the `.pack` file.
///
/// # Returns
/// A `PackStats` struct with object counts, or a `GitError` if the
/// file cannot be read or decoded.
///
/// # Examples
/// ```no_run
/// use git_internal::internal::pack::stats::pack_stats;
///
/// let stats = pack_stats("tests/data/packs/small-sha1.pack").unwrap();
/// println!("Total objects: {}", stats.total);
/// ```
pub fn pack_stats(pack_path: &str) -> Result<PackStats, GitError> {
    // 1. Check if file exists
    let path = Path::new(pack_path);
    if !path.exists() {
        return Err(GitError::InvalidPackFile(format!(
            "File not found: {}",
            pack_path
        )));
    }

    // 2. Open the file
    let f = File::open(path).map_err(|e| {
        GitError::InvalidPackFile(format!("Failed to open file: {}", e))
    })?;
    let mut reader = BufReader::new(f);

    // 3. Create a Pack instance
    let mut pack = Pack::new(None, None, None, true);

    // 4. Create a shared counter for the callback
    //    Arc = shared ownership across threads
    //    Mutex = safe mutable access from multiple threads
    let stats = Arc::new(Mutex::new(PackStats {
        total: 0,
        commits: 0,
        trees: 0,
        blobs: 0,
        tags: 0,
        deltas: 0,
    }));

    // 5. Clone for the callback (Arc::clone just increments a reference count)
    let stats_clone = Arc::clone(&stats);

    // 6. Decode the pack file, counting objects in the callback
    pack.decode(
        &mut reader,
        move |entry| {
            let mut s = stats_clone.lock().unwrap();
            s.total += 1;
            match entry.inner.obj_type {
                ObjectType::Commit => s.commits += 1,
                ObjectType::Tree => s.trees += 1,
                ObjectType::Blob => s.blobs += 1,
                ObjectType::Tag => s.tags += 1,
                _ => s.deltas += 1,
            }
        },
        None::<fn(ObjectHash)>,
    )?;

    // 7. Extract the final result
    let result = Arc::try_unwrap(stats)
        .expect("All references should be gone after decode completes")
        .into_inner()
        .expect("Mutex should not be poisoned");

    Ok(result)
}

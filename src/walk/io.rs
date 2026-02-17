// src/walk/io.rs

use memmap2::Mmap;
use std::{fs::File, path::Path};

/// Opens a file and memory-maps it as read-only.
///
/// Returns `None` if:
/// - the file cannot be opened
/// - the file cannot be memory-mapped
///
/// # Safety
/// All unsafe code is contained within this function.
/// The mapping is read-only and valid for the lifetime of the returned `Mmap`.
pub fn map_file(path: &Path) -> Option<Mmap> {
    let file = File::open(path).ok()?;

    // SAFETY:
    // - The file descriptor outlives the Mmap
    // - The mapping is read-only
    // - We do not mutate through the mapping
    unsafe { Mmap::map(&file).ok() }
}

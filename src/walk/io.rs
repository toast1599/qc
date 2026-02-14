// src/walk/io.rs

use memmap2::Mmap;
use std::{fs::File, path::Path};

/// Opens a file and memory-maps it.
/// 
/// Returns `None` if the file cannot be opened or mapped.
/// All unsafe code is contained here.
pub fn map_file(path: &Path) -> Option<Mmap> {
    let file = File::open(path).ok()?;

    // SAFETY:
    // - The file descriptor lives as long as `file`
    // - The OS guarantees the mapping remains valid
    // - We never mutate through the mapping
    let mmap = unsafe { Mmap::map(&file) }.ok()?;

    Some(mmap)
}

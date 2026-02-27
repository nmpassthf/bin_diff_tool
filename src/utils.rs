mod fs;
mod hash;

pub use fs::{FileInfo, is_text_file, scan_directory};
pub use hash::{HashResult, compute_file_hash};

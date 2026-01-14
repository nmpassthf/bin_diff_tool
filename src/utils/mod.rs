mod fs;
mod hash;

pub use fs::{is_text_file, scan_directory};
pub use hash::compute_file_hash;

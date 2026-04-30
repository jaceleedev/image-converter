#[cfg(test)]
mod tests;

pub mod batch;
pub mod converter;
pub mod interactive;
pub mod utils;

pub use batch::{convert_directory, BatchSummary};
pub use converter::{convert_image, convert_image_silent, ConvertStats};
pub use utils::format_file_size;

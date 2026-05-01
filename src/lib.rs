#[cfg(test)]
mod tests;

pub mod batch;
pub mod converter;
pub mod error;
pub mod format;
pub mod interactive;
pub mod utils;

pub use batch::{convert_directory, BatchSummary};
pub use converter::{convert_image, convert_image_silent, ConvertStats};
pub use error::{ConverterError, Result};
pub use format::OutputFormat;
pub use utils::format_file_size;

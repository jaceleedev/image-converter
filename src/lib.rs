#[cfg(test)]
mod tests;

pub mod batch;
pub mod converter;
pub mod error;
pub mod format;
pub mod interactive;
pub mod utils;

pub use batch::{convert_directory, convert_directory_with_options, BatchSummary};
pub use converter::{
    convert_image, convert_image_silent, convert_image_silent_with_options,
    convert_image_with_options, ConvertStats, ResizeOptions,
};
pub use error::{ConverterError, Result};
pub use format::OutputFormat;
pub use utils::{format_file_size, format_quality_label};

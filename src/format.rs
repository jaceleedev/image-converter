use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

use crate::error::ConverterError;

/// 지원하는 출력 이미지 포맷
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Png,
    Jpg,
    Jpeg,
    Webp,
    Avif,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
            Self::Avif => "avif",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpg => "JPG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WEBP",
            Self::Avif => "AVIF",
        }
    }

    pub fn allowed_extensions(self) -> &'static [&'static str] {
        match self {
            Self::Png => &["png"],
            Self::Jpg | Self::Jpeg => &["jpg", "jpeg"],
            Self::Webp => &["webp"],
            Self::Avif => &["avif"],
        }
    }

    pub fn allowed_extensions_label(self) -> &'static str {
        match self {
            Self::Png => ".png",
            Self::Jpg | Self::Jpeg => ".jpg 또는 .jpeg",
            Self::Webp => ".webp",
            Self::Avif => ".avif",
        }
    }

    pub fn matches_extension(self, extension: &str) -> bool {
        self.allowed_extensions()
            .iter()
            .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    }

    pub fn is_png(self) -> bool {
        matches!(self, Self::Png)
    }

    pub fn is_avif(self) -> bool {
        matches!(self, Self::Avif)
    }

    pub fn is_jpeg(self) -> bool {
        matches!(self, Self::Jpg | Self::Jpeg)
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for OutputFormat {
    type Err = ConverterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" => Ok(Self::Jpg),
            "jpeg" => Ok(Self::Jpeg),
            "webp" => Ok(Self::Webp),
            "avif" => Ok(Self::Avif),
            _ => Err(ConverterError::UnsupportedFormat(s.to_string())),
        }
    }
}

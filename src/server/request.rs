use super::error::ApiError;
use crate::{input::is_supported_input_extension, ConversionOptions, JpegBackground, OutputFormat};
use axum::{body::Bytes, extract::Multipart};
use std::{path::Path, str::FromStr};

#[derive(Debug)]
pub(super) struct UploadedImage {
    pub(super) file_name: String,
    pub(super) extension: String,
    pub(super) bytes: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct ConvertRequest {
    pub(super) image: UploadedImage,
    pub(super) format: OutputFormat,
    pub(super) quality: f32,
    pub(super) options: ConversionOptions,
}

pub(super) async fn parse_multipart(
    mut multipart: Multipart,
    max_upload_bytes: usize,
) -> Result<ConvertRequest, ApiError> {
    let mut image = None;
    let mut format = None;
    let mut quality = None;
    let mut max_width = None;
    let mut jpeg_background = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(str::to_owned);
        let file_name = field.file_name().map(str::to_owned);
        let bytes = field.bytes().await?;
        let Some(name) = name else {
            continue;
        };

        match name.as_str() {
            "file" => {
                if image.is_some() {
                    return Err(ApiError::bad_request(
                        "duplicate_file",
                        "이미지 파일은 하나만 업로드할 수 있습니다",
                    ));
                }
                if bytes.is_empty() {
                    return Err(ApiError::bad_request(
                        "empty_file",
                        "업로드 파일이 비어 있습니다",
                    ));
                }
                if bytes.len() > max_upload_bytes {
                    return Err(ApiError::payload_too_large(format!(
                        "업로드 파일은 최대 {max_upload_bytes} bytes 까지만 허용됩니다"
                    )));
                }

                let file_name = file_name.ok_or_else(|| {
                    ApiError::bad_request(
                        "missing_file_name",
                        "업로드 파일 이름을 확인할 수 없습니다",
                    )
                })?;
                let extension = input_extension(&file_name)?;
                image = Some(UploadedImage {
                    file_name,
                    extension,
                    bytes: bytes.to_vec(),
                });
            }
            "format" => {
                format = Some(parse_output_format(text_field("format", bytes)?)?);
            }
            "quality" => {
                quality = Some(parse_quality(&text_field("quality", bytes)?)?);
            }
            "max_width" => {
                let value = text_field("max_width", bytes)?;
                if !value.trim().is_empty() {
                    max_width = Some(parse_max_width(&value)?);
                }
            }
            "jpeg_background" => {
                let value = text_field("jpeg_background", bytes)?;
                if !value.trim().is_empty() {
                    jpeg_background = Some(parse_jpeg_background(&value)?);
                }
            }
            _ => {}
        }
    }

    let image =
        image.ok_or_else(|| ApiError::bad_request("missing_file", "file 필드가 필요합니다"))?;
    let format = format
        .ok_or_else(|| ApiError::bad_request("missing_format", "format 필드가 필요합니다"))?;
    let options =
        ConversionOptions::for_format(format, max_width, jpeg_background).map_err(|_| {
            ApiError::bad_request(
                "invalid_jpeg_background",
                "jpeg_background 은 JPG/JPEG 출력에서만 사용할 수 있습니다",
            )
        })?;

    Ok(ConvertRequest {
        image,
        format,
        quality: quality.unwrap_or(90.0),
        options,
    })
}

fn text_field(name: &'static str, bytes: Bytes) -> Result<String, ApiError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        ApiError::bad_request(
            "invalid_text_field",
            format!("{name} 필드는 UTF-8 문자열이어야 합니다"),
        )
    })
}

fn parse_output_format(value: String) -> Result<OutputFormat, ApiError> {
    OutputFormat::from_str(value.trim()).map_err(|_| {
        ApiError::bad_request(
            "unsupported_output_format",
            format!("지원하지 않는 출력 포맷입니다: {}", value.trim()),
        )
    })
}

fn parse_quality(value: &str) -> Result<f32, ApiError> {
    let quality = value.trim().parse::<f32>().map_err(|_| {
        ApiError::bad_request(
            "invalid_quality",
            format!("quality 는 1-100 숫자여야 합니다: {}", value.trim()),
        )
    })?;
    if !(1.0..=100.0).contains(&quality) {
        return Err(ApiError::bad_request(
            "invalid_quality",
            format!("quality 는 1-100 범위여야 합니다: {quality}"),
        ));
    }
    Ok(quality)
}

fn parse_max_width(value: &str) -> Result<u32, ApiError> {
    let max_width = value.trim().parse::<u32>().map_err(|_| {
        ApiError::bad_request(
            "invalid_max_width",
            format!(
                "max_width 는 1 이상의 픽셀 값이어야 합니다: {}",
                value.trim()
            ),
        )
    })?;
    if max_width == 0 {
        return Err(ApiError::bad_request(
            "invalid_max_width",
            "max_width 는 1 이상이어야 합니다",
        ));
    }
    Ok(max_width)
}

fn parse_jpeg_background(value: &str) -> Result<JpegBackground, ApiError> {
    JpegBackground::from_name_or_hex(value)
        .map_err(|message| ApiError::bad_request("invalid_jpeg_background", message))
}

fn input_extension(file_name: &str) -> Result<String, ApiError> {
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ApiError::bad_request(
                "missing_input_extension",
                "업로드 파일 확장자를 확인할 수 없습니다",
            )
        })?
        .to_ascii_lowercase();

    if is_supported_input_extension(&extension) {
        Ok(extension)
    } else {
        Err(ApiError::bad_request(
            "unsupported_input_extension",
            format!("지원하지 않는 입력 확장자입니다: .{extension}"),
        ))
    }
}

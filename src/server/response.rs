use axum::{
    body::Body,
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderName, HeaderValue,
    },
    response::Response,
};
use std::path::Path;

use crate::{ConvertStats, OutputFormat};

pub(super) struct ConvertedImage {
    pub(super) bytes: Vec<u8>,
    pub(super) stats: ConvertStats,
    pub(super) format: OutputFormat,
    pub(super) file_name: String,
}

impl ConvertedImage {
    pub(super) fn into_response(self) -> Response {
        let mut response = Response::new(Body::from(self.bytes));
        let headers = response.headers_mut();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(content_type(self.format)),
        );
        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", self.file_name))
                .expect("download file name은 ASCII 안전 문자열이어야 함"),
        );
        insert_number_header(headers, "x-input-size", self.stats.input_size);
        insert_number_header(headers, "x-output-size", self.stats.output_size);
        insert_number_header(headers, "x-input-width", self.stats.width);
        insert_number_header(headers, "x-input-height", self.stats.height);
        insert_number_header(headers, "x-output-width", self.stats.output_width);
        insert_number_header(headers, "x-output-height", self.stats.output_height);
        response
    }
}

pub(super) fn output_extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Jpg | OutputFormat::Jpeg => "jpg",
        _ => format.as_str(),
    }
}

pub(super) fn download_file_name(input_file_name: &str, format: OutputFormat) -> String {
    let stem = Path::new(input_file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "converted".to_string());
    format!("{stem}.{}", output_extension(format))
}

pub(super) fn header_name(name: &'static str) -> HeaderName {
    HeaderName::from_static(name)
}

fn content_type(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Png => "image/png",
        OutputFormat::Jpg | OutputFormat::Jpeg => "image/jpeg",
        OutputFormat::Webp => "image/webp",
        OutputFormat::Avif => "image/avif",
    }
}

fn sanitize_file_stem(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn insert_number_header(headers: &mut HeaderMap, name: &'static str, value: impl ToString) {
    headers.insert(
        header_name(name),
        HeaderValue::from_str(&value.to_string()).expect("숫자 헤더 값은 유효해야 함"),
    );
}

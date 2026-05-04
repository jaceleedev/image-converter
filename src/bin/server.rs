use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use image_converter::{
    convert_image_silent_with_conversion_options, input::register_extra_decoders,
    ConversionOptions, ConvertStats, JpegBackground, OutputFormat, ResizeOptions,
};
use serde::Serialize;
use std::{env, fs, net::SocketAddr, path::Path, str::FromStr, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{OwnedSemaphorePermit, Semaphore},
    task, time,
};
use tower_http::cors::CorsLayer;

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 4000;
const DEFAULT_ALLOWED_ORIGIN: &str = "http://localhost:3000";
const DEFAULT_MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
const DEFAULT_MAX_PIXELS: u64 = 80_000_000;
const DEFAULT_MAX_CONCURRENCY: usize = 2;
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
struct ServerConfig {
    max_upload_bytes: usize,
    max_pixels: u64,
    max_concurrency: usize,
    conversion_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            max_pixels: DEFAULT_MAX_PIXELS,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            conversion_timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        }
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let timeout_seconds = parse_env_u64("CONVERT_TIMEOUT_SECONDS", DEFAULT_TIMEOUT_SECONDS)?;
        Ok(Self {
            max_upload_bytes: parse_env_usize(
                "CONVERT_MAX_UPLOAD_BYTES",
                DEFAULT_MAX_UPLOAD_BYTES,
            )?,
            max_pixels: parse_env_u64("CONVERT_MAX_PIXELS", DEFAULT_MAX_PIXELS)?,
            max_concurrency: parse_env_usize("CONVERT_MAX_CONCURRENCY", DEFAULT_MAX_CONCURRENCY)?
                .max(1),
            conversion_timeout: Duration::from_secs(timeout_seconds.max(1)),
        })
    }
}

#[derive(Clone)]
struct AppState {
    conversions: Arc<Semaphore>,
    config: ServerConfig,
}

#[derive(Debug)]
struct UploadedImage {
    file_name: String,
    extension: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct ConvertRequest {
    image: UploadedImage,
    format: OutputFormat,
    quality: f32,
    options: ConversionOptions,
}

struct ConvertedImage {
    bytes: Vec<u8>,
    stats: ConvertStats,
    format: OutputFormat,
    file_name: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: ErrorDetail<'a>,
}

#[derive(Serialize)]
struct ErrorDetail<'a> {
    code: &'a str,
    message: &'a str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_env()?;
    let host = env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let port = parse_env_u16("PORT", DEFAULT_PORT)?;
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let allowed_origin = allowed_origin_from_env()?;
    let app = build_router(config, allowed_origin);

    println!("이미지 변환 API 서버 실행 중: http://{addr}");
    println!(
        "업로드 제한: {} bytes, 픽셀 제한: {}, 동시 변환: {}",
        config.max_upload_bytes, config.max_pixels, config.max_concurrency
    );

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(config: ServerConfig, allowed_origin: HeaderValue) -> Router {
    let exposed_headers = [
        CONTENT_DISPOSITION,
        header_name("x-input-size"),
        header_name("x-output-size"),
        header_name("x-input-width"),
        header_name("x-input-height"),
        header_name("x-output-width"),
        header_name("x-output-height"),
    ];
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .expose_headers(exposed_headers);

    let state = AppState {
        conversions: Arc::new(Semaphore::new(config.max_concurrency)),
        config,
    };

    Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/v1/convert",
            post(convert).layer(DefaultBodyLimit::max(
                config.max_upload_bytes + MULTIPART_OVERHEAD_BYTES,
            )),
        )
        .layer(cors)
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn convert(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, ApiError> {
    let request = parse_multipart(multipart, state.config.max_upload_bytes).await?;
    let converted = run_conversion(state, request).await?;
    Ok(converted.into_response())
}

async fn parse_multipart(
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

    if jpeg_background.is_some() && !format.is_jpeg() {
        return Err(ApiError::bad_request(
            "invalid_jpeg_background",
            "jpeg_background 은 JPG/JPEG 출력에서만 사용할 수 있습니다",
        ));
    }

    Ok(ConvertRequest {
        image,
        format,
        quality: quality.unwrap_or(90.0),
        options: ConversionOptions {
            resize: max_width.map(|max_width| ResizeOptions { max_width }),
            jpeg_background,
        },
    })
}

async fn run_conversion(
    state: AppState,
    request: ConvertRequest,
) -> Result<ConvertedImage, ApiError> {
    let permit = state
        .conversions
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::internal("변환 동시성 제한기를 사용할 수 없습니다"))?;
    let timeout = state.config.conversion_timeout;
    let max_pixels = state.config.max_pixels;
    let task = task::spawn_blocking(move || convert_in_tempdir(request, permit, max_pixels));

    match time::timeout(timeout, task).await {
        Ok(joined) => joined.map_err(|error| {
            ApiError::internal(format!("변환 작업을 완료하지 못했습니다: {error}"))
        })?,
        Err(_) => Err(ApiError::timeout(format!(
            "이미지 변환이 {}초 안에 끝나지 않았습니다",
            timeout.as_secs()
        ))),
    }
}

fn convert_in_tempdir(
    request: ConvertRequest,
    permit: OwnedSemaphorePermit,
    max_pixels: u64,
) -> Result<ConvertedImage, ApiError> {
    let _permit = permit;
    let temp_dir = tempfile::tempdir()?;
    let input_path = temp_dir
        .path()
        .join(format!("input.{}", request.image.extension));
    let output_path = temp_dir
        .path()
        .join(format!("output.{}", output_extension(request.format)));

    fs::write(&input_path, request.image.bytes)?;
    ensure_pixel_limit(&input_path, max_pixels)?;

    let input_path = path_to_string(&input_path)?;
    let output_path = path_to_string(&output_path)?;
    let stats = convert_image_silent_with_conversion_options(
        &input_path,
        &output_path,
        request.format,
        request.quality,
        request.options,
    )
    .map_err(ApiError::conversion)?;
    let bytes = fs::read(&output_path)?;

    Ok(ConvertedImage {
        bytes,
        stats,
        format: request.format,
        file_name: download_file_name(&request.image.file_name, request.format),
    })
}

fn ensure_pixel_limit(path: &Path, max_pixels: u64) -> Result<(), ApiError> {
    register_extra_decoders();
    let (width, height) = image::image_dimensions(path).map_err(ApiError::image)?;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > max_pixels {
        return Err(ApiError::payload_too_large(format!(
            "이미지가 너무 큽니다: {width}x{height} px ({pixels} pixels, 제한: {max_pixels})"
        )));
    }
    Ok(())
}

impl ConvertedImage {
    fn into_response(self) -> Response {
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

impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn payload_too_large(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "payload_too_large",
            message: message.into(),
        }
    }

    fn timeout(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::REQUEST_TIMEOUT,
            code: "conversion_timeout",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }

    fn image(error: image::ImageError) -> Self {
        Self::bad_request(
            "invalid_image",
            format!("이미지를 읽을 수 없습니다: {error}"),
        )
    }

    fn conversion(error: image_converter::ConverterError) -> Self {
        match error {
            image_converter::ConverterError::Image(error) => Self::image(error),
            image_converter::ConverterError::UnsupportedFormat(format) => Self::bad_request(
                "unsupported_format",
                format!("지원하지 않는 포맷입니다: {format}"),
            ),
            other => Self::internal(format!("이미지 변환에 실패했습니다: {other}")),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: ErrorDetail {
                code: self.code,
                message: &self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        Self::internal(format!("입출력 오류: {error}"))
    }
}

impl From<axum::extract::multipart::MultipartError> for ApiError {
    fn from(error: axum::extract::multipart::MultipartError) -> Self {
        Self::bad_request("invalid_multipart", format!("multipart 요청 오류: {error}"))
    }
}

fn text_field(name: &'static str, bytes: axum::body::Bytes) -> Result<String, ApiError> {
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
    match value.trim().to_ascii_lowercase().as_str() {
        "white" => Ok(JpegBackground::white()),
        "black" => Ok(JpegBackground::black()),
        _ => JpegBackground::from_hex(value)
            .map_err(|message| ApiError::bad_request("invalid_jpeg_background", message)),
    }
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

fn is_supported_input_extension(extension: &str) -> bool {
    matches!(
        extension,
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "heic" | "heif" | "tiff" | "tif" | "bmp" | "ico"
    )
}

fn output_extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Jpg | OutputFormat::Jpeg => "jpg",
        _ => format.as_str(),
    }
}

fn content_type(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Png => "image/png",
        OutputFormat::Jpg | OutputFormat::Jpeg => "image/jpeg",
        OutputFormat::Webp => "image/webp",
        OutputFormat::Avif => "image/avif",
    }
}

fn download_file_name(input_file_name: &str, format: OutputFormat) -> String {
    let stem = Path::new(input_file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "converted".to_string());
    format!("{stem}.{}", output_extension(format))
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

fn path_to_string(path: &Path) -> Result<String, ApiError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| ApiError::internal("임시 파일 경로를 UTF-8로 변환할 수 없습니다"))
}

fn insert_number_header(
    headers: &mut axum::http::HeaderMap,
    name: &'static str,
    value: impl ToString,
) {
    headers.insert(
        header_name(name),
        HeaderValue::from_str(&value.to_string()).expect("숫자 헤더 값은 유효해야 함"),
    );
}

fn header_name(name: &'static str) -> HeaderName {
    HeaderName::from_static(name)
}

fn allowed_origin_from_env() -> Result<HeaderValue, Box<dyn std::error::Error>> {
    let value =
        env::var("CONVERT_ALLOWED_ORIGIN").unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGIN.to_string());
    Ok(HeaderValue::from_str(&value)?)
}

fn parse_env_u16(name: &str, default: u16) -> Result<u16, Box<dyn std::error::Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(Box::new(error)),
    }
}

fn parse_env_usize(name: &str, default: usize) -> Result<usize, Box<dyn std::error::Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(Box::new(error)),
    }
}

fn parse_env_u64(name: &str, default: u64) -> Result<u64, Box<dyn std::error::Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(Box::new(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use image::{ImageBuffer, ImageFormat, Rgb};
    use std::io::Cursor;
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_returns_ok() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"ok");
    }

    #[tokio::test]
    async fn convert_png_to_webp_returns_download() {
        let (content_type, body) =
            multipart_body(&[("format", "webp"), ("quality", "80"), ("max_width", "16")]);
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/convert")
                    .header(CONTENT_TYPE, content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "image/webp");
        assert_eq!(response.headers()[header_name("x-output-width")], "16");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(body.starts_with(b"RIFF"));
        assert_eq!(&body[8..12], b"WEBP");
    }

    #[tokio::test]
    async fn convert_rejects_missing_file() {
        let boundary = "missing-file-boundary";
        let body = text_multipart_body(boundary, &[("format", "webp")]);
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/convert")
                    .header(
                        CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn convert_rejects_too_many_pixels() {
        let config = ServerConfig {
            max_pixels: 10,
            ..ServerConfig::default()
        };
        let (content_type, body) = multipart_body(&[("format", "webp")]);
        let response = build_router(config, HeaderValue::from_static(DEFAULT_ALLOWED_ORIGIN))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/convert")
                    .header(CONTENT_TYPE, content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    fn test_app() -> Router {
        build_router(
            ServerConfig {
                max_upload_bytes: 1024 * 1024,
                max_pixels: 10_000,
                max_concurrency: 1,
                conversion_timeout: Duration::from_secs(30),
            },
            HeaderValue::from_static(DEFAULT_ALLOWED_ORIGIN),
        )
    }

    fn multipart_body(fields: &[(&str, &str)]) -> (String, Vec<u8>) {
        let boundary = "image-converter-test-boundary";
        let mut body = text_multipart_body(boundary, fields);
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"sample.png\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
        body.extend_from_slice(&png_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    fn text_multipart_body(boundary: &str, fields: &[(&str, &str)]) -> Vec<u8> {
        let mut body = Vec::new();
        for (name, value) in fields {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
            );
            body.extend_from_slice(value.as_bytes());
            body.extend_from_slice(b"\r\n");
        }
        body
    }

    fn png_bytes() -> Vec<u8> {
        let image = ImageBuffer::from_fn(32, 24, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255u8, 255u8, 255u8])
            } else {
                Rgb([0u8, 0u8, 0u8])
            }
        });
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }
}

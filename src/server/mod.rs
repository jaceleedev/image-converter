mod config;
mod conversion;
mod error;
mod request;
mod response;
mod routes;
mod state;

use config::{allowed_origin_from_env, socket_addr_from_env, ServerConfig};
use routes::build_router;
use tokio::net::TcpListener;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_env()?;
    let addr = socket_addr_from_env()?;
    let allowed_origin = allowed_origin_from_env()?;
    let app = build_router(config, allowed_origin);

    println!("이미지 변환 API 서버 실행 중: http://{addr}");
    println!(
        "업로드 제한: {} bytes, 픽셀 제한: {}, 동시 변환: {}, 대기 제한: {}초",
        config.max_upload_bytes,
        config.max_pixels,
        config.max_concurrency,
        config.queue_timeout.as_secs()
    );

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        config::{ServerConfig, DEFAULT_ALLOWED_ORIGIN},
        conversion::run_conversion,
        request::{ConvertRequest, UploadedImage},
        response::header_name,
        routes::build_router,
        state::AppState,
    };
    use crate::{ConversionOptions, OutputFormat};
    use axum::{
        body::{to_bytes, Body},
        http::{header::CONTENT_TYPE, HeaderValue, Method, Request, StatusCode},
        Router,
    };
    use image::{ImageBuffer, ImageFormat, Rgb};
    use std::{io::Cursor, sync::Arc, time::Duration};
    use tokio::sync::Semaphore;
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

    #[tokio::test]
    async fn conversion_queue_timeout_returns_429() {
        let state = AppState {
            conversions: Arc::new(Semaphore::new(0)),
            config: ServerConfig {
                queue_timeout: Duration::from_millis(1),
                ..ServerConfig::default()
            },
        };
        let request = ConvertRequest {
            image: UploadedImage {
                file_name: "sample.png".to_string(),
                extension: "png".to_string(),
                bytes: png_bytes(),
            },
            format: OutputFormat::Webp,
            quality: 90.0,
            options: ConversionOptions::default(),
        };

        let error = match run_conversion(state, request).await {
            Ok(_) => panic!("대기열 timeout이 발생해야 합니다"),
            Err(error) => error,
        };

        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(error.code, "conversion_queue_full");
    }

    fn test_app() -> Router {
        build_router(
            ServerConfig {
                max_upload_bytes: 1024 * 1024,
                max_pixels: 10_000,
                max_concurrency: 1,
                queue_timeout: Duration::from_secs(1),
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

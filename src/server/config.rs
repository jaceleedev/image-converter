use axum::http::HeaderValue;
use std::{env, net::SocketAddr, time::Duration};

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 4000;
pub(super) const DEFAULT_ALLOWED_ORIGIN: &str = "http://localhost:3000";
const DEFAULT_MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
const DEFAULT_MAX_PIXELS: u64 = 80_000_000;
const DEFAULT_MAX_CONCURRENCY: usize = 2;
const DEFAULT_QUEUE_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
pub(super) const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub(super) struct ServerConfig {
    pub(super) max_upload_bytes: usize,
    pub(super) max_pixels: u64,
    pub(super) max_concurrency: usize,
    pub(super) queue_timeout: Duration,
    pub(super) conversion_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            max_pixels: DEFAULT_MAX_PIXELS,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            queue_timeout: Duration::from_secs(DEFAULT_QUEUE_TIMEOUT_SECONDS),
            conversion_timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        }
    }
}

impl ServerConfig {
    pub(super) fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let queue_timeout_seconds = parse_env_u64(
            "CONVERT_QUEUE_TIMEOUT_SECONDS",
            DEFAULT_QUEUE_TIMEOUT_SECONDS,
        )?;
        let timeout_seconds = parse_env_u64("CONVERT_TIMEOUT_SECONDS", DEFAULT_TIMEOUT_SECONDS)?;
        Ok(Self {
            max_upload_bytes: parse_env_usize(
                "CONVERT_MAX_UPLOAD_BYTES",
                DEFAULT_MAX_UPLOAD_BYTES,
            )?,
            max_pixels: parse_env_u64("CONVERT_MAX_PIXELS", DEFAULT_MAX_PIXELS)?,
            max_concurrency: parse_env_usize("CONVERT_MAX_CONCURRENCY", DEFAULT_MAX_CONCURRENCY)?
                .max(1),
            queue_timeout: Duration::from_secs(queue_timeout_seconds.max(1)),
            conversion_timeout: Duration::from_secs(timeout_seconds.max(1)),
        })
    }
}

pub(super) fn socket_addr_from_env() -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let host = env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let port = parse_env_u16("PORT", DEFAULT_PORT)?;
    Ok(format!("{host}:{port}").parse()?)
}

pub(super) fn allowed_origin_from_env() -> Result<HeaderValue, Box<dyn std::error::Error>> {
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

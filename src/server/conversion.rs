use super::{
    error::ApiError,
    request::ConvertRequest,
    response::{download_file_name, output_extension, ConvertedImage},
    state::AppState,
};
use crate::{convert_image_silent_with_conversion_options, input::register_extra_decoders};
use std::{fs, path::Path};
use tokio::{sync::OwnedSemaphorePermit, task, time};

pub(super) async fn run_conversion(
    state: AppState,
    request: ConvertRequest,
) -> Result<ConvertedImage, ApiError> {
    let permit = acquire_conversion_permit(&state).await?;
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

async fn acquire_conversion_permit(state: &AppState) -> Result<OwnedSemaphorePermit, ApiError> {
    match time::timeout(
        state.config.queue_timeout,
        state.conversions.clone().acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(_)) => Err(ApiError::internal(
            "변환 동시성 제한기를 사용할 수 없습니다",
        )),
        Err(_) => Err(ApiError::too_many_requests(format!(
            "현재 처리 중인 변환이 많습니다. {}초 뒤 다시 시도하세요",
            state.config.queue_timeout.as_secs()
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

fn path_to_string(path: &Path) -> Result<String, ApiError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| ApiError::internal("임시 파일 경로를 UTF-8로 변환할 수 없습니다"))
}

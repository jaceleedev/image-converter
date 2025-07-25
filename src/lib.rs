#[cfg(test)]
mod tests;

pub mod converter;
pub mod interactive;
pub mod utils;

// Re-export commonly used functions
pub use converter::convert_image;
pub use utils::format_file_size;

use image::GenericImageView;
use ravif::{Encoder as AvifEncoder, Img, RGBA8};
use std::fs;
use webp::Encoder as WebpEncoder;

/// 테스트용 이미지 변환 함수 (출력 없음)
pub fn convert_image_silent(
    input_path: &str,
    output_path: &str,
    format: &str,
    quality: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 이미지 로드
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();
    
    match format {
        "webp" => {
            let encoder = WebpEncoder::from_image(&img)?;
            let webp_data = encoder.encode(quality);
            fs::write(output_path, &*webp_data)?;
        }
        "avif" => {
            let rgba_img = img.to_rgba8();
            let pixels: Vec<RGBA8> = rgba_img
                .pixels()
                .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
                .collect();

            let avif_encoder = AvifEncoder::new()
                .with_quality(quality)
                .with_speed(4);
            let res = avif_encoder.encode_rgba(Img::new(&pixels, width as usize, height as usize))?;
            fs::write(output_path, res.avif_file)?;
        }
        _ => return Err("지원하지 않는 포맷입니다".into()),
    }
    
    Ok(())
}
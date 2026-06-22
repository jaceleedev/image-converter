use colored::*;
use image::{
    codecs::jpeg::JpegEncoder, imageops::FilterType, DynamicImage, GenericImageView, ImageFormat,
    Rgb, RgbImage,
};
use indicatif::{ProgressBar, ProgressStyle};
use ravif::{Encoder as AvifEncoder, Img, RGBA8};
use std::fs;
use std::io::{Cursor, ErrorKind, Write};
use std::path::Path;
use webp::Encoder as WebpEncoder;

use crate::error::{ConverterError, Result};
use crate::format::OutputFormat;
use crate::input::register_extra_decoders;

/// 단일 이미지 변환 결과 통계
#[derive(Debug)]
pub struct ConvertStats {
    pub input_size: u64,
    pub output_size: u64,
    pub width: u32,
    pub height: u32,
    pub output_width: u32,
    pub output_height: u32,
}

struct PreparedImage {
    image: DynamicImage,
    width: u32,
    height: u32,
    output_width: u32,
    output_height: u32,
}

/// 변환 전 적용할 이미지 리사이즈 옵션
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResizeOptions {
    pub max_width: u32,
}

/// JPEG 출력 시 투명 픽셀 아래에 깔 배경색
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JpegBackground {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl JpegBackground {
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub fn from_hex(input: &str) -> std::result::Result<Self, String> {
        let trimmed = input.trim();
        let trimmed = trimmed.strip_prefix('#').unwrap_or(trimmed);
        if trimmed.len() != 6 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("#RRGGBB 형식으로 입력하세요".to_string());
        }

        let r =
            u8::from_str_radix(&trimmed[0..2], 16).map_err(|_| "빨간색 값을 읽을 수 없습니다")?;
        let g =
            u8::from_str_radix(&trimmed[2..4], 16).map_err(|_| "초록색 값을 읽을 수 없습니다")?;
        let b =
            u8::from_str_radix(&trimmed[4..6], 16).map_err(|_| "파란색 값을 읽을 수 없습니다")?;
        Ok(Self { r, g, b })
    }

    pub fn from_name_or_hex(input: &str) -> std::result::Result<Self, String> {
        match input.trim().to_ascii_lowercase().as_str() {
            "white" => Ok(Self::white()),
            "black" => Ok(Self::black()),
            _ => Self::from_hex(input),
        }
    }
}

/// 변환 전후에 적용할 추가 옵션
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConversionOptions {
    pub resize: Option<ResizeOptions>,
    pub jpeg_background: Option<JpegBackground>,
}

impl ConversionOptions {
    pub fn for_format(
        format: OutputFormat,
        max_width: Option<u32>,
        jpeg_background: Option<JpegBackground>,
    ) -> std::result::Result<Self, String> {
        if jpeg_background.is_some() && !format.is_jpeg() {
            return Err("JPEG 배경색은 JPG/JPEG 출력에서만 사용할 수 있습니다".to_string());
        }

        Ok(Self {
            resize: max_width.map(|max_width| ResizeOptions { max_width }),
            jpeg_background,
        })
    }
}

/// 메모리에 로드된 이미지를 지정한 포맷으로 인코딩
fn encode_to(
    img: &DynamicImage,
    format: OutputFormat,
    quality: f32,
    options: ConversionOptions,
) -> Result<Vec<u8>> {
    match format {
        OutputFormat::Webp => {
            // webp 크레이트의 from_image 는 Luma/LumaA(흑백·LA) 와 16-bit/32F 입력에서
            // Err("Unimplemented") 를 반환한다. 해당 모드는 RGBA8 로 정규화해 인코딩하고,
            // RGB8/RGBA8 입력은 기존 from_rgb/from_rgba 경로를 그대로 유지한다.
            let data = match img {
                DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_) => {
                    let encoder = WebpEncoder::from_image(img)
                        .map_err(|e| ConverterError::Webp(e.to_string()))?;
                    encoder.encode(quality).to_vec()
                }
                other => {
                    let rgba = other.to_rgba8();
                    let width = rgba.width();
                    let height = rgba.height();
                    WebpEncoder::from_rgba(rgba.as_raw(), width, height)
                        .encode(quality)
                        .to_vec()
                }
            };
            Ok(data)
        }
        OutputFormat::Avif => {
            let (width, height) = img.dimensions();
            let rgba_img = img.to_rgba8();
            let pixels: Vec<RGBA8> = rgba_img
                .pixels()
                .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
                .collect();
            let encoder = AvifEncoder::new().with_quality(quality).with_speed(4);
            let res = encoder.encode_rgba(Img::new(&pixels, width as usize, height as usize))?;
            Ok(res.avif_file)
        }
        OutputFormat::Png => {
            // PNG 는 무손실 — quality 는 의미 없음 (조용히 무시)
            let mut buf: Vec<u8> = Vec::new();
            img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
            Ok(buf)
        }
        OutputFormat::Jpg | OutputFormat::Jpeg => {
            // JPEG 는 알파 채널을 가질 수 없으므로 RGB 로 다운샘플 후 인코딩
            let q = quality.clamp(1.0, 100.0).round() as u8;
            let rgb = match options.jpeg_background {
                Some(background) => flatten_for_jpeg(img, background),
                None => DynamicImage::ImageRgb8(img.to_rgb8()),
            };
            let mut buf: Vec<u8> = Vec::new();
            JpegEncoder::new_with_quality(&mut buf, q).encode_image(&rgb)?;
            Ok(buf)
        }
    }
}

fn flatten_for_jpeg(img: &DynamicImage, background: JpegBackground) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb = RgbImage::new(width, height);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as u32;
        let inverse_alpha = 255 - alpha;
        let blend = |foreground: u8, background: u8| -> u8 {
            ((foreground as u32 * alpha + background as u32 * inverse_alpha + 127) / 255) as u8
        };
        rgb.put_pixel(
            x,
            y,
            Rgb([
                blend(pixel[0], background.r),
                blend(pixel[1], background.g),
                blend(pixel[2], background.b),
            ]),
        );
    }

    DynamicImage::ImageRgb8(rgb)
}

fn resize_image(img: DynamicImage, options: ConversionOptions) -> DynamicImage {
    let Some(resize) = options.resize else {
        return img;
    };
    let (width, height) = img.dimensions();
    if resize.max_width == 0 || width <= resize.max_width {
        return img;
    }

    let ratio = resize.max_width as f64 / width as f64;
    let resized_height = ((height as f64 * ratio).round() as u32).max(1);
    img.resize_exact(resize.max_width, resized_height, FilterType::Lanczos3)
}

fn ensure_output_available(output_path: &str) -> Result<()> {
    if std::path::Path::new(output_path).exists() {
        return Err(ConverterError::OutputExists(output_path.to_string()));
    }
    Ok(())
}

fn output_extension_mismatch(
    output_path: &str,
    format: OutputFormat,
    actual: String,
) -> ConverterError {
    ConverterError::OutputExtensionMismatch {
        output_path: output_path.to_string(),
        actual,
        expected: format.allowed_extensions_label().to_string(),
    }
}

/// 출력 파일 확장자가 선택한 출력 포맷과 일치하는지 확인
pub fn validate_output_extension(output_path: &str, format: OutputFormat) -> Result<()> {
    let actual = Path::new(output_path)
        .extension()
        .and_then(|extension| extension.to_str());

    match actual {
        Some(extension) if format.matches_extension(extension) => Ok(()),
        Some(extension) => Err(output_extension_mismatch(
            output_path,
            format,
            format!(".{extension}"),
        )),
        None => Err(output_extension_mismatch(
            output_path,
            format,
            "없음".to_string(),
        )),
    }
}

fn write_output_file(output_path: &str, data: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|e| {
            if e.kind() == ErrorKind::AlreadyExists {
                ConverterError::OutputExists(output_path.to_string())
            } else {
                ConverterError::Io(e)
            }
        })?;
    file.write_all(data)?;
    Ok(())
}

fn prepare_output_target(input_path: &str, output_path: &str) -> Result<u64> {
    let input_size = fs::metadata(input_path)?.len();
    ensure_output_available(output_path)?;
    Ok(input_size)
}

fn load_image(input_path: &str) -> Result<DynamicImage> {
    register_extra_decoders();
    Ok(image::open(input_path)?)
}

fn prepare_image(input_path: &str, options: ConversionOptions) -> Result<PreparedImage> {
    let img = load_image(input_path)?;
    let (width, height) = img.dimensions();
    let img = resize_image(img, options);
    let (output_width, output_height) = img.dimensions();

    Ok(PreparedImage {
        image: img,
        width,
        height,
        output_width,
        output_height,
    })
}

fn encode_and_write_output(
    output_path: &str,
    image: &DynamicImage,
    format: OutputFormat,
    quality: f32,
    options: ConversionOptions,
) -> Result<u64> {
    let data = encode_to(image, format, quality, options)?;
    write_output_file(output_path, &data)?;
    Ok(fs::metadata(output_path)?.len())
}

fn conversion_stats(input_size: u64, output_size: u64, image: &PreparedImage) -> ConvertStats {
    ConvertStats {
        input_size,
        output_size,
        width: image.width,
        height: image.height,
        output_width: image.output_width,
        output_height: image.output_height,
    }
}

/// 이미지 변환 (출력 없음). 테스트와 배치 모드에서 사용
pub fn convert_image_silent(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
) -> Result<ConvertStats> {
    convert_image_silent_with_conversion_options(
        input_path,
        output_path,
        format,
        quality,
        ConversionOptions::default(),
    )
}

/// 이미지 변환 (출력 없음). 리사이즈 같은 추가 옵션을 적용할 때 사용
pub fn convert_image_silent_with_options(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
    resize: Option<ResizeOptions>,
) -> Result<ConvertStats> {
    convert_image_silent_with_conversion_options(
        input_path,
        output_path,
        format,
        quality,
        ConversionOptions {
            resize,
            ..ConversionOptions::default()
        },
    )
}

/// 이미지 변환 (출력 없음). 리사이즈와 JPEG 배경색 같은 추가 옵션을 적용할 때 사용
pub fn convert_image_silent_with_conversion_options(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
    options: ConversionOptions,
) -> Result<ConvertStats> {
    validate_output_extension(output_path, format)?;
    let input_size = prepare_output_target(input_path, output_path)?;
    let image = prepare_image(input_path, options)?;
    let output_size = encode_and_write_output(output_path, &image.image, format, quality, options)?;
    Ok(conversion_stats(input_size, output_size, &image))
}

/// 단일 이미지 변환 (진행률 표시 + 결과 출력)
pub fn convert_image(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
) -> Result<()> {
    convert_image_with_conversion_options(
        input_path,
        output_path,
        format,
        quality,
        ConversionOptions::default(),
    )
}

/// 단일 이미지 변환 (진행률 표시 + 결과 출력). 리사이즈 같은 추가 옵션을 적용할 때 사용
pub fn convert_image_with_options(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
    resize: Option<ResizeOptions>,
) -> Result<()> {
    convert_image_with_conversion_options(
        input_path,
        output_path,
        format,
        quality,
        ConversionOptions {
            resize,
            ..ConversionOptions::default()
        },
    )
}

/// 단일 이미지 변환 (진행률 표시 + 결과 출력). 리사이즈와 JPEG 배경색 같은 추가 옵션을 적용할 때 사용
pub fn convert_image_with_conversion_options(
    input_path: &str,
    output_path: &str,
    format: OutputFormat,
    quality: f32,
    options: ConversionOptions,
) -> Result<()> {
    validate_output_extension(output_path, format)?;

    println!("\n{} 이미지 변환을 시작합니다...", "🚀".bright_blue());

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}% {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    pb.set_message("파일 분석 중...");
    pb.set_position(10);
    let input_size = prepare_output_target(input_path, output_path)?;

    pb.set_position(20);
    pb.set_message("이미지 로딩 중...");
    let img = load_image(input_path)?;
    let (width, height) = img.dimensions();

    pb.set_position(35);
    pb.set_message("크기 조정 중...");
    let img = resize_image(img, options);
    let (output_width, output_height) = img.dimensions();
    let image = PreparedImage {
        image: img,
        width,
        height,
        output_width,
        output_height,
    };

    pb.set_position(40);
    pb.set_message(format!(
        "{} 인코딩 중... {}",
        format.display_name(),
        if format.is_avif() {
            "(시간이 걸릴 수 있습니다)"
        } else {
            ""
        }
    ));
    let data = encode_to(&image.image, format, quality, options)?;

    pb.set_position(80);
    pb.set_message("파일 저장 중...");
    write_output_file(output_path, &data)?;

    pb.set_position(100);
    pb.finish_with_message("✅ 변환 완료!");

    let output_size = fs::metadata(output_path)?.len();
    let stats = conversion_stats(input_size, output_size, &image);
    print_single_summary(input_path, output_path, &stats, format, quality);
    Ok(())
}

fn print_single_summary(
    input_path: &str,
    output_path: &str,
    stats: &ConvertStats,
    format: OutputFormat,
    quality: f32,
) {
    let reduction =
        ((stats.input_size as f64 - stats.output_size as f64) / stats.input_size as f64) * 100.0;
    let quality_label = crate::utils::format_quality_label(format, quality);

    println!("\n{} 변환 결과:", "📊".bright_blue());
    println!(
        "  {} 원본: {} ({}x{} px)",
        "📁".bright_yellow(),
        crate::utils::format_file_size(stats.input_size).bright_yellow(),
        stats.width,
        stats.height
    );
    println!(
        "  {} 변환: {} ({}{})",
        "💾".bright_green(),
        crate::utils::format_file_size(stats.output_size).bright_green(),
        quality_label,
        format_output_dimensions(stats)
    );

    let emoji = pick_reduction_emoji(reduction);
    println!(
        "  {} 용량 감소: {:.1}% {}",
        emoji,
        reduction.abs(),
        if reduction > 0.0 {
            "↓".bright_green()
        } else {
            "↑".bright_red()
        }
    );

    println!(
        "\n{} 변환 완료: {} → {}",
        "✨".bright_magenta(),
        input_path.bright_cyan(),
        output_path.bright_cyan()
    );
}

fn format_output_dimensions(stats: &ConvertStats) -> String {
    if stats.width == stats.output_width && stats.height == stats.output_height {
        String::new()
    } else {
        format!(", {}x{} px", stats.output_width, stats.output_height)
    }
}

fn pick_reduction_emoji(reduction: f64) -> &'static str {
    if reduction > 50.0 {
        "🎉"
    } else if reduction > 30.0 {
        "👍"
    } else if reduction > 10.0 {
        "✅"
    } else {
        "📊"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn flatten_for_jpeg_composites_transparent_pixels_over_background() {
        let mut rgba = RgbaImage::new(2, 1);
        rgba.put_pixel(0, 0, Rgba([255, 0, 0, 0]));
        rgba.put_pixel(1, 0, Rgba([255, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(rgba);

        let flattened = flatten_for_jpeg(&img, JpegBackground::white()).to_rgb8();

        assert_eq!(flattened.get_pixel(0, 0).0, [255, 255, 255]);
        assert_eq!(flattened.get_pixel(1, 0).0, [255, 0, 0]);
    }

    #[test]
    fn flatten_for_jpeg_blends_partial_alpha() {
        let mut rgba = RgbaImage::new(1, 1);
        rgba.put_pixel(0, 0, Rgba([255, 0, 0, 128]));
        let img = DynamicImage::ImageRgba8(rgba);

        let flattened = flatten_for_jpeg(&img, JpegBackground::black()).to_rgb8();

        assert_eq!(flattened.get_pixel(0, 0).0, [128, 0, 0]);
    }
}

use colored::*;
use image::{DynamicImage, GenericImageView};
use indicatif::{ProgressBar, ProgressStyle};
use ravif::{Encoder as AvifEncoder, Img, RGBA8};
use std::fs;
use webp::Encoder as WebpEncoder;

use crate::error::{ConverterError, Result};

/// 단일 이미지 변환 결과 통계
#[derive(Debug)]
pub struct ConvertStats {
    pub input_size: u64,
    pub output_size: u64,
    pub width: u32,
    pub height: u32,
}

/// 메모리에 로드된 이미지를 지정한 포맷으로 인코딩
fn encode_to(img: &DynamicImage, format: &str, quality: f32) -> Result<Vec<u8>> {
    match format {
        "webp" => {
            let encoder = WebpEncoder::from_image(img)
                .map_err(|e| ConverterError::Webp(e.to_string()))?;
            let data = encoder.encode(quality);
            Ok(data.to_vec())
        }
        "avif" => {
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
        _ => Err(ConverterError::UnsupportedFormat(format.to_string())),
    }
}

/// 이미지 변환 (출력 없음). 테스트와 배치 모드에서 사용
pub fn convert_image_silent(
    input_path: &str,
    output_path: &str,
    format: &str,
    quality: f32,
) -> Result<ConvertStats> {
    let input_size = fs::metadata(input_path)?.len();
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();
    let data = encode_to(&img, format, quality)?;
    fs::write(output_path, &data)?;
    let output_size = fs::metadata(output_path)?.len();
    Ok(ConvertStats {
        input_size,
        output_size,
        width,
        height,
    })
}

/// 단일 이미지 변환 (진행률 표시 + 결과 출력)
pub fn convert_image(
    input_path: &str,
    output_path: &str,
    format: &str,
    quality: f32,
) -> Result<()> {
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
    let input_size = fs::metadata(input_path)?.len();

    pb.set_position(20);
    pb.set_message("이미지 로딩 중...");
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();

    pb.set_position(40);
    pb.set_message(format!(
        "{} 인코딩 중... {}",
        format.to_uppercase(),
        if format == "avif" {
            "(시간이 걸릴 수 있습니다)"
        } else {
            ""
        }
    ));
    let data = encode_to(&img, format, quality)?;

    pb.set_position(80);
    pb.set_message("파일 저장 중...");
    fs::write(output_path, &data)?;

    pb.set_position(100);
    pb.finish_with_message("✅ 변환 완료!");

    let output_size = fs::metadata(output_path)?.len();
    print_single_summary(
        input_path,
        output_path,
        &ConvertStats {
            input_size,
            output_size,
            width,
            height,
        },
        quality,
    );
    Ok(())
}

fn print_single_summary(input_path: &str, output_path: &str, stats: &ConvertStats, quality: f32) {
    let reduction =
        ((stats.input_size as f64 - stats.output_size as f64) / stats.input_size as f64) * 100.0;

    println!("\n{} 변환 결과:", "📊".bright_blue());
    println!(
        "  {} 원본: {} ({}x{} px)",
        "📁".bright_yellow(),
        crate::utils::format_file_size(stats.input_size).bright_yellow(),
        stats.width,
        stats.height
    );
    println!(
        "  {} 변환: {} (품질: {}%)",
        "💾".bright_green(),
        crate::utils::format_file_size(stats.output_size).bright_green(),
        quality as u32
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

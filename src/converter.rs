use colored::*;
use image::GenericImageView;
use indicatif::{ProgressBar, ProgressStyle};
use ravif::{Encoder as AvifEncoder, Img, RGBA8};
use std::fs;
use webp::Encoder as WebpEncoder;

/// 이미지 변환 함수 (진행률 표시 포함)
pub fn convert_image(
    input_path: &str,
    output_path: &str,
    format: &str,
    quality: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 변환 시작 메시지
    println!("\n{} 이미지 변환을 시작합니다...", "🚀".bright_blue());

    // 진행 바 설정
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}% {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    // 원본 파일 크기 확인
    pb.set_message("파일 분석 중...");
    pb.set_position(10);
    let input_size = fs::metadata(input_path)?.len();
    
    pb.set_position(20);
    pb.set_message("이미지 로딩 중...");

    // 이미지 로드
    let img = image::open(input_path)?;
    let (width, height) = img.dimensions();
    
    pb.set_position(30);
    pb.set_message(format!("{} 형식으로 변환 준비 중...", format.to_uppercase()));

    match format {
        "webp" => {
            pb.set_position(40);
            pb.set_message("WebP 인코더 생성 중...");
            let encoder = WebpEncoder::from_image(&img)?;
            
            pb.set_position(60);
            pb.set_message("WebP 인코딩 중...");
            let webp_data = encoder.encode(quality);
            
            pb.set_position(80);
            pb.set_message("파일 저장 중...");
            fs::write(output_path, &*webp_data)?;
        }
        "avif" => {
            pb.set_position(40);
            pb.set_message("RGBA 픽셀 데이터 준비 중...");
            
            let rgba_img = img.to_rgba8();
            let pixels: Vec<RGBA8> = rgba_img
                .pixels()
                .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
                .collect();

            pb.set_position(50);
            pb.set_message("AVIF 인코더 설정 중...");
            
            let avif_encoder = AvifEncoder::new()
                .with_quality(quality)
                .with_speed(4);
                
            pb.set_position(60);
            pb.set_message("AVIF 인코딩 중... (시간이 걸릴 수 있습니다)");
            
            let res = avif_encoder.encode_rgba(Img::new(&pixels, width as usize, height as usize))?;
            
            pb.set_position(80);
            pb.set_message("파일 저장 중...");
            
            fs::write(output_path, res.avif_file)?;
        }
        _ => return Err("지원하지 않는 포맷입니다".into()),
    }

    pb.set_position(90);
    pb.set_message("변환 마무리 중...");

    // 변환된 파일 크기 확인
    let output_size = fs::metadata(output_path)?.len();
    let reduction = ((input_size as f64 - output_size as f64) / input_size as f64) * 100.0;

    pb.set_message("변환 완료!");
    pb.set_position(100);
    pb.finish_with_message("✅ 변환 완료!");

    // 결과 출력 (진행률 바가 완료된 후)
    println!("\n{} 변환 결과:", "📊".bright_blue());
    println!("  {} 원본: {} ({}x{} px)", 
        "📁".bright_yellow(),
        crate::utils::format_file_size(input_size).bright_yellow(),
        width,
        height
    );
    println!("  {} 변환: {} (품질: {}%)",
        "💾".bright_green(),
        crate::utils::format_file_size(output_size).bright_green(),
        quality as u32
    );

    let emoji = if reduction > 50.0 {
        "🎉"
    } else if reduction > 30.0 {
        "👍"
    } else if reduction > 10.0 {
        "✅"
    } else {
        "📊"
    };

    println!("  {} 용량 감소: {:.1}% {}",
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

    Ok(())
}
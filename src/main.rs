use clap::Parser;
use colored::*;
use std::path::Path;

use image_converter::{convert_directory, convert_image, interactive::interactive_mode};

/// PNG/JPG/JPEG/WebP/AVIF/TIFF/BMP/ICO 이미지를 PNG/JPG/WebP/AVIF 로 변환합니다 (단일 파일 + 디렉토리 일괄)
#[derive(Parser, Debug)]
#[command(name = "image_converter", version = "2.4", about, long_about = None)]
struct Cli {
    /// 대화형 모드로 실행
    #[arg(short = 'I', long)]
    interactive: bool,

    /// 변환할 입력 이미지 파일 또는 디렉토리 경로
    #[arg(short, long, value_name = "PATH", required_unless_present = "interactive")]
    input: Option<String>,

    /// 출력 파일 또는 디렉토리 경로
    #[arg(short, long, value_name = "PATH", required_unless_present = "interactive")]
    output: Option<String>,

    /// 출력 포맷 (png, jpg, jpeg, webp, avif)
    #[arg(short, long, value_name = "FORMAT", required_unless_present = "interactive")]
    format: Option<String>,

    /// 변환 품질 1-100 (PNG 는 무손실이라 무시됨, 기본값: 90)
    #[arg(short, long, default_value_t = 90.0)]
    quality: f32,

    /// 디렉토리 입력 시 하위 폴더까지 재귀 변환
    #[arg(short, long)]
    recursive: bool,

    /// 디렉토리 모드에서 사용할 스레드 수 (미지정 시 RAYON_NUM_THREADS 또는 CPU 코어 수)
    #[arg(short, long, value_name = "N")]
    threads: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    let result = if cli.interactive {
        interactive_mode()
    } else {
        let input = cli.input.expect("input은 비대화형 모드에서 필수입니다");
        let output = cli.output.expect("output은 비대화형 모드에서 필수입니다");
        let format = cli.format.expect("format은 비대화형 모드에서 필수입니다");

        if Path::new(&input).is_dir() {
            convert_directory(
                &input,
                &output,
                &format,
                cli.quality,
                cli.recursive,
                cli.threads,
            )
            .map(|_| ())
        } else {
            convert_image(&input, &output, &format, cli.quality)
        }
    };

    if let Err(e) = result {
        eprintln!("{} 오류: {}", "❌".bright_red(), e);
        std::process::exit(1);
    }
}

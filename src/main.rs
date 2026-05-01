use clap::{error::ErrorKind, CommandFactory, Parser};
use colored::*;
use std::path::Path;

use image_converter::{
    convert_directory, convert_image, interactive::interactive_mode, OutputFormat,
};

fn parse_quality(s: &str) -> Result<f32, String> {
    let q: f32 = s
        .parse()
        .map_err(|_| format!("'{s}' 는 유효한 숫자가 아닙니다"))?;
    if !(1.0..=100.0).contains(&q) {
        return Err(format!("품질은 1.0~100.0 범위여야 합니다 (입력: {q})"));
    }
    Ok(q)
}

fn parse_threads(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|_| format!("'{s}' 는 유효한 정수가 아닙니다"))?;
    if n == 0 {
        return Err("스레드 수는 1 이상이어야 합니다".into());
    }
    Ok(n)
}

/// PNG/JPG/JPEG/WebP/AVIF/TIFF/BMP/ICO 이미지를 PNG/JPG/WebP/AVIF 로 변환합니다 (단일 파일 + 디렉토리 일괄)
#[derive(Parser, Debug)]
#[command(name = "image_converter", version, about, long_about = None)]
struct Cli {
    /// 대화형 모드로 실행
    #[arg(short = 'I', long)]
    interactive: bool,

    /// 변환할 입력 이미지 파일 또는 디렉토리 경로
    #[arg(short, long, value_name = "PATH")]
    input: Option<String>,

    /// 출력 파일 또는 디렉토리 경로
    #[arg(short, long, value_name = "PATH")]
    output: Option<String>,

    /// 출력 포맷 (png, jpg, jpeg, webp, avif)
    #[arg(short, long, value_name = "FORMAT", value_enum, ignore_case = true)]
    format: Option<OutputFormat>,

    /// 변환 품질 1-100 (PNG 는 무손실이라 무시됨, 기본값: 90)
    #[arg(short, long, default_value_t = 90.0, value_parser = parse_quality)]
    quality: f32,

    /// 디렉토리 입력 시 하위 폴더까지 재귀 변환
    #[arg(short, long)]
    recursive: bool,

    /// 디렉토리 모드에서 사용할 스레드 수 (1 이상, 미지정 시 RAYON_NUM_THREADS 또는 CPU 코어 수)
    #[arg(short, long, value_name = "N", value_parser = parse_threads)]
    threads: Option<usize>,
}

fn should_enter_interactive(cli: &Cli, invoked_without_args: bool) -> bool {
    cli.interactive || invoked_without_args
}

fn missing_non_interactive_args(cli: &Cli) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if cli.input.is_none() {
        missing.push("-i/--input");
    }
    if cli.output.is_none() {
        missing.push("-o/--output");
    }
    if cli.format.is_none() {
        missing.push("-f/--format");
    }
    missing
}

fn main() {
    let invoked_without_args = std::env::args_os().len() == 1;
    let cli = Cli::parse();

    let result = if should_enter_interactive(&cli, invoked_without_args) {
        interactive_mode()
    } else {
        let missing = missing_non_interactive_args(&cli);
        if !missing.is_empty() {
            Cli::command()
                .error(
                    ErrorKind::MissingRequiredArgument,
                    format!(
                        "비대화형 모드에서는 {} 옵션이 필요합니다. 인자 없이 실행하면 대화형 모드로 시작합니다.",
                        missing.join(", ")
                    ),
                )
                .exit();
        }

        let input = cli.input.expect("input은 비대화형 모드에서 필수입니다");
        let output = cli.output.expect("output은 비대화형 모드에서 필수입니다");
        let format = cli.format.expect("format은 비대화형 모드에서 필수입니다");

        if Path::new(&input).is_dir() {
            convert_directory(
                &input,
                &output,
                format,
                cli.quality,
                cli.recursive,
                cli.threads,
            )
            .map(|_| ())
        } else {
            convert_image(&input, &output, format, cli.quality)
        }
    };

    if let Err(e) = result {
        eprintln!("{} 오류: {}", "❌".bright_red(), e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quality_accepts_valid_range() {
        assert_eq!(parse_quality("1").unwrap(), 1.0);
        assert_eq!(parse_quality("50.5").unwrap(), 50.5);
        assert_eq!(parse_quality("100").unwrap(), 100.0);
    }

    #[test]
    fn parse_quality_rejects_out_of_range() {
        assert!(parse_quality("0").is_err());
        assert!(parse_quality("0.99").is_err());
        assert!(parse_quality("100.01").is_err());
        assert!(parse_quality("-10").is_err());
        assert!(parse_quality("200").is_err());
    }

    #[test]
    fn parse_quality_rejects_non_numeric() {
        let err = parse_quality("abc").unwrap_err();
        assert!(err.contains("유효한 숫자가 아닙니다"));
    }

    #[test]
    fn parse_threads_accepts_positive() {
        assert_eq!(parse_threads("1").unwrap(), 1);
        assert_eq!(parse_threads("16").unwrap(), 16);
    }

    #[test]
    fn parse_threads_rejects_zero() {
        let err = parse_threads("0").unwrap_err();
        assert!(err.contains("1 이상"));
    }

    #[test]
    fn parse_threads_rejects_non_numeric() {
        assert!(parse_threads("abc").is_err());
        assert!(parse_threads("-1").is_err());
    }

    #[test]
    fn parse_format_accepts_valid_values_case_insensitive() {
        let cli = Cli::try_parse_from([
            "image_converter",
            "-i",
            "input.png",
            "-o",
            "output.webp",
            "-f",
            "WEBP",
        ])
        .unwrap();
        assert_eq!(cli.format, Some(OutputFormat::Webp));
    }

    #[test]
    fn parse_format_rejects_invalid_value() {
        let err = Cli::try_parse_from([
            "image_converter",
            "-i",
            "input.png",
            "-o",
            "output.xyz",
            "-f",
            "xyz",
        ])
        .unwrap_err()
        .to_string();
        assert!(err.contains("xyz"));
        assert!(err.contains("png") && err.contains("webp") && err.contains("avif"));
    }

    #[test]
    fn parse_no_args_is_valid_for_interactive_default() {
        let cli = Cli::try_parse_from(["image_converter"]).unwrap();
        assert!(should_enter_interactive(&cli, true));
        assert!(missing_non_interactive_args(&cli).contains(&"-i/--input"));
    }

    #[test]
    fn interactive_flag_enters_interactive_even_with_no_paths() {
        let cli = Cli::try_parse_from(["image_converter", "-I"]).unwrap();
        assert!(should_enter_interactive(&cli, false));
    }

    #[test]
    fn non_interactive_mode_reports_missing_required_args() {
        let cli = Cli::try_parse_from(["image_converter", "-q", "80"]).unwrap();
        assert!(!should_enter_interactive(&cli, false));
        assert_eq!(
            missing_non_interactive_args(&cli),
            vec!["-i/--input", "-o/--output", "-f/--format"]
        );
    }

    #[test]
    fn non_interactive_mode_accepts_required_args() {
        let cli = Cli::try_parse_from([
            "image_converter",
            "-i",
            "input.png",
            "-o",
            "output.webp",
            "-f",
            "webp",
        ])
        .unwrap();
        assert!(!should_enter_interactive(&cli, false));
        assert!(missing_non_interactive_args(&cli).is_empty());
    }
}

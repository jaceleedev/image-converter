use clap::{error::ErrorKind, CommandFactory, Parser};
use colored::*;
use std::path::Path;

use image_converter::{
    convert_directory_with_conversion_options, convert_image_with_conversion_options,
    interactive::interactive_mode, BatchSummary, ConversionOptions, ConverterError, JpegBackground,
    OutputFormat, ResizeOptions,
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

fn parse_max_width(s: &str) -> Result<u32, String> {
    let n: u32 = s
        .parse()
        .map_err(|_| format!("'{s}' 는 유효한 픽셀 값이 아닙니다"))?;
    if n == 0 {
        return Err("최대 가로 크기는 1 이상이어야 합니다".into());
    }
    Ok(n)
}

fn parse_jpeg_background(s: &str) -> Result<JpegBackground, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "white" => Ok(JpegBackground::white()),
        "black" => Ok(JpegBackground::black()),
        _ => JpegBackground::from_hex(s),
    }
}

fn build_conversion_options(
    format: OutputFormat,
    max_width: Option<u32>,
    jpeg_background: Option<JpegBackground>,
) -> Result<ConversionOptions, String> {
    if jpeg_background.is_some() && !format.is_jpeg() {
        return Err("--jpeg-background 옵션은 JPG/JPEG 출력에서만 사용할 수 있습니다".into());
    }

    Ok(ConversionOptions {
        resize: max_width.map(|max_width| ResizeOptions { max_width }),
        jpeg_background,
    })
}

/// 대화형 안내로 웹용 이미지를 PNG/JPG/WebP/AVIF 로 변환합니다
#[derive(Parser, Debug)]
#[command(
    name = "image_converter",
    version,
    about,
    long_about = "인자 없이 실행하면 대화형 모드로 시작합니다. -i/-o/-f 옵션은 반복 작업이나 스크립트 자동화가 필요할 때 사용할 수 있습니다."
)]
struct Cli {
    /// 대화형 모드로 명시 실행
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

    /// 최대 가로 크기(px). 원본보다 작을 때만 비율 유지 축소
    #[arg(long, value_name = "PX", value_parser = parse_max_width)]
    max_width: Option<u32>,

    /// JPEG 출력 시 투명 영역 배경색 (white, black, #RRGGBB)
    #[arg(long, value_name = "COLOR", value_parser = parse_jpeg_background)]
    jpeg_background: Option<JpegBackground>,
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

fn batch_summary_to_result(summary: BatchSummary) -> image_converter::Result<()> {
    if summary.failed > 0 {
        Err(ConverterError::BatchPartialFailure {
            failed: summary.failed,
        })
    } else {
        Ok(())
    }
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
        let conversion_options =
            match build_conversion_options(format, cli.max_width, cli.jpeg_background) {
                Ok(options) => options,
                Err(message) => Cli::command()
                    .error(ErrorKind::ValueValidation, message)
                    .exit(),
            };

        if Path::new(&input).is_dir() {
            convert_directory_with_conversion_options(
                &input,
                &output,
                format,
                cli.quality,
                cli.recursive,
                cli.threads,
                conversion_options,
            )
            .and_then(batch_summary_to_result)
        } else {
            convert_image_with_conversion_options(
                &input,
                &output,
                format,
                cli.quality,
                conversion_options,
            )
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
    fn parse_max_width_accepts_positive_pixels() {
        assert_eq!(parse_max_width("1").unwrap(), 1);
        assert_eq!(parse_max_width("1600").unwrap(), 1600);
    }

    #[test]
    fn parse_max_width_rejects_invalid_values() {
        assert!(parse_max_width("0").is_err());
        assert!(parse_max_width("-1").is_err());
        assert!(parse_max_width("abc").is_err());
    }

    #[test]
    fn parse_jpeg_background_accepts_named_and_hex_values() {
        assert_eq!(
            parse_jpeg_background("white").unwrap(),
            JpegBackground::white()
        );
        assert_eq!(
            parse_jpeg_background("BLACK").unwrap(),
            JpegBackground::black()
        );
        assert_eq!(
            parse_jpeg_background("#1A2b3C").unwrap(),
            JpegBackground {
                r: 0x1A,
                g: 0x2B,
                b: 0x3C,
            }
        );
    }

    #[test]
    fn parse_jpeg_background_rejects_invalid_values() {
        assert!(parse_jpeg_background("#fff").is_err());
        assert!(parse_jpeg_background("blue").is_err());
    }

    #[test]
    fn build_conversion_options_maps_cli_values() {
        let options = build_conversion_options(
            OutputFormat::Jpeg,
            Some(1200),
            Some(JpegBackground::black()),
        )
        .unwrap();

        assert_eq!(options.resize, Some(ResizeOptions { max_width: 1200 }));
        assert_eq!(options.jpeg_background, Some(JpegBackground::black()));
    }

    #[test]
    fn build_conversion_options_rejects_jpeg_background_for_non_jpeg() {
        let err = build_conversion_options(OutputFormat::Webp, None, Some(JpegBackground::white()))
            .unwrap_err();
        assert!(err.contains("JPG/JPEG 출력"));
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

    #[test]
    fn non_interactive_mode_parses_conversion_options() {
        let cli = Cli::try_parse_from([
            "image_converter",
            "-i",
            "input.png",
            "-o",
            "output.jpg",
            "-f",
            "jpeg",
            "--max-width",
            "800",
            "--jpeg-background",
            "#ffffff",
        ])
        .unwrap();

        assert_eq!(cli.max_width, Some(800));
        assert_eq!(cli.jpeg_background, Some(JpegBackground::white()));
    }

    #[test]
    fn batch_summary_to_result_accepts_success_and_skips() {
        let summary = BatchSummary {
            total_files: 2,
            succeeded: 1,
            failed: 0,
            skipped: 1,
            total_input_size: 100,
            total_output_size: 50,
        };

        assert!(batch_summary_to_result(summary).is_ok());
    }

    #[test]
    fn batch_summary_to_result_rejects_partial_failures() {
        let summary = BatchSummary {
            total_files: 3,
            succeeded: 2,
            failed: 1,
            skipped: 0,
            total_input_size: 100,
            total_output_size: 50,
        };

        let err = batch_summary_to_result(summary).unwrap_err();
        assert!(matches!(
            err,
            ConverterError::BatchPartialFailure { failed: 1 }
        ));
    }
}

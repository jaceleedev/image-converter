use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::converter::{convert_image_silent_with_options, ConvertStats, ResizeOptions};
use crate::error::{ConverterError, Result};
use crate::format::OutputFormat;
use crate::utils::{format_file_size, format_quality_label};

/// 디렉토리 변환 결과 합계
pub struct BatchSummary {
    pub total_files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_input_size: u64,
    pub total_output_size: u64,
}

enum ProcessOutcome {
    Converted(ConvertStats),
    Skipped,
    Failed,
}

/// 입력 파일이 지원하는 이미지 확장자인지 확인
fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "avif" | "tiff" | "tif" | "bmp" | "ico")
    )
}

/// 입력 디렉토리의 이미지 파일을 모두 수집
fn collect_images(input_dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let walker = if recursive {
        WalkDir::new(input_dir)
    } else {
        WalkDir::new(input_dir).max_depth(1)
    };

    walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| is_supported_image(path))
        .collect()
}

/// 입력 파일 경로를 출력 디렉토리 내의 새 경로로 매핑
/// 재귀 모드에서는 입력 디렉토리 기준 상대 구조를 유지
fn map_output_path(
    input_file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    target_format: OutputFormat,
) -> PathBuf {
    let relative = input_file
        .strip_prefix(input_dir)
        .unwrap_or(input_file)
        .with_extension(target_format.as_str());
    output_dir.join(relative)
}

/// 디렉토리 내 이미지를 일괄 변환
///
/// `threads`: 명시적 스레드 수. `None` 이면 rayon default
/// (전역 풀 = 환경변수 `RAYON_NUM_THREADS` 또는 CPU 코어 수)
pub fn convert_directory(
    input_dir: &str,
    output_dir: &str,
    format: OutputFormat,
    quality: f32,
    recursive: bool,
    threads: Option<usize>,
) -> Result<BatchSummary> {
    convert_directory_with_options(
        input_dir, output_dir, format, quality, recursive, threads, None,
    )
}

/// 디렉토리 내 이미지를 일괄 변환한다. 리사이즈 같은 추가 옵션을 적용할 때 사용한다.
pub fn convert_directory_with_options(
    input_dir: &str,
    output_dir: &str,
    format: OutputFormat,
    quality: f32,
    recursive: bool,
    threads: Option<usize>,
    resize: Option<ResizeOptions>,
) -> Result<BatchSummary> {
    let input_path = Path::new(input_dir);
    let output_path = Path::new(output_dir);

    if !input_path.is_dir() {
        return Err(ConverterError::InvalidPath(format!(
            "입력 경로가 디렉토리가 아닙니다: {}",
            input_dir
        )));
    }
    std::fs::create_dir_all(output_path)?;

    println!(
        "\n{} 디렉토리 일괄 변환을 시작합니다...",
        "🚀".bright_blue()
    );
    println!(
        "  {} 입력: {}{}",
        "📂".bright_yellow(),
        input_dir.bright_cyan(),
        if recursive {
            " (재귀)".bright_magenta().to_string()
        } else {
            String::new()
        }
    );
    println!(
        "  {} 출력: {} ({})",
        "📁".bright_green(),
        output_dir.bright_cyan(),
        format.display_name().bright_magenta()
    );
    if let Some(n) = threads {
        println!(
            "  {} 스레드: {}",
            "🧵".bright_yellow(),
            n.to_string().bright_yellow()
        );
    }
    if let Some(options) = resize {
        println!(
            "  {} 최대 가로: {}px",
            "📐".bright_yellow(),
            options.max_width.to_string().bright_yellow()
        );
    }

    let files = collect_images(input_path, recursive);

    if files.is_empty() {
        println!(
            "\n{} 변환 가능한 이미지(.png/.jpg/.jpeg/.webp/.avif/.tiff/.bmp/.ico)가 없습니다.",
            "⚠️".bright_yellow()
        );
        return Ok(BatchSummary {
            total_files: 0,
            succeeded: 0,
            failed: 0,
            skipped: 0,
            total_input_size: 0,
            total_output_size: 0,
        });
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    let mut summary = BatchSummary {
        total_files: files.len(),
        succeeded: 0,
        failed: 0,
        skipped: 0,
        total_input_size: 0,
        total_output_size: 0,
    };

    // 병렬 변환 — ProgressBar/println 은 indicatif 내부 Mutex 로 thread-safe
    // `threads` 가 지정되면 local pool 을 만들어 scoped 실행, 아니면 rayon 전역 풀 사용
    let run_par = |files: &[PathBuf]| -> Vec<ProcessOutcome> {
        files
            .par_iter()
            .map(|file| process_one(file, input_path, output_path, format, quality, resize, &pb))
            .collect()
    };
    let outcomes: Vec<ProcessOutcome> = match threads {
        Some(n) => {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(n).build()?;
            pool.install(|| run_par(&files))
        }
        None => run_par(&files),
    };

    // 결과 직렬 합산
    for outcome in outcomes {
        match outcome {
            ProcessOutcome::Converted(stats) => {
                summary.succeeded += 1;
                summary.total_input_size += stats.input_size;
                summary.total_output_size += stats.output_size;
            }
            ProcessOutcome::Skipped => summary.skipped += 1,
            ProcessOutcome::Failed => summary.failed += 1,
        }
    }

    pb.finish_with_message("✅ 일괄 변환 완료!");
    print_batch_summary(&summary, format, quality);
    Ok(summary)
}

/// 단일 파일을 변환하고, 진행률 바를 1 증가시킨다. 실패 시 progressbar 위에 메시지를 찍고 None 반환
fn process_one(
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    format: OutputFormat,
    quality: f32,
    resize: Option<ResizeOptions>,
    pb: &ProgressBar,
) -> ProcessOutcome {
    let display = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let dest = map_output_path(file, input_dir, output_dir, format);
    if let Some(parent) = dest.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            pb.println(format!(
                "  {} {}: 출력 디렉토리 생성 실패 ({})",
                "❌".bright_red(),
                display,
                e
            ));
            pb.inc(1);
            return ProcessOutcome::Failed;
        }
    }

    if dest.exists() {
        pb.println(format!(
            "  {} {}: 출력 경로가 이미 있어 건너뜀 ({})",
            "⏭️".bright_yellow(),
            display,
            dest.display()
        ));
        pb.inc(1);
        return ProcessOutcome::Skipped;
    }

    let in_str = match file.to_str() {
        Some(s) => s,
        None => {
            pb.println(format!(
                "  {} {}: 입력 경로 인코딩 오류",
                "❌".bright_red(),
                display
            ));
            pb.inc(1);
            return ProcessOutcome::Failed;
        }
    };
    let out_str = match dest.to_str() {
        Some(s) => s,
        None => {
            pb.println(format!(
                "  {} {}: 출력 경로 인코딩 오류",
                "❌".bright_red(),
                display
            ));
            pb.inc(1);
            return ProcessOutcome::Failed;
        }
    };

    let result = match convert_image_silent_with_options(in_str, out_str, format, quality, resize) {
        Ok(stats) => ProcessOutcome::Converted(stats),
        Err(ConverterError::OutputExists(_)) => {
            pb.println(format!(
                "  {} {}: 출력 경로가 이미 있어 건너뜀 ({})",
                "⏭️".bright_yellow(),
                display,
                dest.display()
            ));
            ProcessOutcome::Skipped
        }
        Err(e) => {
            pb.println(format!("  {} {}: {}", "❌".bright_red(), display, e));
            ProcessOutcome::Failed
        }
    };
    pb.inc(1);
    result
}

fn print_batch_summary(summary: &BatchSummary, format: OutputFormat, quality: f32) {
    println!("\n{} 일괄 변환 결과:", "📊".bright_blue());
    println!(
        "  {} 처리 대상: {}개",
        "🗂️".bright_yellow(),
        summary.total_files.to_string().bright_yellow()
    );
    println!(
        "  {} 성공: {}개  {} 건너뜀: {}개  {} 실패: {}개",
        "✅".bright_green(),
        summary.succeeded.to_string().bright_green(),
        "⏭️".bright_yellow(),
        summary.skipped.to_string().bright_yellow(),
        "❌".bright_red(),
        summary.failed.to_string().bright_red()
    );

    if summary.succeeded > 0 {
        let reduction = if summary.total_input_size > 0 {
            ((summary.total_input_size as f64 - summary.total_output_size as f64)
                / summary.total_input_size as f64)
                * 100.0
        } else {
            0.0
        };
        println!(
            "  {} 원본 합계: {}",
            "📁".bright_yellow(),
            format_file_size(summary.total_input_size).bright_yellow()
        );
        println!(
            "  {} 변환 합계: {} ({})",
            "💾".bright_green(),
            format_file_size(summary.total_output_size).bright_green(),
            format_quality_label(format, quality)
        );
        println!(
            "  {} 평균 용량 감소: {:.1}% {}",
            if reduction > 30.0 { "🎉" } else { "✅" },
            reduction.abs(),
            if reduction > 0.0 {
                "↓".bright_green()
            } else {
                "↑".bright_red()
            }
        );
    }
}

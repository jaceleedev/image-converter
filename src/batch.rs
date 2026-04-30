use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::converter::{convert_image_silent, ConvertStats};
use crate::error::{ConverterError, Result};
use crate::utils::format_file_size;

/// 디렉토리 변환 결과 합계
pub struct BatchSummary {
    pub total_files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_input_size: u64,
    pub total_output_size: u64,
}

/// 입력 파일이 지원하는 이미지 확장자인지 확인
fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .as_deref(),
        Some(
            "png" | "jpg" | "jpeg" | "webp" | "avif" | "tiff" | "tif" | "bmp" | "ico"
        )
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
    target_format: &str,
) -> PathBuf {
    let relative = input_file
        .strip_prefix(input_dir)
        .unwrap_or(input_file)
        .with_extension(target_format);
    output_dir.join(relative)
}

/// 디렉토리 내 이미지를 일괄 변환
///
/// `threads`: 명시적 스레드 수. `None` 이면 rayon default
/// (전역 풀 = 환경변수 `RAYON_NUM_THREADS` 또는 CPU 코어 수)
pub fn convert_directory(
    input_dir: &str,
    output_dir: &str,
    format: &str,
    quality: f32,
    recursive: bool,
    threads: Option<usize>,
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
        format.to_uppercase().bright_magenta()
    );
    if let Some(n) = threads {
        println!(
            "  {} 스레드: {}",
            "🧵".bright_yellow(),
            n.to_string().bright_yellow()
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
    let run_par = |files: &[PathBuf]| -> Vec<Option<ConvertStats>> {
        files
            .par_iter()
            .map(|file| process_one(file, input_path, output_path, format, quality, &pb))
            .collect()
    };
    let outcomes: Vec<Option<ConvertStats>> = match threads {
        Some(n) => {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(n).build()?;
            pool.install(|| run_par(&files))
        }
        None => run_par(&files),
    };

    // 결과 직렬 합산
    for outcome in outcomes {
        match outcome {
            Some(stats) => {
                summary.succeeded += 1;
                summary.total_input_size += stats.input_size;
                summary.total_output_size += stats.output_size;
            }
            None => summary.failed += 1,
        }
    }

    pb.finish_with_message("✅ 일괄 변환 완료!");
    print_batch_summary(&summary, quality);
    Ok(summary)
}

/// 단일 파일을 변환하고, 진행률 바를 1 증가시킨다. 실패 시 progressbar 위에 메시지를 찍고 None 반환
fn process_one(
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    format: &str,
    quality: f32,
    pb: &ProgressBar,
) -> Option<ConvertStats> {
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
            return None;
        }
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
            return None;
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
            return None;
        }
    };

    let result = match convert_image_silent(in_str, out_str, format, quality) {
        Ok(stats) => Some(stats),
        Err(e) => {
            pb.println(format!("  {} {}: {}", "❌".bright_red(), display, e));
            None
        }
    };
    pb.inc(1);
    result
}

fn print_batch_summary(summary: &BatchSummary, quality: f32) {
    println!("\n{} 일괄 변환 결과:", "📊".bright_blue());
    println!(
        "  {} 처리 대상: {}개",
        "🗂️".bright_yellow(),
        summary.total_files.to_string().bright_yellow()
    );
    println!(
        "  {} 성공: {}개  {} 실패: {}개",
        "✅".bright_green(),
        summary.succeeded.to_string().bright_green(),
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
            "  {} 변환 합계: {} (품질: {}%)",
            "💾".bright_green(),
            format_file_size(summary.total_output_size).bright_green(),
            quality as u32
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

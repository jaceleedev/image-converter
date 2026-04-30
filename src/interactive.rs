use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::{Path, PathBuf};

use crate::batch::convert_directory;
use crate::converter::convert_image;

fn validate_input_path(input: &str, is_batch: bool) -> Result<(), &'static str> {
    let path = Path::new(input);
    if !path.exists() {
        return Err("경로가 존재하지 않습니다");
    }
    if is_batch && !path.is_dir() {
        return Err("디렉토리 경로를 입력해야 합니다");
    }
    if !is_batch && !path.is_file() {
        return Err("파일 경로를 입력해야 합니다");
    }
    Ok(())
}

fn validate_quality_input(input: &str) -> Result<(), &'static str> {
    match input.parse::<f32>() {
        Ok(q) if (1.0..=100.0).contains(&q) => Ok(()),
        _ => Err("1-100 사이의 숫자를 입력하세요"),
    }
}

fn validate_threads_input(input: &str) -> Result<(), &'static str> {
    match input.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(()),
        _ => Err("1 이상의 정수를 입력하세요"),
    }
}

fn default_output_path_for_file(input: &Path, format: &str) -> String {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    format!("{}_converted.{}", stem, format)
}

fn default_output_path_for_dir(input: &Path, format: &str) -> String {
    let dir_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    format!("{}_converted_{}", dir_name, format)
}

/// 대화형 모드로 이미지 변환
pub fn interactive_mode() -> crate::error::Result<()> {
    println!("{}", "🖼️  이미지 변환기 - 대화형 모드".bright_cyan().bold());
    println!("{}", "================================".bright_cyan());

    // 모드 선택
    let modes = vec!["단일 파일 변환", "디렉토리 일괄 변환"];
    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("변환 모드를 선택하세요")
        .items(&modes)
        .default(0)
        .interact()?;

    let is_batch = mode_selection == 1;

    // 입력 경로 입력 (모드에 따라 검증 로직 다름)
    let input_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(if is_batch {
            "변환할 디렉토리 경로를 입력하세요"
        } else {
            "변환할 이미지 파일 경로를 입력하세요"
        })
        .validate_with(|input: &String| validate_input_path(input, is_batch))
        .interact_text()?;

    // 재귀 옵션 (배치 모드만)
    let recursive = if is_batch {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("하위 폴더까지 재귀적으로 변환할까요?")
            .default(false)
            .interact()?
    } else {
        false
    };

    // 출력 형식 선택
    let formats = vec!["WebP", "AVIF", "PNG", "JPEG"];
    let format_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("출력 형식을 선택하세요")
        .items(&formats)
        .default(0)
        .interact()?;

    let format = formats[format_selection].to_lowercase();

    // 품질 선택 — PNG 는 무손실이라 의미 없으므로 스킵
    let quality = if format == "png" {
        println!(
            "  {} PNG 는 무손실 포맷이라 품질 설정이 적용되지 않습니다.",
            "ℹ️".bright_blue()
        );
        100.0
    } else {
        let quality_options = vec![
            "최고 품질 (100%)",
            "높음 (90%)",
            "보통 (80%)",
            "낮음 (70%)",
            "사용자 지정",
        ];
        let quality_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("변환 품질을 선택하세요")
            .items(&quality_options)
            .default(1)
            .interact()?;

        match quality_selection {
            0 => 100.0,
            1 => 90.0,
            2 => 80.0,
            3 => 70.0,
            _ => Input::with_theme(&ColorfulTheme::default())
                .with_prompt("품질 값을 입력하세요 (1-100)")
                .validate_with(|input: &String| validate_quality_input(input))
                .interact_text()?
                .parse::<f32>()?,
        }
    };

    // 스레드 수 (배치 모드만, 빈 입력은 None = rayon default)
    let threads: Option<usize> = if is_batch {
        let raw: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("스레드 수 (1 이상, 비워두면 모든 코어 사용)")
            .allow_empty(true)
            .validate_with(|input: &String| -> Result<(), &'static str> {
                if input.trim().is_empty() {
                    Ok(())
                } else {
                    validate_threads_input(input.trim())
                }
            })
            .interact_text()?;
        if raw.trim().is_empty() {
            None
        } else {
            Some(raw.trim().parse::<usize>().expect("validate 통과"))
        }
    } else {
        None
    };

    // 출력 경로
    let input_path_buf = PathBuf::from(&input_path);
    if is_batch {
        let default_output = default_output_path_for_dir(&input_path_buf, &format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("출력 디렉토리 경로를 입력하세요")
            .default(default_output)
            .interact_text()?;

        convert_directory(&input_path, &output_path, &format, quality, recursive, threads)?;
    } else {
        let default_output = default_output_path_for_file(&input_path_buf, &format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("출력 파일 경로를 입력하세요")
            .default(default_output)
            .interact_text()?;

        convert_image(&input_path, &output_path, &format, quality)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn validate_input_path_rejects_nonexistent() {
        let err = validate_input_path("/nonexistent/path/abc", false).unwrap_err();
        assert!(err.contains("존재하지 않습니다"));
    }

    #[test]
    fn validate_input_path_rejects_dir_in_single_mode() {
        let dir = TempDir::new().unwrap();
        let err = validate_input_path(dir.path().to_str().unwrap(), false).unwrap_err();
        assert!(err.contains("파일 경로"));
    }

    #[test]
    fn validate_input_path_rejects_file_in_batch_mode() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("a.txt");
        fs::write(&file_path, "x").unwrap();
        let err = validate_input_path(file_path.to_str().unwrap(), true).unwrap_err();
        assert!(err.contains("디렉토리 경로"));
    }

    #[test]
    fn validate_input_path_accepts_matching_kind() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("a.txt");
        fs::write(&file_path, "x").unwrap();
        assert!(validate_input_path(file_path.to_str().unwrap(), false).is_ok());
        assert!(validate_input_path(dir.path().to_str().unwrap(), true).is_ok());
    }

    #[test]
    fn validate_quality_input_accepts_valid() {
        assert!(validate_quality_input("1").is_ok());
        assert!(validate_quality_input("50.5").is_ok());
        assert!(validate_quality_input("100").is_ok());
    }

    #[test]
    fn validate_quality_input_rejects_invalid() {
        assert!(validate_quality_input("0").is_err());
        assert!(validate_quality_input("0.99").is_err());
        assert!(validate_quality_input("100.01").is_err());
        assert!(validate_quality_input("-10").is_err());
        assert!(validate_quality_input("abc").is_err());
    }

    #[test]
    fn validate_threads_input_accepts_positive() {
        assert!(validate_threads_input("1").is_ok());
        assert!(validate_threads_input("16").is_ok());
    }

    #[test]
    fn validate_threads_input_rejects_invalid() {
        assert!(validate_threads_input("0").is_err());
        assert!(validate_threads_input("-1").is_err());
        assert!(validate_threads_input("abc").is_err());
        assert!(validate_threads_input("").is_err());
    }

    #[test]
    fn default_output_path_for_file_uses_stem() {
        let path = Path::new("/tmp/photo.png");
        assert_eq!(default_output_path_for_file(path, "webp"), "photo_converted.webp");
    }

    #[test]
    fn default_output_path_for_file_handles_no_extension() {
        let path = Path::new("/tmp/no_ext");
        assert_eq!(default_output_path_for_file(path, "png"), "no_ext_converted.png");
    }

    #[test]
    fn default_output_path_for_dir_uses_dirname() {
        let path = Path::new("/tmp/photos");
        assert_eq!(default_output_path_for_dir(path, "webp"), "photos_converted_webp");
    }

    #[test]
    fn default_output_path_for_dir_handles_trailing_slash() {
        // Path::new("/tmp/photos/") 의 file_name() 도 "photos" 를 반환해야 함
        let path = Path::new("/tmp/photos/");
        assert_eq!(default_output_path_for_dir(path, "avif"), "photos_converted_avif");
    }
}

use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::{Path, PathBuf};

use crate::batch::convert_directory;
use crate::converter::{convert_image, validate_output_extension};
use crate::format::OutputFormat;

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

fn validate_output_file_path(input: &str, format: OutputFormat) -> Result<(), String> {
    validate_output_extension(input, format).map_err(|e| e.to_string())
}

fn default_output_path_for_file(input: &Path, format: OutputFormat) -> String {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let extension = format.as_str();
    let natural = input.with_file_name(format!("{stem}.{extension}"));
    if !natural.exists() {
        return natural.display().to_string();
    }

    let converted = input.with_file_name(format!("{stem}_converted.{extension}"));
    if !converted.exists() {
        return converted.display().to_string();
    }

    for index in 2.. {
        let numbered = input.with_file_name(format!("{stem}_converted_{index}.{extension}"));
        if !numbered.exists() {
            return numbered.display().to_string();
        }
    }

    unreachable!("무한한 번호 후보 중 하나는 반드시 사용 가능해야 합니다")
}

fn default_output_path_for_dir(input: &Path, format: OutputFormat) -> String {
    let dir_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    format!("{}_converted_{}", dir_name, format.as_str())
}

fn quality_options() -> [&'static str; 5] {
    [
        "웹 권장 (90%)",
        "균형 (80%)",
        "작게 저장 (70%)",
        "원본에 가깝게 (100%)",
        "직접 입력",
    ]
}

fn quality_for_selection(selection: usize) -> Option<f32> {
    match selection {
        0 => Some(90.0),
        1 => Some(80.0),
        2 => Some(70.0),
        3 => Some(100.0),
        _ => None,
    }
}

/// 대화형 모드로 이미지 변환
pub fn interactive_mode() -> crate::error::Result<()> {
    println!("{}", "🖼️  이미지 변환기 - 대화형 모드".bright_cyan().bold());
    println!("{}", "================================".bright_cyan());

    // 모드 선택
    let modes = vec!["이미지 1개 변환", "폴더 전체 변환"];
    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("무엇을 변환할까요?")
        .items(&modes)
        .default(0)
        .interact()?;

    let is_batch = mode_selection == 1;

    // 입력 경로 입력 (모드에 따라 검증 로직 다름)
    let input_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(if is_batch {
            "이미지가 들어 있는 폴더 경로"
        } else {
            "변환할 이미지 파일 경로"
        })
        .validate_with(|input: &String| validate_input_path(input, is_batch))
        .interact_text()?;

    // 재귀 옵션 (배치 모드만)
    let recursive = if is_batch {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("하위 폴더까지 포함할까요?")
            .default(false)
            .interact()?
    } else {
        false
    };

    // 출력 형식 선택
    let formats = [
        ("WebP - 웹 권장", OutputFormat::Webp),
        ("AVIF - 더 작은 용량", OutputFormat::Avif),
        ("PNG - 무손실", OutputFormat::Png),
        ("JPEG - 사진 호환성", OutputFormat::Jpeg),
    ];
    let format_labels: Vec<&str> = formats.iter().map(|(label, _)| *label).collect();
    let format_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("어떤 형식으로 저장할까요?")
        .items(&format_labels)
        .default(0)
        .interact()?;

    let format = formats[format_selection].1;

    // 품질 선택 — PNG 는 무손실이라 의미 없으므로 스킵
    let quality = if format.is_png() {
        println!(
            "  {} PNG 는 무손실 포맷이라 품질 단계를 건너뜁니다.",
            "ℹ️".bright_blue()
        );
        100.0
    } else {
        let quality_options = quality_options();
        let quality_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("품질을 선택하세요")
            .items(&quality_options)
            .default(0)
            .interact()?;

        match quality_for_selection(quality_selection) {
            Some(quality) => quality,
            None => Input::with_theme(&ColorfulTheme::default())
                .with_prompt("품질 값 (1-100)")
                .validate_with(|input: &String| validate_quality_input(input))
                .interact_text()?
                .parse::<f32>()?,
        }
    };

    // 스레드 수 (배치 모드만, 빈 입력은 None = rayon default)
    let threads: Option<usize> = if is_batch {
        let raw: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("동시 변환 스레드 수 (비워두면 자동)")
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
        let default_output = default_output_path_for_dir(&input_path_buf, format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("저장할 폴더 경로")
            .default(default_output)
            .interact_text()?;

        convert_directory(
            &input_path,
            &output_path,
            format,
            quality,
            recursive,
            threads,
        )?;
    } else {
        let default_output = default_output_path_for_file(&input_path_buf, format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("저장할 파일 경로")
            .default(default_output)
            .validate_with(|input: &String| validate_output_file_path(input, format))
            .interact_text()?;

        convert_image(&input_path, &output_path, format, quality)?;
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
    fn quality_options_put_web_recommended_first() {
        let options = quality_options();
        assert_eq!(options[0], "웹 권장 (90%)");
        assert_eq!(options[4], "직접 입력");
    }

    #[test]
    fn quality_for_selection_maps_presets() {
        assert_eq!(quality_for_selection(0), Some(90.0));
        assert_eq!(quality_for_selection(1), Some(80.0));
        assert_eq!(quality_for_selection(2), Some(70.0));
        assert_eq!(quality_for_selection(3), Some(100.0));
        assert_eq!(quality_for_selection(4), None);
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
    fn validate_output_file_path_accepts_matching_extension() {
        assert!(validate_output_file_path("photo.webp", OutputFormat::Webp).is_ok());
        assert!(validate_output_file_path("photo.AVIF", OutputFormat::Avif).is_ok());
        assert!(validate_output_file_path("photo.png", OutputFormat::Png).is_ok());
    }

    #[test]
    fn validate_output_file_path_accepts_jpg_jpeg_aliases() {
        assert!(validate_output_file_path("photo.jpg", OutputFormat::Jpeg).is_ok());
        assert!(validate_output_file_path("photo.jpeg", OutputFormat::Jpg).is_ok());
    }

    #[test]
    fn validate_output_file_path_rejects_mismatch_or_missing_extension() {
        let mismatch = validate_output_file_path("photo.jpg", OutputFormat::Webp).unwrap_err();
        assert!(mismatch.contains("허용: .webp"));

        let missing = validate_output_file_path("photo", OutputFormat::Png).unwrap_err();
        assert!(missing.contains("현재: 없음"));
    }

    #[test]
    fn default_output_path_for_file_uses_natural_extension_swap() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("photo.png");
        fs::write(&path, "x").unwrap();

        assert_eq!(
            default_output_path_for_file(&path, OutputFormat::Webp),
            dir.path().join("photo.webp").display().to_string()
        );
    }

    #[test]
    fn default_output_path_for_file_keeps_input_directory() {
        let dir = TempDir::new().unwrap();
        let asset_dir = dir.path().join("assets");
        fs::create_dir(&asset_dir).unwrap();
        let path = asset_dir.join("photo.png");
        fs::write(&path, "x").unwrap();

        assert_eq!(
            default_output_path_for_file(&path, OutputFormat::Avif),
            asset_dir.join("photo.avif").display().to_string()
        );
    }

    #[test]
    fn default_output_path_for_file_uses_converted_when_target_exists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("photo.png");
        fs::write(&path, "x").unwrap();
        fs::write(dir.path().join("photo.webp"), "x").unwrap();

        assert_eq!(
            default_output_path_for_file(&path, OutputFormat::Webp),
            dir.path()
                .join("photo_converted.webp")
                .display()
                .to_string()
        );
    }

    #[test]
    fn default_output_path_for_file_numbers_repeated_collisions() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("photo.png");
        fs::write(&path, "x").unwrap();
        fs::write(dir.path().join("photo.webp"), "x").unwrap();
        fs::write(dir.path().join("photo_converted.webp"), "x").unwrap();

        assert_eq!(
            default_output_path_for_file(&path, OutputFormat::Webp),
            dir.path()
                .join("photo_converted_2.webp")
                .display()
                .to_string()
        );
    }

    #[test]
    fn default_output_path_for_file_handles_no_extension() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("no_ext");
        fs::write(&path, "x").unwrap();

        assert_eq!(
            default_output_path_for_file(&path, OutputFormat::Png),
            dir.path().join("no_ext.png").display().to_string()
        );
    }

    #[test]
    fn default_output_path_for_dir_uses_dirname() {
        let path = Path::new("/tmp/photos");
        assert_eq!(
            default_output_path_for_dir(path, OutputFormat::Webp),
            "photos_converted_webp"
        );
    }

    #[test]
    fn default_output_path_for_dir_handles_trailing_slash() {
        // Path::new("/tmp/photos/") 의 file_name() 도 "photos" 를 반환해야 함
        let path = Path::new("/tmp/photos/");
        assert_eq!(
            default_output_path_for_dir(path, OutputFormat::Avif),
            "photos_converted_avif"
        );
    }
}

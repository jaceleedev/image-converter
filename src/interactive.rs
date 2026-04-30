use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::{Path, PathBuf};

use crate::batch::convert_directory;
use crate::converter::convert_image;

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
        .validate_with(|input: &String| -> Result<(), &str> {
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
        })
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
                .validate_with(|input: &String| -> Result<(), &str> {
                    match input.parse::<f32>() {
                        Ok(q) if q >= 1.0 && q <= 100.0 => Ok(()),
                        _ => Err("1-100 사이의 숫자를 입력하세요"),
                    }
                })
                .interact_text()?
                .parse::<f32>()?,
        }
    };

    // 출력 경로
    if is_batch {
        let input_path_buf = PathBuf::from(&input_path);
        let dir_name = input_path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output");
        let default_output = format!("{}_converted_{}", dir_name, format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("출력 디렉토리 경로를 입력하세요")
            .default(default_output)
            .interact_text()?;

        convert_directory(&input_path, &output_path, &format, quality, recursive)?;
    } else {
        let input_path_buf = PathBuf::from(&input_path);
        let file_stem = input_path_buf.file_stem().unwrap().to_str().unwrap();
        let default_output = format!("{}_converted.{}", file_stem, format);

        let output_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("출력 파일 경로를 입력하세요")
            .default(default_output)
            .interact_text()?;

        convert_image(&input_path, &output_path, &format, quality)?;
    }

    Ok(())
}

use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::{Path, PathBuf};

use crate::converter::convert_image;

/// 대화형 모드로 이미지 변환
pub fn interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🖼️  이미지 변환기 - 대화형 모드".bright_cyan().bold());
    println!("{}", "================================".bright_cyan());

    // 입력 파일 선택
    let input_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("변환할 이미지 파일 경로를 입력하세요")
        .validate_with(|input: &String| -> Result<(), &str> {
            if Path::new(input).exists() {
                Ok(())
            } else {
                Err("파일이 존재하지 않습니다")
            }
        })
        .interact_text()?;

    // 출력 형식 선택
    let formats = vec!["WebP", "AVIF"];
    let format_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("출력 형식을 선택하세요")
        .items(&formats)
        .default(0)
        .interact()?;

    let format = formats[format_selection].to_lowercase();

    // 품질 선택
    let quality_options = vec!["최고 품질 (100%)", "높음 (90%)", "보통 (80%)", "낮음 (70%)", "사용자 지정"];
    let quality_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("변환 품질을 선택하세요")
        .items(&quality_options)
        .default(1)
        .interact()?;

    let quality = match quality_selection {
        0 => 100.0,
        1 => 90.0,
        2 => 80.0,
        3 => 70.0,
        _ => {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("품질 값을 입력하세요 (1-100)")
                .validate_with(|input: &String| -> Result<(), &str> {
                    match input.parse::<f32>() {
                        Ok(q) if q >= 1.0 && q <= 100.0 => Ok(()),
                        _ => Err("1-100 사이의 숫자를 입력하세요"),
                    }
                })
                .interact_text()?
                .parse::<f32>()?
        }
    };

    // 출력 파일명 생성
    let input_path_buf = PathBuf::from(&input_path);
    let file_stem = input_path_buf.file_stem().unwrap().to_str().unwrap();
    let default_output = format!("{}_converted.{}", file_stem, format);

    let output_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("출력 파일 경로를 입력하세요")
        .default(default_output)
        .interact_text()?;

    // 변환 실행
    convert_image(&input_path, &output_path, &format, quality)?;

    Ok(())
}
use clap::{App, Arg};
use colored::*;
use std::path::Path;

use image_converter::{convert_directory, convert_image, interactive::interactive_mode};

fn main() {
    let matches = App::new("Image Converter")
        .version("2.1")
        .about("PNG, JPG, JPEG 이미지를 WebP와 AVIF로 변환합니다 (단일 파일 + 디렉토리 일괄)")
        .arg(
            Arg::new("interactive")
                .short('I')
                .long("interactive")
                .help("대화형 모드로 실행")
                .takes_value(false),
        )
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("PATH")
                .help("변환할 입력 이미지 파일 또는 디렉토리 경로")
                .required_unless_present("interactive")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("출력 파일 또는 디렉토리 경로")
                .required_unless_present("interactive")
                .takes_value(true),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("출력 포맷 (webp 또는 avif)")
                .required_unless_present("interactive")
                .takes_value(true),
        )
        .arg(
            Arg::new("quality")
                .short('q')
                .long("quality")
                .value_name("QUALITY")
                .help("변환 품질 (1-100, 기본값: 90)")
                .default_value("90")
                .takes_value(true),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("디렉토리 입력 시 하위 폴더까지 재귀 변환")
                .takes_value(false),
        )
        .get_matches();

    let result = if matches.is_present("interactive") {
        interactive_mode()
    } else {
        let input_path = matches.value_of("input").unwrap();
        let output_path = matches.value_of("output").unwrap();
        let format = matches.value_of("format").unwrap();
        let quality: f32 = matches
            .value_of("quality")
            .unwrap()
            .parse()
            .expect("품질은 1-100 사이 숫자여야 합니다");
        let recursive = matches.is_present("recursive");

        if Path::new(input_path).is_dir() {
            convert_directory(input_path, output_path, format, quality, recursive).map(|_| ())
        } else {
            convert_image(input_path, output_path, format, quality)
        }
    };

    if let Err(e) = result {
        eprintln!("{} 오류: {}", "❌".bright_red(), e);
        std::process::exit(1);
    }
}

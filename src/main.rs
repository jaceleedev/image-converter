use clap::{App, Arg};
use colored::*;

mod converter;
mod interactive;
mod utils;

use converter::convert_image;
use interactive::interactive_mode;

fn main() {
    let matches = App::new("Image Converter")
        .version("2.0")
        .about("PNG, JPG, JPEG 이미지를 WebP와 AVIF로 변환합니다")
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
                .value_name("FILE")
                .help("변환할 입력 이미지 파일 경로")
                .required_unless_present("interactive")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("출력 파일 경로")
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

        convert_image(input_path, output_path, format, quality)
    };

    if let Err(e) = result {
        eprintln!("{} 오류: {}", "❌".bright_red(), e);
        std::process::exit(1);
    }
}
use crate::tests::test_utils::create_test_image;
use crate::{convert_directory, convert_image_silent, ConverterError, OutputFormat};
use crate::{test_description, test_step, test_success};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_webp_conversion() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("WebP 변환 기능 전체 테스트");
    test_step!("PNG → WebP 변환이 정상적으로 작동하는지 확인");

    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("test.png");
    let output_path = temp_dir.path().join("test.webp");

    // 100x100 테스트 이미지 생성
    test_step!("100x100 테스트 이미지 생성 중...");
    create_test_image(input_path.to_str().unwrap(), 100, 100)?;
    test_success!("테스트 이미지 생성 완료");

    // WebP로 변환 (품질 90%)
    test_step!("WebP로 변환 중 (품질: 90%)...");
    convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        OutputFormat::Webp,
        90.0,
    )?;
    test_success!("변환 완료");

    // 출력 파일이 생성되었는지 확인
    assert!(output_path.exists(), "WebP 파일이 생성되어야 함");
    test_success!("출력 파일 확인");

    // 파일 크기가 감소했는지 확인
    let input_size = fs::metadata(&input_path)?.len();
    let output_size = fs::metadata(&output_path)?.len();
    assert!(output_size < input_size, "WebP 파일이 PNG보다 작아야 함");

    let reduction = ((input_size as f64 - output_size as f64) / input_size as f64) * 100.0;
    test_success!("파일 크기 {:.1}% 감소 확인", reduction);

    Ok(())
}

#[test]
fn test_avif_conversion() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("AVIF 변환 기능 전체 테스트");
    test_step!("PNG → AVIF 변환이 정상적으로 작동하는지 확인");

    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("test.png");
    let output_path = temp_dir.path().join("test.avif");

    // 100x100 테스트 이미지 생성
    test_step!("100x100 테스트 이미지 생성 중...");
    create_test_image(input_path.to_str().unwrap(), 100, 100)?;
    test_success!("테스트 이미지 생성 완료");

    // AVIF로 변환 (품질 90%)
    test_step!("AVIF로 변환 중 (품질: 90%)... (시간이 걸릴 수 있습니다)");
    convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        OutputFormat::Avif,
        90.0,
    )?;
    test_success!("변환 완료");

    // 출력 파일이 생성되었는지 확인
    assert!(output_path.exists(), "AVIF 파일이 생성되어야 함");
    test_success!("출력 파일 확인");

    // 파일 크기가 감소했는지 확인
    let input_size = fs::metadata(&input_path)?.len();
    let output_size = fs::metadata(&output_path)?.len();
    assert!(output_size < input_size, "AVIF 파일이 PNG보다 작아야 함");

    let reduction = ((input_size as f64 - output_size as f64) / input_size as f64) * 100.0;
    test_success!("파일 크기 {:.1}% 감소 확인", reduction);

    Ok(())
}

#[test]
fn test_quality_parameter_bounds() {
    test_description!("품질 파라미터 경계값 테스트");
    test_step!("최소값(1)과 최대값(100)에서 정상 작동하는지 확인");

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");
    let output_min_path = temp_dir.path().join("test_min.webp");
    let output_max_path = temp_dir.path().join("test_max.webp");

    // 50x50 테스트 이미지 생성
    test_step!("50x50 테스트 이미지 생성 중...");
    create_test_image(input_path.to_str().unwrap(), 50, 50).unwrap();
    test_success!("테스트 이미지 생성 완료");

    // 최소 품질(1%) 테스트
    test_step!("최소 품질(1%) 테스트 중...");
    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_min_path.to_str().unwrap(),
        OutputFormat::Webp,
        1.0,
    );
    assert!(result.is_ok(), "품질 1%로도 변환이 가능해야 함");
    test_success!("최소 품질 변환 성공");

    // 최대 품질(100%) 테스트
    test_step!("최대 품질(100%) 테스트 중...");
    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_max_path.to_str().unwrap(),
        OutputFormat::Webp,
        100.0,
    );
    assert!(result.is_ok(), "품질 100%로도 변환이 가능해야 함");
    test_success!("최대 품질 변환 성공");
}

#[test]
fn test_invalid_format() {
    test_description!("지원하지 않는 형식 에러 처리 테스트");
    test_step!("올바른 에러 메시지가 반환되는지 확인");

    // 지원하지 않는 형식(xyz) 파싱 시도
    test_step!("지원하지 않는 형식(xyz) 파싱 시도 중...");
    let result = "xyz".parse::<OutputFormat>();

    // 에러가 발생해야 함
    assert!(result.is_err(), "지원하지 않는 형식은 에러를 반환해야 함");
    test_success!("예상대로 에러 발생");

    // 에러 메시지 확인 (포맷명까지 포함되어야 함)
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("지원하지 않는 포맷입니다") && err_msg.contains("xyz"),
        "에러 메시지가 예상과 다름: {}",
        err_msg
    );
    test_success!("올바른 에러 메시지 확인 ({})", err_msg);
}

#[test]
fn test_nonexistent_input_file() {
    test_description!("존재하지 않는 입력 파일 처리 테스트");
    test_step!("파일이 없을 때 적절한 에러가 발생하는지 확인");

    // 존재하지 않는 파일로 변환 시도
    test_step!("존재하지 않는 파일로 변환 시도 중...");
    let result = convert_image_silent(
        "/nonexistent/file.png",
        "output.webp",
        OutputFormat::Webp,
        90.0,
    );

    // 에러가 발생해야 함
    assert!(result.is_err(), "존재하지 않는 파일은 에러를 반환해야 함");
    test_success!("예상대로 에러 발생");
}

#[test]
fn test_single_conversion_rejects_existing_output() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("단일 변환 출력 덮어쓰기 방지 테스트");
    test_step!("출력 파일이 이미 있으면 변환을 중단하고 기존 파일을 보존하는지 확인");

    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input.png");
    let output_path = temp_dir.path().join("output.webp");
    create_test_image(input_path.to_str().unwrap(), 40, 40)?;
    fs::write(&output_path, b"keep me")?;

    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        OutputFormat::Webp,
        90.0,
    );

    assert!(
        matches!(result, Err(ConverterError::OutputExists(_))),
        "기존 출력 파일은 OutputExists 에러를 반환해야 함"
    );
    assert_eq!(fs::read(&output_path)?, b"keep me");
    test_success!("기존 출력 파일 보존 확인");

    Ok(())
}

#[test]
fn test_batch_directory_conversion() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("디렉토리 일괄 변환 테스트");
    test_step!("여러 PNG 파일이 모두 WebP로 변환되는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&input_dir)?;

    test_step!("3개의 테스트 이미지 생성 중...");
    for i in 0..3 {
        let path = input_dir.join(format!("image_{}.png", i));
        create_test_image(path.to_str().unwrap(), 60, 60)?;
    }
    test_success!("테스트 이미지 3개 생성 완료");

    test_step!("일괄 변환 실행 (재귀 X)...");
    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
        false,
        None,
    )?;

    assert_eq!(summary.total_files, 3, "3개 파일이 처리되어야 함");
    assert_eq!(summary.succeeded, 3, "3개 모두 성공해야 함");
    assert_eq!(summary.failed, 0, "실패는 없어야 함");
    test_success!("3개 모두 성공 확인");

    for i in 0..3 {
        let expected = output_dir.join(format!("image_{}.webp", i));
        assert!(expected.exists(), "{} 가 존재해야 함", expected.display());
    }
    test_success!("출력 파일 3개 모두 생성 확인");

    Ok(())
}

#[test]
fn test_batch_skips_existing_output_files() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("일괄 변환 출력 덮어쓰기 방지 테스트");
    test_step!("이미 존재하는 출력 파일은 건너뛰고 기존 파일을 보존하는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&input_dir)?;
    fs::create_dir(&output_dir)?;

    create_test_image(input_dir.join("photo.png").to_str().unwrap(), 50, 50)?;
    let existing_output = output_dir.join("photo.webp");
    fs::write(&existing_output, b"already here")?;

    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        80.0,
        false,
        None,
    )?;

    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.failed, 0);
    assert_eq!(fs::read(&existing_output)?, b"already here");
    test_success!("기존 출력 파일 건너뜀 + 보존 확인");

    Ok(())
}

#[test]
fn test_batch_skips_non_image_files() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("일괄 변환 시 비이미지 파일 스킵 테스트");
    test_step!(".txt 같은 비이미지 파일은 무시되는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&input_dir)?;

    test_step!("이미지 1개 + 텍스트 파일 1개 생성 중...");
    create_test_image(input_dir.join("photo.png").to_str().unwrap(), 50, 50)?;
    fs::write(input_dir.join("notes.txt"), "스킵되어야 함")?;
    fs::write(input_dir.join("readme.md"), "이것도 스킵")?;
    test_success!("혼합 파일 생성 완료");

    test_step!("일괄 변환 실행...");
    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        80.0,
        false,
        None,
    )?;

    assert_eq!(summary.total_files, 1, "이미지 1개만 처리되어야 함");
    assert_eq!(summary.succeeded, 1, "1개 성공");
    assert!(
        output_dir.join("photo.webp").exists(),
        "WebP 파일 생성 확인"
    );
    assert!(
        !output_dir.join("notes.txt").exists(),
        "텍스트 파일은 복사되면 안 됨"
    );
    test_success!("비이미지 파일 스킵 확인");

    Ok(())
}

#[test]
fn test_batch_recursive_conversion() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("재귀 모드 일괄 변환 테스트");
    test_step!("하위 디렉토리 구조가 출력에 미러링되는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    let sub_dir = input_dir.join("sub");
    fs::create_dir_all(&sub_dir)?;

    test_step!("루트 + 서브 디렉토리에 이미지 생성 중...");
    create_test_image(input_dir.join("root.png").to_str().unwrap(), 40, 40)?;
    create_test_image(sub_dir.join("nested.png").to_str().unwrap(), 40, 40)?;
    test_success!("이미지 2개 생성 완료");

    test_step!("재귀 모드로 일괄 변환 실행...");
    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        80.0,
        true,
        None,
    )?;

    assert_eq!(
        summary.total_files, 2,
        "재귀 모드에서 2개 파일이 처리되어야 함"
    );
    assert!(output_dir.join("root.webp").exists(), "루트 출력 확인");
    assert!(
        output_dir.join("sub").join("nested.webp").exists(),
        "서브 디렉토리 구조 미러링 확인"
    );
    test_success!("재귀 변환 + 디렉토리 구조 보존 확인");

    Ok(())
}

#[test]
fn test_batch_non_recursive_skips_subdirs() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("비재귀 모드에서 하위 디렉토리 스킵 테스트");
    test_step!("재귀 옵션 없이 실행 시 하위 디렉토리는 무시되는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    let sub_dir = input_dir.join("sub");
    fs::create_dir_all(&sub_dir)?;

    create_test_image(input_dir.join("top.png").to_str().unwrap(), 40, 40)?;
    create_test_image(sub_dir.join("nested.png").to_str().unwrap(), 40, 40)?;
    test_success!("최상위 + 하위 이미지 생성 완료");

    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        80.0,
        false,
        None,
    )?;

    assert_eq!(
        summary.total_files, 1,
        "비재귀 모드에서는 1개만 처리되어야 함"
    );
    assert!(output_dir.join("top.webp").exists());
    assert!(!output_dir.join("sub").join("nested.webp").exists());
    test_success!("하위 디렉토리 스킵 확인");

    Ok(())
}

#[test]
fn test_batch_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("이미지가 없는 디렉토리 처리 테스트");
    test_step!("빈 디렉토리에서도 에러 없이 0개 처리로 끝나는지 확인");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("empty_input");
    let output_dir = temp_dir.path().join("empty_output");
    fs::create_dir(&input_dir)?;

    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Webp,
        90.0,
        false,
        None,
    )?;

    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.succeeded, 0);
    test_success!("빈 디렉토리도 정상 처리");

    Ok(())
}

#[test]
fn test_png_output_from_webp_input() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("WebP → PNG 역변환 테스트");
    test_step!("WebP 파일을 PNG 로 되돌릴 수 있는지 확인");

    let temp_dir = TempDir::new()?;
    let png_seed = temp_dir.path().join("seed.png");
    let webp_path = temp_dir.path().join("intermediate.webp");
    let restored_path = temp_dir.path().join("restored.png");

    // 1) PNG 시드 → 2) WebP 로 변환 → 3) WebP 를 다시 PNG 로
    create_test_image(png_seed.to_str().unwrap(), 80, 80)?;
    convert_image_silent(
        png_seed.to_str().unwrap(),
        webp_path.to_str().unwrap(),
        OutputFormat::Webp,
        90.0,
    )?;
    test_success!("WebP 중간 파일 생성");

    convert_image_silent(
        webp_path.to_str().unwrap(),
        restored_path.to_str().unwrap(),
        OutputFormat::Png,
        90.0,
    )?;

    assert!(restored_path.exists(), "복원된 PNG 파일이 생성되어야 함");

    // PNG 시그니처(0x89 P N G) 검증
    let bytes = fs::read(&restored_path)?;
    assert_eq!(
        &bytes[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "PNG 매직 바이트 확인"
    );
    test_success!("WebP → PNG 역변환 + 시그니처 확인");

    Ok(())
}

#[test]
fn test_jpeg_output_from_png_input() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("PNG → JPEG 변환 테스트");
    test_step!("PNG 입력을 JPEG 로 변환할 수 있는지 확인");

    let temp_dir = TempDir::new()?;
    let png_path = temp_dir.path().join("test.png");
    let jpg_path = temp_dir.path().join("test.jpg");

    create_test_image(png_path.to_str().unwrap(), 100, 100)?;

    convert_image_silent(
        png_path.to_str().unwrap(),
        jpg_path.to_str().unwrap(),
        OutputFormat::Jpeg,
        85.0,
    )?;

    assert!(jpg_path.exists(), "JPEG 파일이 생성되어야 함");

    // JPEG 시그니처(FF D8 FF) 검증
    let bytes = fs::read(&jpg_path)?;
    assert_eq!(&bytes[0..3], &[0xFF, 0xD8, 0xFF], "JPEG 매직 바이트 확인");
    test_success!("PNG → JPEG 변환 + 시그니처 확인");

    Ok(())
}

#[test]
fn test_jpg_alias_for_jpeg() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("'jpg' 가 'jpeg' 와 동일하게 처리되는지 테스트");

    let temp_dir = TempDir::new()?;
    let png_path = temp_dir.path().join("test.png");
    let jpg_path = temp_dir.path().join("test.jpg");

    create_test_image(png_path.to_str().unwrap(), 60, 60)?;

    convert_image_silent(
        png_path.to_str().unwrap(),
        jpg_path.to_str().unwrap(),
        OutputFormat::Jpg,
        85.0,
    )?;

    assert!(jpg_path.exists());
    let bytes = fs::read(&jpg_path)?;
    assert_eq!(&bytes[0..3], &[0xFF, 0xD8, 0xFF]);
    test_success!("'jpg' 별칭 정상 동작");

    Ok(())
}

#[test]
fn test_tiff_input_to_webp() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("TIFF 입력을 WebP 로 변환");

    let temp_dir = TempDir::new()?;
    let tiff_path = temp_dir.path().join("test.tiff");
    let webp_path = temp_dir.path().join("test.webp");

    // ImageBuffer::save 가 확장자로 포맷 추론 → TIFF 로 저장됨
    create_test_image(tiff_path.to_str().unwrap(), 80, 80)?;

    convert_image_silent(
        tiff_path.to_str().unwrap(),
        webp_path.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
    )?;
    assert!(webp_path.exists());
    test_success!("TIFF → WebP 변환 성공");

    Ok(())
}

#[test]
fn test_bmp_input_to_jpeg() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("BMP 입력을 JPEG 로 변환");

    let temp_dir = TempDir::new()?;
    let bmp_path = temp_dir.path().join("test.bmp");
    let jpg_path = temp_dir.path().join("test.jpg");

    create_test_image(bmp_path.to_str().unwrap(), 60, 60)?;

    convert_image_silent(
        bmp_path.to_str().unwrap(),
        jpg_path.to_str().unwrap(),
        OutputFormat::Jpeg,
        85.0,
    )?;
    assert!(jpg_path.exists());
    let bytes = fs::read(&jpg_path)?;
    assert_eq!(&bytes[0..3], &[0xFF, 0xD8, 0xFF]);
    test_success!("BMP → JPEG 변환 성공");

    Ok(())
}

#[test]
fn test_batch_mixed_input_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("일괄 변환에서 다양한 입력 포맷 혼합 테스트");
    test_step!("PNG + WebP + AVIF + TIFF + BMP 가 한 디렉토리에 있을 때 모두 PNG 로 변환되는지");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("mixed_input");
    let output_dir = temp_dir.path().join("mixed_output");
    fs::create_dir(&input_dir)?;

    // PNG, TIFF, BMP 는 ImageBuffer::save 가 직접 만들 수 있음
    create_test_image(input_dir.join("a.png").to_str().unwrap(), 50, 50)?;
    create_test_image(input_dir.join("b.tiff").to_str().unwrap(), 50, 50)?;
    create_test_image(input_dir.join("c.bmp").to_str().unwrap(), 50, 50)?;

    // WebP / AVIF 는 별도 인코더라 PNG 시드를 거쳐 생성
    let temp_png = temp_dir.path().join("temp_seed.png");
    create_test_image(temp_png.to_str().unwrap(), 50, 50)?;
    convert_image_silent(
        temp_png.to_str().unwrap(),
        input_dir.join("d.webp").to_str().unwrap(),
        OutputFormat::Webp,
        90.0,
    )?;
    convert_image_silent(
        temp_png.to_str().unwrap(),
        input_dir.join("e.avif").to_str().unwrap(),
        OutputFormat::Avif,
        85.0,
    )?;
    test_success!("혼합 입력 5개 (PNG/TIFF/BMP/WebP/AVIF) 준비 완료");

    let summary = convert_directory(
        input_dir.to_str().unwrap(),
        output_dir.to_str().unwrap(),
        OutputFormat::Png,
        100.0,
        false,
        None,
    )?;

    assert_eq!(summary.total_files, 5, "5개 파일이 처리 대상");
    assert_eq!(summary.succeeded, 5, "5개 모두 성공");
    assert!(output_dir.join("a.png").exists());
    assert!(output_dir.join("b.png").exists());
    assert!(output_dir.join("c.png").exists());
    assert!(output_dir.join("d.png").exists());
    assert!(output_dir.join("e.png").exists());
    test_success!("5종 입력 포맷 모두 PNG 로 변환 성공");

    Ok(())
}

#[test]
fn test_batch_with_explicit_threads() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("--threads 옵션으로 스레드 수 명시 테스트");
    test_step!("threads=Some(1) 과 threads=None 이 동일한 결과(개수/성공)를 내는지");

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("threads_input");
    let output_default = temp_dir.path().join("out_default");
    let output_one = temp_dir.path().join("out_one");
    fs::create_dir(&input_dir)?;

    test_step!("4개 테스트 이미지 생성...");
    for i in 0..4 {
        create_test_image(
            input_dir.join(format!("img_{}.png", i)).to_str().unwrap(),
            50,
            50,
        )?;
    }

    test_step!("threads=None (rayon default) 으로 변환...");
    let summary_default = convert_directory(
        input_dir.to_str().unwrap(),
        output_default.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
        false,
        None,
    )?;

    test_step!("threads=Some(1) (단일 스레드) 으로 변환...");
    let summary_one = convert_directory(
        input_dir.to_str().unwrap(),
        output_one.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
        false,
        Some(1),
    )?;

    assert_eq!(summary_default.total_files, 4, "기본 모드 4개");
    assert_eq!(summary_one.total_files, 4, "단일 스레드 모드 4개");
    assert_eq!(
        summary_default.succeeded, summary_one.succeeded,
        "스레드 수와 무관하게 같은 성공 개수"
    );
    test_success!("두 모드 모두 4개 변환 성공 — 결과 일관성 확인");

    Ok(())
}

#[test]
fn test_avif_input_to_png() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("AVIF → PNG 역변환 테스트");
    test_step!("AVIF 파일을 PNG 로 디코딩 (avif-decoder feature + dav1d)");

    let temp_dir = TempDir::new()?;
    let png_seed = temp_dir.path().join("seed.png");
    let avif_path = temp_dir.path().join("intermediate.avif");
    let restored_path = temp_dir.path().join("restored.png");

    // 1) PNG 시드 → 2) AVIF 인코딩 → 3) AVIF 입력을 다시 PNG 로 디코딩
    create_test_image(png_seed.to_str().unwrap(), 60, 60)?;
    convert_image_silent(
        png_seed.to_str().unwrap(),
        avif_path.to_str().unwrap(),
        OutputFormat::Avif,
        85.0,
    )?;
    test_success!("AVIF 중간 파일 생성");

    convert_image_silent(
        avif_path.to_str().unwrap(),
        restored_path.to_str().unwrap(),
        OutputFormat::Png,
        90.0,
    )?;

    assert!(restored_path.exists(), "복원된 PNG 파일이 생성되어야 함");

    // PNG 시그니처 검증
    let bytes = fs::read(&restored_path)?;
    assert_eq!(
        &bytes[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "PNG 매직 바이트 확인"
    );
    test_success!("AVIF → PNG 역변환 + 시그니처 확인");

    Ok(())
}

#[test]
fn test_jpeg_input_to_webp() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("JPEG 입력을 WebP 로 변환");
    test_step!("JPEG 디코더 + WebP 인코더 결합 검증 (단일 입력 명시 케이스)");

    let temp_dir = TempDir::new()?;
    let jpeg_path = temp_dir.path().join("seed.jpeg");
    let webp_path = temp_dir.path().join("out.webp");

    // ImageBuffer::save 가 .jpeg 확장자로 JPEG 인코더 호출
    create_test_image(jpeg_path.to_str().unwrap(), 80, 80)?;
    test_success!("JPEG 시드 파일 생성");

    convert_image_silent(
        jpeg_path.to_str().unwrap(),
        webp_path.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
    )?;

    assert!(webp_path.exists(), "WebP 파일이 생성되어야 함");

    // WebP 시그니처 검증 (RIFF...WEBP)
    let bytes = fs::read(&webp_path)?;
    assert_eq!(&bytes[0..4], b"RIFF", "RIFF 매직 바이트 확인");
    assert_eq!(&bytes[8..12], b"WEBP", "WEBP 시그니처 확인");
    test_success!("JPEG → WebP 변환 + 시그니처 확인");

    Ok(())
}

#[test]
fn test_jpeg_input_to_png() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("JPEG 입력을 PNG 로 변환");
    test_step!("JPEG 디코더 + PNG 인코더 결합 검증");

    let temp_dir = TempDir::new()?;
    let jpeg_path = temp_dir.path().join("seed.jpeg");
    let png_path = temp_dir.path().join("out.png");

    create_test_image(jpeg_path.to_str().unwrap(), 70, 70)?;

    convert_image_silent(
        jpeg_path.to_str().unwrap(),
        png_path.to_str().unwrap(),
        OutputFormat::Png,
        100.0,
    )?;

    assert!(png_path.exists(), "PNG 파일이 생성되어야 함");

    let bytes = fs::read(&png_path)?;
    assert_eq!(
        &bytes[0..4],
        &[0x89, 0x50, 0x4E, 0x47],
        "PNG 매직 바이트 확인"
    );
    test_success!("JPEG → PNG 변환 + 시그니처 확인");

    Ok(())
}

#[test]
fn test_jpg_extension_input() -> Result<(), Box<dyn std::error::Error>> {
    test_description!("'.jpg' 확장자 입력이 JPEG 디코더로 인식되는지 테스트");
    test_step!("입력 측 jpg/jpeg 별칭 동작 확인");

    let temp_dir = TempDir::new()?;
    let jpg_path = temp_dir.path().join("seed.jpg");
    let webp_path = temp_dir.path().join("out.webp");

    // .jpg 확장자도 JPEG 인코더로 저장됨
    create_test_image(jpg_path.to_str().unwrap(), 60, 60)?;
    test_success!("'.jpg' 시드 파일 생성");

    convert_image_silent(
        jpg_path.to_str().unwrap(),
        webp_path.to_str().unwrap(),
        OutputFormat::Webp,
        85.0,
    )?;

    assert!(webp_path.exists(), "'.jpg' 입력도 정상 디코딩되어야 함");
    test_success!("'.jpg' 확장자 입력 디코딩 성공");

    Ok(())
}

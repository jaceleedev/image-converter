use crate::{test_description, test_step, test_success};
use crate::tests::test_utils::create_test_image;
use crate::convert_image_silent;
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
        "webp",
        90.0
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
        "avif",
        90.0
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
    let output_path = temp_dir.path().join("test.webp");
    
    // 50x50 테스트 이미지 생성
    test_step!("50x50 테스트 이미지 생성 중...");
    create_test_image(input_path.to_str().unwrap(), 50, 50).unwrap();
    test_success!("테스트 이미지 생성 완료");
    
    // 최소 품질(1%) 테스트
    test_step!("최소 품질(1%) 테스트 중...");
    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        "webp",
        1.0
    );
    assert!(result.is_ok(), "품질 1%로도 변환이 가능해야 함");
    test_success!("최소 품질 변환 성공");
    
    // 최대 품질(100%) 테스트
    test_step!("최대 품질(100%) 테스트 중...");
    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        "webp",
        100.0
    );
    assert!(result.is_ok(), "품질 100%로도 변환이 가능해야 함");
    test_success!("최대 품질 변환 성공");
}

#[test]
fn test_invalid_format() {
    test_description!("지원하지 않는 형식 에러 처리 테스트");
    test_step!("올바른 에러 메시지가 반환되는지 확인");
    
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.png");
    let output_path = temp_dir.path().join("test.xyz");
    
    // 테스트 이미지 생성
    test_step!("테스트 이미지 생성 중...");
    create_test_image(input_path.to_str().unwrap(), 50, 50).unwrap();
    test_success!("테스트 이미지 생성 완료");
    
    // 지원하지 않는 형식(xyz)으로 변환 시도
    test_step!("지원하지 않는 형식(xyz)으로 변환 시도 중...");
    let result = convert_image_silent(
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap(),
        "xyz",
        90.0
    );
    
    // 에러가 발생해야 함
    assert!(result.is_err(), "지원하지 않는 형식은 에러를 반환해야 함");
    test_success!("예상대로 에러 발생");
    
    // 에러 메시지 확인
    assert_eq!(result.unwrap_err().to_string(), "지원하지 않는 포맷입니다");
    test_success!("올바른 에러 메시지 확인");
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
        "webp",
        90.0
    );
    
    // 에러가 발생해야 함
    assert!(result.is_err(), "존재하지 않는 파일은 에러를 반환해야 함");
    test_success!("예상대로 에러 발생");
}
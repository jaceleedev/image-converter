use crate::format_file_size;
use crate::{test_description, test_step, test_success};

#[test]
fn test_format_file_size() {
    test_description!("파일 크기 포맷팅 함수 테스트");
    test_step!("다양한 크기를 사람이 읽기 쉬운 형식으로 변환하는지 확인");

    // 0 바이트 처리
    assert_eq!(format_file_size(0), "0.00 B");
    test_success!("0 바이트 처리 완료");

    // 1KB 미만
    assert_eq!(format_file_size(512), "512.00 B");
    test_success!("512 바이트 → 512.00 B");

    // 정확히 1KB
    assert_eq!(format_file_size(1024), "1.00 KB");
    test_success!("1024 바이트 → 1.00 KB");

    // KB 단위 소수점 표시
    assert_eq!(format_file_size(1536), "1.50 KB");
    test_success!("1536 바이트 → 1.50 KB");

    // 정확히 1MB
    assert_eq!(format_file_size(1048576), "1.00 MB");
    test_success!("1048576 바이트 → 1.00 MB");

    // 정확히 1GB
    assert_eq!(format_file_size(1073741824), "1.00 GB");
    test_success!("1073741824 바이트 → 1.00 GB");
}

#[test]
fn test_format_file_size_edge_cases() {
    test_description!("파일 크기 변환 함수의 경계값 테스트");
    test_step!("단위가 바뀌는 경계에서 올바르게 동작하는지 확인");

    // 1KB 직전 (1023 bytes)
    assert_eq!(format_file_size(1023), "1023.00 B");
    test_success!("1KB 직전 (1023 bytes) 처리 완료");

    // 1MB 직전
    assert_eq!(format_file_size(1024 * 1024 - 1), "1024.00 KB");
    test_success!("1MB 직전 처리 완료");

    // 1GB 직전
    assert_eq!(format_file_size(1024 * 1024 * 1024 - 1), "1024.00 MB");
    test_success!("1GB 직전 처리 완료");
}

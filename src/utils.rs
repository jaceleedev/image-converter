use crate::format::OutputFormat;

/// 파일 크기를 사람이 읽기 쉬운 형식으로 변환하는 함수
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// 출력 요약에 표시할 품질/무손실 라벨
pub fn format_quality_label(format: OutputFormat, quality: f32) -> String {
    if format.is_png() {
        "무손실".to_string()
    } else {
        format!("품질: {}%", quality as u32)
    }
}

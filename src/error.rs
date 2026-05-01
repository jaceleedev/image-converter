use thiserror::Error;

/// 이미지 변환 과정에서 발생할 수 있는 모든 에러
#[derive(Debug, Error)]
pub enum ConverterError {
    #[error("입출력 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("이미지 디코딩 오류: {0}")]
    Image(#[from] image::ImageError),

    #[error("WebP 인코딩 오류: {0}")]
    Webp(String),

    #[error("AVIF 인코딩 오류: {0}")]
    Avif(#[from] ravif::Error),

    #[error("지원하지 않는 포맷입니다: {0}")]
    UnsupportedFormat(String),

    #[error("경로 오류: {0}")]
    InvalidPath(String),

    #[error("출력 경로가 이미 존재합니다: {0}")]
    OutputExists(String),

    #[error("대화형 입력 오류: {0}")]
    Dialog(#[from] dialoguer::Error),

    #[error("품질 값 파싱 오류: {0}")]
    QualityParse(#[from] std::num::ParseFloatError),

    #[error("스레드 풀 생성 오류: {0}")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),
}

/// 라이브러리 전역에서 사용하는 Result 별칭
pub type Result<T> = std::result::Result<T, ConverterError>;

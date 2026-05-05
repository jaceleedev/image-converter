use std::sync::Once;

static REGISTER_EXTRA_DECODERS: Once = Once::new();

pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "avif", "heic", "heif", "tiff", "tif", "bmp", "ico",
];

/// 확장자가 변환 입력으로 지원되는 이미지 형식인지 확인한다.
pub fn is_supported_input_extension(extension: &str) -> bool {
    SUPPORTED_INPUT_EXTENSIONS
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
}

/// 사용자 메시지에 넣을 지원 입력 확장자 목록을 반환한다.
pub fn supported_input_extensions_label() -> String {
    SUPPORTED_INPUT_EXTENSIONS
        .iter()
        .map(|extension| format!(".{extension}"))
        .collect::<Vec<_>>()
        .join("/")
}

/// image 크레이트 기본 디코더 밖의 입력 포맷을 등록한다.
pub fn register_extra_decoders() {
    REGISTER_EXTRA_DECODERS.call_once(|| {
        libheif_rs::integration::image::register_heic_decoding_hook();
        libheif_rs::integration::image::register_heif_decoding_hook();
    });
}

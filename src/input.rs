use std::sync::Once;

static REGISTER_EXTRA_DECODERS: Once = Once::new();

/// image 크레이트 기본 디코더 밖의 입력 포맷을 등록한다.
pub fn register_extra_decoders() {
    REGISTER_EXTRA_DECODERS.call_once(|| {
        libheif_rs::integration::image::register_heic_decoding_hook();
        libheif_rs::integration::image::register_heif_decoding_hook();
    });
}

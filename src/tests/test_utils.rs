use image::{ImageBuffer, Rgb};

/// 테스트용 체커보드 패턴 이미지를 생성하는 헬퍼 함수
pub fn create_test_image(path: &str, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255u8, 255u8, 255u8])  // 흰색
        } else {
            Rgb([0u8, 0u8, 0u8])         // 검은색
        }
    });
    
    img.save(path)?;
    Ok(())
}

/// 테스트 시작 시 설명을 출력하는 매크로
#[macro_export]
macro_rules! test_description {
    ($($arg:tt)*) => {
        println!("\n🧪 {}", format!($($arg)*));
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    };
}

/// 테스트 단계를 표시하는 매크로
#[macro_export]
macro_rules! test_step {
    ($($arg:tt)*) => {
        println!("  → {}", format!($($arg)*));
    };
}

/// 테스트 성공을 표시하는 매크로
#[macro_export]
macro_rules! test_success {
    ($($arg:tt)*) => {
        println!("  ✓ {}", format!($($arg)*));
    };
}
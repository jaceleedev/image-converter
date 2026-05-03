#[cfg(unix)]
mod unix {
    use image::{GenericImageView, ImageBuffer, Rgb, Rgba, RgbaImage};
    use rexpect::session::{spawn_command, PtySession};
    use std::error::Error;
    use std::fs;
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    fn create_test_image(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
        let img = ImageBuffer::from_fn(32, 32, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255u8, 255u8, 255u8])
            } else {
                Rgb([0u8, 0u8, 0u8])
            }
        });
        img.save(path)?;
        Ok(())
    }

    fn pty_text(text: &str) -> String {
        text.as_bytes().iter().map(|byte| *byte as char).collect()
    }

    fn expect_text(session: &mut PtySession, text: &str) -> Result<(), rexpect::error::Error> {
        session.exp_string(&pty_text(text)).map(|_| ())
    }

    fn choose_select(
        session: &mut PtySession,
        down_count: usize,
    ) -> Result<(), rexpect::error::Error> {
        if down_count == 0 {
            session.send_line("").map(|_| ())
        } else {
            session.send_line(&"\x1b[B".repeat(down_count)).map(|_| ())
        }
    }

    #[test]
    fn interactive_default_single_file_flow_converts_to_webp() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_path = temp_dir.path().join("sample.png");
        let output_path = temp_dir.path().join("sample.webp");
        create_test_image(&input_path)?;

        let mut command = Command::new(env!("CARGO_BIN_EXE_image_converter"));
        command.env("NO_COLOR", "1");
        let mut session = spawn_command(command, Some(20_000))?;

        expect_text(&mut session, "이미지 변환기 - 대화형 모드")?;
        expect_text(&mut session, "무엇을 변환할까요?")?;
        session.send_line("")?;

        expect_text(&mut session, "변환할 이미지 파일 경로")?;
        session.send_line(input_path.to_str().expect("테스트 경로는 UTF-8"))?;

        expect_text(&mut session, "어떤 형식으로 저장할까요?")?;
        session.send_line("")?;

        expect_text(&mut session, "품질을 선택하세요")?;
        session.send_line("")?;

        expect_text(&mut session, "가로 크기를 줄일까요?")?;
        session.send_line("")?;

        expect_text(&mut session, "저장할 파일 경로")?;
        session.send_line("")?;

        expect_text(&mut session, "변환 완료")?;
        session.exp_eof()?;

        assert!(output_path.exists(), "기본 출력 WebP 파일이 생성되어야 함");
        let output = std::fs::read(&output_path)?;
        assert!(
            output.starts_with(b"RIFF") && output.get(8..12) == Some(b"WEBP"),
            "출력 파일은 WebP 시그니처를 가져야 함"
        );

        Ok(())
    }

    #[test]
    fn interactive_batch_flow_applies_resize() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&input_dir)?;
        create_test_image(&input_dir.join("first.png"))?;
        create_test_image(&input_dir.join("second.png"))?;

        let mut command = Command::new(env!("CARGO_BIN_EXE_image_converter"));
        command.env("NO_COLOR", "1");
        let mut session = spawn_command(command, Some(20_000))?;

        expect_text(&mut session, "무엇을 변환할까요?")?;
        choose_select(&mut session, 1)?;

        expect_text(&mut session, "이미지가 들어 있는 폴더 경로")?;
        session.send_line(input_dir.to_str().expect("테스트 경로는 UTF-8"))?;

        expect_text(&mut session, "하위 폴더까지 포함할까요?")?;
        session.send_line("")?;

        expect_text(&mut session, "어떤 형식으로 저장할까요?")?;
        choose_select(&mut session, 2)?;

        expect_text(&mut session, "가로 크기를 줄일까요?")?;
        session.send_line("y")?;

        expect_text(&mut session, "최대 가로 크기")?;
        session.send_line("16")?;

        expect_text(&mut session, "동시 변환 스레드 수")?;
        session.send_line("")?;

        expect_text(&mut session, "저장할 폴더 경로")?;
        session.send_line(output_dir.to_str().expect("테스트 경로는 UTF-8"))?;

        expect_text(&mut session, "일괄 변환 완료")?;
        session.exp_eof()?;

        assert_eq!(
            image::open(output_dir.join("first.png"))?.dimensions(),
            (16, 16)
        );
        assert_eq!(
            image::open(output_dir.join("second.png"))?.dimensions(),
            (16, 16)
        );

        Ok(())
    }

    #[test]
    fn interactive_jpeg_custom_background_flow_flattens_transparency() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = TempDir::new()?;
        let input_path = temp_dir.path().join("transparent.png");
        let output_path = temp_dir.path().join("transparent.jpeg");
        let mut rgba = RgbaImage::new(16, 16);
        for pixel in rgba.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 0]);
        }
        rgba.save(&input_path)?;

        let mut command = Command::new(env!("CARGO_BIN_EXE_image_converter"));
        command.env("NO_COLOR", "1");
        let mut session = spawn_command(command, Some(20_000))?;

        expect_text(&mut session, "무엇을 변환할까요?")?;
        choose_select(&mut session, 0)?;

        expect_text(&mut session, "변환할 이미지 파일 경로")?;
        session.send_line(input_path.to_str().expect("테스트 경로는 UTF-8"))?;

        expect_text(&mut session, "어떤 형식으로 저장할까요?")?;
        choose_select(&mut session, 3)?;

        expect_text(&mut session, "품질을 선택하세요")?;
        choose_select(&mut session, 0)?;

        expect_text(&mut session, "투명 영역 배경색")?;
        choose_select(&mut session, 2)?;

        expect_text(&mut session, "배경색")?;
        session.send_line("#000000")?;

        expect_text(&mut session, "가로 크기를 줄일까요?")?;
        session.send_line("")?;

        expect_text(&mut session, "저장할 파일 경로")?;
        session.send_line("")?;

        expect_text(&mut session, "변환 완료")?;
        session.exp_eof()?;

        let output = image::open(&output_path)?.to_rgb8();
        let pixel = output.get_pixel(0, 0);
        assert!(
            pixel[0] < 15 && pixel[1] < 15 && pixel[2] < 15,
            "투명 영역이 직접 입력한 검정 배경에 가깝게 합성되어야 함: {:?}",
            pixel
        );

        Ok(())
    }

    #[test]
    fn non_interactive_cli_applies_max_width() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_path = temp_dir.path().join("wide.png");
        let output_path = temp_dir.path().join("wide.png.out.png");
        create_test_image(&input_path)?;

        let status = Command::new(env!("CARGO_BIN_EXE_image_converter"))
            .args([
                "-i",
                input_path.to_str().expect("테스트 경로는 UTF-8"),
                "-o",
                output_path.to_str().expect("테스트 경로는 UTF-8"),
                "-f",
                "png",
                "--max-width",
                "16",
            ])
            .env("NO_COLOR", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        assert!(status.success(), "CLI 변환이 성공해야 함");
        let output = image::open(&output_path)?;
        assert_eq!(output.dimensions(), (16, 16));

        Ok(())
    }

    #[test]
    fn non_interactive_cli_applies_jpeg_background() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_path = temp_dir.path().join("transparent.png");
        let output_path = temp_dir.path().join("transparent.jpg");
        let mut rgba = RgbaImage::new(16, 16);
        for pixel in rgba.pixels_mut() {
            *pixel = Rgba([255, 0, 0, 0]);
        }
        rgba.save(&input_path)?;

        let status = Command::new(env!("CARGO_BIN_EXE_image_converter"))
            .args([
                "-i",
                input_path.to_str().expect("테스트 경로는 UTF-8"),
                "-o",
                output_path.to_str().expect("테스트 경로는 UTF-8"),
                "-f",
                "jpeg",
                "-q",
                "100",
                "--jpeg-background",
                "black",
            ])
            .env("NO_COLOR", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        assert!(status.success(), "CLI 변환이 성공해야 함");
        let output = image::open(&output_path)?.to_rgb8();
        let pixel = output.get_pixel(0, 0);
        assert!(
            pixel[0] < 15 && pixel[1] < 15 && pixel[2] < 15,
            "투명 영역이 검정 배경에 가깝게 합성되어야 함: {:?}",
            pixel
        );

        Ok(())
    }

    #[test]
    fn non_interactive_batch_exits_nonzero_when_any_file_fails() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&input_dir)?;
        create_test_image(&input_dir.join("good.png"))?;
        fs::write(input_dir.join("broken.png"), b"not really a png")?;

        let status = Command::new(env!("CARGO_BIN_EXE_image_converter"))
            .args([
                "-i",
                input_dir.to_str().expect("테스트 경로는 UTF-8"),
                "-o",
                output_dir.to_str().expect("테스트 경로는 UTF-8"),
                "-f",
                "webp",
            ])
            .env("NO_COLOR", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        assert!(
            !status.success(),
            "손상 파일이 포함된 배치 변환은 non-zero 로 종료해야 함"
        );
        assert!(
            output_dir.join("good.webp").exists(),
            "정상 파일 출력은 생성되어야 함"
        );
        assert!(
            !output_dir.join("broken.webp").exists(),
            "손상 파일 출력은 생성되면 안 됨"
        );

        Ok(())
    }

    #[test]
    fn non_interactive_batch_skipped_outputs_still_succeed() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&input_dir)?;
        fs::create_dir(&output_dir)?;
        create_test_image(&input_dir.join("photo.png"))?;
        let existing_output = output_dir.join("photo.webp");
        fs::write(&existing_output, b"already converted")?;

        let status = Command::new(env!("CARGO_BIN_EXE_image_converter"))
            .args([
                "-i",
                input_dir.to_str().expect("테스트 경로는 UTF-8"),
                "-o",
                output_dir.to_str().expect("테스트 경로는 UTF-8"),
                "-f",
                "webp",
            ])
            .env("NO_COLOR", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        assert!(
            status.success(),
            "건너뜀만 있는 배치 변환은 성공으로 종료해야 함"
        );
        assert_eq!(fs::read(&existing_output)?, b"already converted");

        Ok(())
    }
}

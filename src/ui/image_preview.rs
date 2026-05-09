use std::path::Path;

/// Verifica se o terminal suporta o protocolo Kitty Graphics
pub fn is_kitty_supported() -> bool {
    // Dentro do tmux, verifica se kitten está disponível
    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        // Verifica se kitten icat está disponível
        return std::process::Command::new("kitten")
            .arg("--version")
            .output()
            .is_ok();
    }

    // Fora do tmux, verifica variáveis do Kitty
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM")
            .map(|t| t.contains("kitty"))
            .unwrap_or(false)
        || std::env::var("TERM_PROGRAM")
            .map(|t| t.to_lowercase().contains("kitty"))
            .unwrap_or(false)
}

/// Verifica se o arquivo existe e é uma imagem suportada
pub fn is_valid_image(path: &str) -> bool {
    let p = Path::new(path);
    if !p.exists() {
        return false;
    }
    matches!(
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp")
    )
}

/// Renderiza a imagem via protocolo Kitty diretamente no stdout
/// Suspende a TUI, exibe a imagem e aguarda Enter para voltar
pub fn show_kitty_preview(path: &str) -> std::io::Result<()> {
    use crossterm::{
        cursor::MoveTo,
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    };
    use std::io::Write;

    disable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    stdout.flush()?;

    // Usa kitten icat — funciona dentro do tmux
    let status = std::process::Command::new("kitten")
        .args(["icat", "--align", "left", path])
        .status();

    match status {
        Ok(_) => {}
        Err(_) => {
            // Fallback: tenta o protocolo direto se kitten não estiver disponível
            write!(
                stdout,
                "  kitten não encontrado. Use O para abrir externamente.\n"
            )?;
            stdout.flush()?;
        }
    }

    write!(stdout, "\r\n  Pressione Enter ou Esc para voltar...")?;
    stdout.flush()?;

    enable_raw_mode()?;

    loop {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Enter
                | crossterm::event::KeyCode::Esc
                | crossterm::event::KeyCode::Char('q') => break,
                _ => {}
            },
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    enable_raw_mode()?;

    Ok(())
}

/// Detecta o formato da imagem pelo magic bytes
/// Retorna o código de formato do protocolo Kitty:
/// 32 = RGBA, 24 = RGB, 100 = PNG
fn detect_format(data: &[u8]) -> u8 {
    if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
        return 100; // PNG
    }
    if data.len() >= 3 && &data[0..3] == b"\xff\xd8\xff" {
        return 100; // JPEG — Kitty aceita como 100 também
    }
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return 100; // GIF
    }
    100 // fallback
}

/// Abre o arquivo com o visualizador padrão do sistema
pub fn open_with_system(path: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

/// Encoder base64 simples sem dependência externa
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };

        result.push(CHARS[(b0 >> 2)] as char);
        result.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        result.push(if chunk.len() > 1 {
            CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[b2 & 0x3f] as char
        } else {
            '='
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_valid_image_png() {
        // Cria um arquivo temporário .png
        let path = "/tmp/rataria_test.png";
        std::fs::write(path, b"fake png data").unwrap();
        assert!(is_valid_image(path));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_is_valid_image_jpg() {
        let path = "/tmp/rataria_test.jpg";
        std::fs::write(path, b"fake jpg data").unwrap();
        assert!(is_valid_image(path));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_is_valid_image_extensao_invalida() {
        let path = "/tmp/rataria_test.txt";
        std::fs::write(path, b"not an image").unwrap();
        assert!(!is_valid_image(path));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_is_valid_image_arquivo_inexistente() {
        assert!(!is_valid_image("/tmp/nao_existe_rataria_xyz.png"));
    }

    #[test]
    fn test_is_valid_image_webp() {
        let path = "/tmp/rataria_test.webp";
        std::fs::write(path, b"fake webp").unwrap();
        assert!(is_valid_image(path));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_base64_encode_vazio() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_um_byte() {
        // 'M' = 0x4D = 0100 1101
        // base64("M") = "TQ=="
        assert_eq!(base64_encode(b"M"), "TQ==");
    }

    #[test]
    fn test_base64_encode_dois_bytes() {
        // "Ma" -> "TWE="
        assert_eq!(base64_encode(b"Ma"), "TWE=");
    }

    #[test]
    fn test_base64_encode_tres_bytes() {
        // "Man" -> "TWFu"
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn test_base64_encode_string_conhecida() {
        // "Hello" -> "SGVsbG8="
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn test_is_kitty_supported_retorna_bool() {
        // Apenas verifica que a função roda sem panic
        let _ = is_kitty_supported();
    }

    #[test]
    fn test_detect_format_png() {
        let png_header = b"\x89PNG\r\n\x1a\n rest of data";
        assert_eq!(detect_format(png_header), 100);
    }

    #[test]
    fn test_detect_format_jpeg() {
        let jpg_header = b"\xff\xd8\xff rest of data";
        assert_eq!(detect_format(jpg_header), 100);
    }

    #[test]
    fn test_detect_format_fallback() {
        let unknown = b"unknown format";
        assert_eq!(detect_format(unknown), 100);
    }
}

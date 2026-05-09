use std::path::Path;

/// Verifica se o terminal suporta o protocolo Kitty Graphics
pub fn is_kitty_supported() -> bool {
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
    use std::io::Write;

    let data = std::fs::read(path)?;
    let encoded = base64_encode(&data);

    // Suspende a TUI
    crossterm::terminal::disable_raw_mode()?;
    let mut stdout = std::io::stdout();

    // Limpa a tela
    write!(stdout, "\x1b[2J\x1b[H")?;

    // Protocolo Kitty: transmite a imagem em chunks de 4096 bytes
    let chunk_size = 4096;
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        let more = if is_last { 0 } else { 1 };

        if i == 0 {
            // Primeiro chunk: inclui os parâmetros (formato PNG/auto, ação display)
            write!(stdout, "\x1b_Ga=T,f=100,m={};{}\x1b\\", more, chunk)?;
        } else {
            write!(stdout, "\x1b_Gm={};{}\x1b\\", more, chunk)?;
        }
    }

    stdout.flush()?;

    // Mensagem e aguarda Enter
    write!(stdout, "\n\n  Pressione Enter para voltar...")?;
    stdout.flush()?;

    // Aguarda Enter
    crossterm::terminal::enable_raw_mode()?;
    loop {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.code == crossterm::event::KeyCode::Enter
                || key.code == crossterm::event::KeyCode::Esc
            {
                break;
            }
        }
    }

    // Limpa a imagem do terminal Kitty
    let mut stdout = std::io::stdout();
    write!(stdout, "\x1b_Ga=d\x1b\\")?;
    stdout.flush()?;

    Ok(())
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
}

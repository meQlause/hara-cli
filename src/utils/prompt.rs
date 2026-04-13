use std::io::{self, BufRead, Write};

pub fn ask_reset(dirs: &[&str]) -> bool {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    ask_reset_internal(dirs, &mut stdin, &mut stdout)
}

pub fn ask_reset_internal<R: BufRead, W: Write>(
    dirs: &[&str],
    reader: &mut R,
    writer: &mut W,
) -> bool {
    writeln!(writer, "⚠️  WARNING: You are about to RESET the following directories:").unwrap();
    for dir in dirs {
        writeln!(writer, "   - {}/", dir).unwrap();
    }
    writeln!(writer, "   All files inside these directories will be PERMANENTLY DELETED.").unwrap();

    write!(writer, "Do you want to reset? (y/N): ").unwrap();
    let _ = writer.flush();

    let mut input = String::new();
    if reader.read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_ask_reset_yes() {
        let mut input = Cursor::new("y\n");
        let mut output = Vec::new();
        let result = ask_reset_internal(&["src"], &mut input, &mut output);
        assert!(result);
    }

    #[test]
    fn test_ask_reset_no() {
        let mut input = Cursor::new("n\n");
        let mut output = Vec::new();
        let result = ask_reset_internal(&["src"], &mut input, &mut output);
        assert!(!result);
    }

    #[test]
    fn test_ask_reset_default() {
        let mut input = Cursor::new("\n");
        let mut output = Vec::new();
        let result = ask_reset_internal(&["src"], &mut input, &mut output);
        assert!(!result);
    }
}

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Processing stopped. Previously saved files remain changed.");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    print!("Enter folder path (or exit): ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    let input = input.trim();
    if input.is_empty() || input.eq_ignore_ascii_case("exit") {
        return Ok(());
    }

    // Accept paths copied with surrounding double quotes.
    let folder = Path::new(input.trim_matches('"'));
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(folder)
        .map_err(|e| format!("Cannot open folder '{}': {e}", folder.display()))?;

    // Collect regular XML files only; do not enter subdirectories.
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read folder entry: {e}"))?;
        let path = entry.path();
        let file_type = entry.file_type()
            .map_err(|e| format!("Cannot inspect '{}': {e}", path.display()))?;
        let is_xml = path.extension().and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("xml"));
        if file_type.is_file() && is_xml {
            files.push(path);
        }
    }

    // Sort alphabetically, ignoring case. Exact spelling breaks ties.
    files.sort_by_cached_key(|path| {
        (path.file_name().unwrap_or_default().to_string_lossy().to_lowercase(), path.clone())
    });
    if files.is_empty() {
        return Err("No XML files found in the selected folder.".into());
    }

    // Read, change and save each file before processing the next one.
    for (index, path) in files.iter().enumerate() {
        let mut text = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read '{}': {e}", path.display()))?;
        if !replace_number(&mut text, index + 1) {
            println!("Skipped (no >digits< pattern): {}", path.display());
            continue;
        }
        fs::write(path, text.as_bytes())
            .map_err(|e| format!("Cannot save '{}': {e}", path.display()))?;
        println!("Saved: {} -> {}", path.display(), index + 1);
    }

    println!("Done. {} XML file(s) processed.", files.len());
    Ok(())
}

/// Replace the first >digits< pattern, for example >6< or >123<.
/// Return false if no matching pattern exists. All other bytes stay unchanged.
fn replace_number(text: &mut String, number: usize) -> bool {
    let bytes = text.as_bytes();

    for (position, byte) in bytes.iter().enumerate() {
        if *byte != b'>' {
            continue;
        }

        let start = position + 1;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }

        // Require at least one digit followed immediately by '<'.
        if end > start && bytes.get(end) == Some(&b'<') {
            text.replace_range(start..end, &number.to_string());
            return true;
        }
    }

    false
}

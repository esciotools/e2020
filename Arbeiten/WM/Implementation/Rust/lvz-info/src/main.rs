use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
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

    let folder = Path::new(input.trim_matches('"'));
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(folder)
        .map_err(|e| format!("Cannot open folder '{}': {e}", folder.display()))?;

    // Collect regular XML files only. Ignore subdirectories and symbolic links.
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot read folder entry: {e}"))?;
        let path = entry.path();
        let kind = entry.file_type()
            .map_err(|e| format!("Cannot inspect '{}': {e}", path.display()))?;
        let is_xml = path.extension().and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("xml"));
        if kind.is_file() && is_xml {
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

    for path in &files {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read '{}': {e}", path.display()))?;
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        print_anchors(&text, &filename);
    }

    println!("\nDone. {} XML file(s) analyzed. No files changed.", files.len());
    Ok(())
}

/// Inspect literal anchor="..." attributes using text searches only.
/// Associate each anchor with the number immediately after its opening tag.
/// This expects the sample format and does not validate or parse XML.
fn print_anchors(text: &str, filename: &str) {
    const MARKER: &str = "anchor=\"";
    let mut remaining = text;
    let mut found = false;

    while let Some(position) = remaining.find(MARKER) {
        // Require an attribute boundary to avoid matching names such as data-anchor.
        if position > 0 && !remaining.as_bytes()[position - 1].is_ascii_whitespace() {
            remaining = &remaining[position + MARKER.len()..];
            continue;
        }
        found = true;
        let value = &remaining[position + MARKER.len()..];
        let Some(quote) = value.find('"') else {
            println!("{filename}: [missing closing quote] -> [not found]");
            return;
        };
        let anchor = &value[..quote];
        let after_anchor = &value[quote + 1..];

        // Stay inside the current opening tag when looking for its end.
        let tag_end = after_anchor.find(['>', '<']);
        let number = tag_end.and_then(|end| {
            if after_anchor.as_bytes()[end] != b'>' {
                return None;
            }
            let content = &after_anchor[end + 1..];
            let digit_count = content.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digit_count > 0 && content.as_bytes().get(digit_count) == Some(&b'<') {
                Some(&content[..digit_count])
            } else {
                None
            }
        });

        println!("{filename}: {anchor} -> {}", number.unwrap_or("[not found]"));
        remaining = after_anchor;
    }

    if !found {
        println!("{filename}: [no anchor found] -> [not found]");
    }
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    /// Byte offset of the heading line within the source string passed to
    /// `extract_toc`.  Used to split the document into per-heading sections
    /// for scroll-to anchors.
    pub byte_offset: usize,
}

pub fn extract_toc(markdown: &str) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut in_code_block = false;
    // Track byte position by splitting on '\n' so we can record offsets.
    // split('\n') leaves '\r' at end of lines on Windows; trim() handles it.
    let mut byte_pos: usize = 0;

    for line in markdown.split('\n') {
        let line_start = byte_pos;
        // Advance past this line and its '\n' separator.
        byte_pos += line.len() + 1;

        let line = line.trim_end_matches('\r'); // tolerate \r\n

        if line.starts_with("```") || line.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let level = line.chars().take_while(|&c| c == '#').count();
        if level > 0 && level <= 6 {
            let rest = line[level..].trim();
            if !rest.is_empty() {
                entries.push(TocEntry {
                    level: level as u8,
                    title: strip_inline_markup(rest).to_string(),
                    anchor: slugify(rest),
                    byte_offset: line_start,
                });
            }
        }
    }

    entries
}

/// GitHub-compatible heading anchor generation.
pub fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut prev_dash = false;

    for c in text.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_lowercase().next().unwrap());
            prev_dash = false;
        } else if c == '-' || c == '_' {
            if !prev_dash && !result.is_empty() {
                result.push('-');
                prev_dash = true;
            }
        } else if !prev_dash && !result.is_empty() {
            result.push('-');
            prev_dash = true;
        }
    }

    result.trim_end_matches('-').to_string()
}

/// Remove `**`, `__`, `*`, `_`, `` ` `` so TOC titles are plain text.
fn strip_inline_markup(s: &str) -> &str {
    // Fast path: no markup
    s.trim_matches(|c: char| c == '#' || c == ' ')
}

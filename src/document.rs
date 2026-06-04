use std::path::Path;

use crate::outline::{extract_toc, TocEntry};

/// A key-value pair from YAML frontmatter.
#[derive(Debug, Clone)]
pub struct FrontmatterEntry {
    pub key: String,
    pub value: String,
}

pub struct Document {
    /// Raw source content (unmodified).
    pub content: String,
    /// Content with relative image paths rewritten to `file://` absolute URIs,
    /// and frontmatter stripped — this is what the viewer renders.
    pub display_content: String,
    /// Byte ranges into `display_content` for each render section.
    /// `section_ranges[0]` = pre-heading preamble (may be empty).
    /// `section_ranges[i+1]` = TOC entry `i`'s heading + body.
    pub section_ranges: Vec<(usize, usize)>,
    pub toc: Vec<TocEntry>,
    /// YAML front-matter entries, or empty if none.
    pub frontmatter: Vec<FrontmatterEntry>,
}

impl Document {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let base_path = path.parent().unwrap_or(Path::new(".")).to_path_buf();

        let (frontmatter, body) = extract_frontmatter(&raw);
        let display_content = resolve_image_paths(body, &base_path);
        let toc = extract_toc(&display_content);
        let section_ranges = build_section_ranges(&display_content, &toc);

        Ok(Self {
            content: raw,
            display_content,
            section_ranges,
            toc,
            frontmatter,
        })
    }
}

// ── Section splitting ─────────────────────────────────────────────────────────

/// Build byte-range pairs for each render section of `content`.
///
/// `ranges[0]`   = preamble before the first heading (may be `0..0`).
/// `ranges[i+1]` = heading `i` line + body up to the next heading.
///
/// All ranges are valid slices of `content` and are clamped to `content.len()`.
fn build_section_ranges(content: &str, toc: &[TocEntry]) -> Vec<(usize, usize)> {
    let total = content.len();

    if toc.is_empty() {
        return vec![(0, total)];
    }

    let mut ranges = Vec::with_capacity(toc.len() + 1);

    // Preamble: everything before the first heading
    let first = toc[0].byte_offset.min(total);
    ranges.push((0, first));

    // One range per heading
    for i in 0..toc.len() {
        let start = toc[i].byte_offset.min(total);
        let end = if i + 1 < toc.len() {
            toc[i + 1].byte_offset.min(total)
        } else {
            total
        };
        ranges.push((start, end));
    }

    ranges
}

// ── Frontmatter ───────────────────────────────────────────────────────────────

/// Returns `(entries, body_without_frontmatter)`.
/// Frontmatter is the YAML block between the opening `---` and closing `---`
/// or `...` on lines by themselves at the very start of the file.
fn extract_frontmatter(src: &str) -> (Vec<FrontmatterEntry>, &str) {
    let src = src.trim_start_matches('\u{feff}'); // strip optional UTF-8 BOM

    if !src.starts_with("---") {
        return (vec![], src);
    }

    // Find the closing delimiter on its own line
    let after_open = match src[3..].find('\n') {
        Some(nl) => &src[3 + nl + 1..], // skip `---\n`
        None => return (vec![], src),
    };

    let close_pos = after_open
        .lines()
        .scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find(|(_, line)| *line == "---" || *line == "...")
        .map(|(pos, _)| pos);

    let Some(close) = close_pos else {
        return (vec![], src);
    };

    let yaml_block = &after_open[..close];
    let body = &after_open[close..];
    // Skip the closing `---\n`
    let body = body.find('\n').map(|n| &body[n + 1..]).unwrap_or("");

    let entries = parse_simple_yaml(yaml_block);
    (entries, body)
}

/// Parse simple `key: value` lines from a YAML block.
/// Does not handle nested objects, arrays, or multi-line values.
fn parse_simple_yaml(block: &str) -> Vec<FrontmatterEntry> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once(':')?;
            Some(FrontmatterEntry {
                key: k.trim().to_string(),
                value: v.trim().trim_matches('"').trim_matches('\'').to_string(),
            })
        })
        .collect()
}

// ── Image path resolution ─────────────────────────────────────────────────────

/// Replace relative image paths with absolute `file://` URIs so egui_extras
/// can load them regardless of the process working directory.
///
/// Handles both `![alt](path)` and `![alt](path "title")`.
pub fn resolve_image_paths(src: &str, base: &Path) -> String {
    let mut out = String::with_capacity(src.len() + 64);
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for `![`
        if i + 1 < len && bytes[i] == b'!' && bytes[i + 1] == b'[' {
            // Find the closing `](`
            if let Some(bracket_close) = find_byte(bytes, i + 2, b']') {
                if bracket_close + 1 < len && bytes[bracket_close + 1] == b'(' {
                    let path_start = bracket_close + 2;
                    if let Some(paren_close) = find_closing_paren(bytes, path_start) {
                        let path_str = &src[path_start..paren_close];
                        // Separate optional title `"..."` from the path
                        let (path_part, title_part) = split_path_and_title(path_str);

                        if should_rewrite(path_part) {
                            let abs = base.join(path_part);
                            // Write everything up to and including `](`
                            out.push_str(&src[i..path_start]);
                            // Write the rewritten path
                            #[cfg(windows)]
                            {
                                // On Windows, Path::display() may use backslashes
                                out.push_str("file:///");
                                out.push_str(&abs.to_string_lossy().replace('\\', "/"));
                            }
                            #[cfg(not(windows))]
                            {
                                out.push_str("file://");
                                out.push_str(&abs.to_string_lossy());
                            }
                            if !title_part.is_empty() {
                                out.push(' ');
                                out.push_str(title_part);
                            }
                            out.push(')');
                            i = paren_close + 1;
                            continue;
                        }
                    }
                }
            }
        }
        let ch = src[i..]
            .chars()
            .next()
            .expect("index should always point at a UTF-8 character boundary");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn should_rewrite(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with("http://")
        && !path.starts_with("https://")
        && !path.starts_with("file://")
        && !path.starts_with('/')
        && !path.starts_with('#')
        && !path.starts_with("data:")
}

/// Split `path "optional title"` into `(path, "optional title")`.
fn split_path_and_title(s: &str) -> (&str, &str) {
    if let Some(space) = s.find(|c: char| c == ' ' || c == '\t') {
        (s[..space].trim(), s[space..].trim())
    } else {
        (s.trim(), "")
    }
}

fn find_byte(bytes: &[u8], from: usize, target: u8) -> Option<usize> {
    bytes[from..]
        .iter()
        .position(|&b| b == target)
        .map(|p| from + p)
}

/// Find the `)` that closes the markdown link, skipping over nested `(...)`.
fn find_closing_paren(bytes: &[u8], from: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, &b) in bytes[from..].iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Some(from + i);
                }
                depth -= 1;
            }
            b'\n' => return None, // don't cross line boundaries
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outline::extract_toc;

    #[test]
    fn image_path_resolution_preserves_utf8_text_for_toc() {
        let markdown =
            "# Chapter 04 — Error Handling\n\n![diagram](images/flow.png)\n\n> “quoted”\n";
        let display = resolve_image_paths(markdown, Path::new("/docs"));

        assert!(display.contains("# Chapter 04 — Error Handling"));
        assert!(display.contains("> “quoted”"));

        let toc = extract_toc(&display);
        assert_eq!(toc[0].title, "Chapter 04 — Error Handling");
    }
}

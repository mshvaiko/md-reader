use std::num::{NonZeroU16, NonZeroU32};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use piper_rs::Piper;
use rodio::buffer::SamplesBuffer;

pub const DEFAULT_MODEL_PATH: &str = "models/en_US-libritts_r-medium.onnx";
pub const DEFAULT_CONFIG_PATH: &str = "models/en_US-libritts_r-medium.onnx.json";
const MAX_TTS_CHARS_PER_CHUNK: usize = 700;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechOutcome {
    Finished,
    Stopped,
}

pub fn default_model_paths() -> (PathBuf, PathBuf) {
    (
        PathBuf::from(DEFAULT_MODEL_PATH),
        PathBuf::from(DEFAULT_CONFIG_PATH),
    )
}

pub fn speak_markdown(
    markdown: &str,
    model_path: &Path,
    config_path: &Path,
    stop_requested: Arc<AtomicBool>,
) -> Result<SpeechOutcome, String> {
    let text = markdown_to_speech_text(markdown);
    if text.is_empty() {
        return Err("No readable text found".to_string());
    }

    speak_text(&text, model_path, config_path, stop_requested)
}

fn speak_text(
    text: &str,
    model_path: &Path,
    config_path: &Path,
    stop_requested: Arc<AtomicBool>,
) -> Result<SpeechOutcome, String> {
    let chunks = speech_chunks(text, MAX_TTS_CHARS_PER_CHUNK);
    if chunks.is_empty() {
        return Err("No readable text found".to_string());
    }

    let mut piper = Piper::new(model_path, config_path).map_err(|e| e.to_string())?;
    let device_sink = rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
    let player = rodio::Player::connect_new(device_sink.mixer());

    for chunk in chunks {
        if stop_requested.load(Ordering::Relaxed) {
            return Ok(SpeechOutcome::Stopped);
        }

        let (samples, sample_rate) = piper
            .create(&chunk, false, None, Some(1.6), None, None)
            .map_err(|e| e.to_string())?;

        if stop_requested.load(Ordering::Relaxed) {
            return Ok(SpeechOutcome::Stopped);
        }

        let channels = NonZeroU16::new(1).expect("one audio channel is non-zero");
        let sample_rate = NonZeroU32::new(sample_rate).ok_or("Piper returned zero sample rate")?;
        let source = SamplesBuffer::new(channels, sample_rate, samples);

        player.append(source);
        while !player.empty() {
            if stop_requested.load(Ordering::Relaxed) {
                player.stop();
                return Ok(SpeechOutcome::Stopped);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    Ok(SpeechOutcome::Finished)
}

pub fn markdown_to_speech_text(markdown: &str) -> String {
    let mut out = String::new();
    let mut in_code_block = false;
    let mut in_frontmatter = false;

    for (line_idx, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();

        if line_idx == 0 && trimmed == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" || trimmed == "..." {
                in_frontmatter = false;
            }
            continue;
        }

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || trimmed.is_empty() {
            continue;
        }

        let cleaned = clean_markdown_line(trimmed);
        if !cleaned.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&cleaned);
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_markdown_line(line: &str) -> String {
    let mut s = line.trim();

    s = s.trim_start_matches('#').trim_start();
    s = s.trim_start_matches('>').trim_start();
    s = strip_list_marker(s);

    let s = remove_images_and_keep_link_text(s);
    s.chars()
        .filter(|c| !matches!(c, '*' | '_' | '`' | '~' | '<' | '>'))
        .collect::<String>()
}

fn strip_list_marker(line: &str) -> &str {
    let s = line.trim_start();
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = s.strip_prefix(marker) {
            return rest.trim_start();
        }
    }

    let Some((number, rest)) = s.split_once('.') else {
        return s;
    };
    if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) {
        return rest.trim_start();
    }

    s
}

fn remove_images_and_keep_link_text(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'!' && bytes[i + 1] == b'[' {
            if let Some(end) = markdown_link_end(bytes, i + 1) {
                i = end;
                continue;
            }
        }

        if bytes[i] == b'[' {
            if let Some((label_end, end)) = markdown_link_parts(bytes, i) {
                out.push_str(&s[i + 1..label_end]);
                i = end;
                continue;
            }
        }

        let ch = s[i..]
            .chars()
            .next()
            .expect("index should always point at a UTF-8 character boundary");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn markdown_link_end(bytes: &[u8], from_bracket: usize) -> Option<usize> {
    markdown_link_parts(bytes, from_bracket).map(|(_, end)| end)
}

fn markdown_link_parts(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let label_end = bytes[start + 1..]
        .iter()
        .position(|&b| b == b']')
        .map(|p| start + 1 + p)?;
    if label_end + 1 >= bytes.len() || bytes[label_end + 1] != b'(' {
        return None;
    }
    let target_end = bytes[label_end + 2..]
        .iter()
        .position(|&b| b == b')')
        .map(|p| label_end + 2 + p)?;
    Some((label_end, target_end + 1))
}

fn speech_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let separator_len = usize::from(!current.is_empty());
        if !current.is_empty() && current.len() + separator_len + word.len() > max_chars {
            chunks.push(std::mem::take(&mut current));
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::{markdown_to_speech_text, speech_chunks};

    #[test]
    fn converts_markdown_to_readable_speech_text() {
        let markdown = r#"---
title: Hidden
---
# Chapter 04 — Error Handling

> “Handle errors explicitly.”

- Read [the docs](https://example.com)
- ![diagram](img/flow.png)

```rust
panic!("skip code");
```
"#;

        let text = markdown_to_speech_text(markdown);

        assert_eq!(
            text,
            "Chapter 04 — Error Handling “Handle errors explicitly.” Read the docs"
        );
    }

    #[test]
    fn splits_long_speech_text_into_bounded_chunks() {
        let text = (0..300)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");

        let chunks = speech_chunks(&text, 120);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.len() <= 120));
        assert_eq!(chunks.join(" "), text);
    }
}

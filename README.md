# md-reader

A fast, lightweight Markdown reader desktop app built with **Rust** + **egui**.

## Features

- Open `.md` / `.markdown` / `.txt` files — via toolbar, drag-and-drop, or folder browser
- Full CommonMark + GFM: **tables**, **task lists**, **strikethrough**, **footnotes**
- Syntax-highlighted code blocks (via syntect)
- **Live reload** — watches the open file; updates within ~1 s when you save in any editor
- **Table of Contents** sidebar — click to jump to any heading
- **Ctrl+F search** — find text, navigate matches
- Piper text-to-speech — reads the open Markdown file aloud, with stop support
- Dark / Light theme toggle
- Zoom (Ctrl+`+` / Ctrl+`-` / Ctrl+`0`)
- Persistent state: window size, theme, zoom, last file, sidebar width

## Build & Run

```bash
# debug
cargo run -- assets/demo.md

# release (small binary)
cargo build --release
./target/release/md-reader path/to/file.md
```

### Platform dependencies (Linux)

```bash
# Debian / Ubuntu
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
     libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

## Text to Speech Models

The app looks for the default US Piper voice in this order:

1. `models/en_US-libritts_r-medium.onnx` next to the source tree
2. `/usr/share/md-reader/models/en_US-libritts_r-medium.onnx` after Debian package install

Build the Debian package with:

```bash
packaging/build-deb.sh
```

The package build downloads the default US Piper model if it is missing locally, then installs it under `/usr/share/md-reader/models` so text-to-speech works after `apt install`.

## Keyboard Shortcuts

| Action | Key |
|--------|-----|
| Open file | `Ctrl+O` |
| Search | `Ctrl+F` |
| Next match | `Enter` |
| Close search | `Esc` |
| Read aloud | toolbar button |
| Zoom in | `Ctrl++` |
| Zoom out | `Ctrl+-` |
| Reset zoom | `Ctrl+0` |
| Toggle sidebar | toolbar button |

## Project Layout

```
src/
  main.rs           entry point
  app.rs            app state, eframe::App impl
  document.rs       file loading
  outline.rs        TOC extraction
  watcher.rs        live-reload via notify
  theme.rs          visuals, fonts, spacing
  ui/
    topbar.rs       toolbar
    sidebar.rs      outline + file tree
    reading_pane.rs centered reading column
    search.rs       Ctrl+F overlay
assets/
  demo.md           sample document
```

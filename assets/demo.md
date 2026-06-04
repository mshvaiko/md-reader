# md-reader Demo

A lightweight Markdown reader built with **Rust** and [egui](https://github.com/emilk/egui).

---

## Features

- 📄 Open `.md`, `.markdown`, `.txt` files via dialog or **drag & drop**
- 📁 Browse a folder of Markdown docs in the sidebar
- 🔄 **Live reload** — edit in any external editor and the preview updates automatically
- 🗂 **Table of Contents** — click a heading in the Outline to jump to it
- 🔍 **Search** — Ctrl+F to find text in the document
- 🌙 **Dark / Light theme** toggle
- 🔎 **Zoom** — Ctrl+`+` / Ctrl+`-` / Ctrl+`0`

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Open file | `Ctrl+O` |
| Search | `Ctrl+F` |
| Next match | `Enter` |
| Close search | `Esc` |
| Zoom in | `Ctrl++` |
| Zoom out | `Ctrl+-` |
| Reset zoom | `Ctrl+0` |

---

## Code Example

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

fn main() {
    println!("{}", greet("world"));
}
```

```python
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

print([fibonacci(i) for i in range(10)])
```

---

## Task List

- [x] Basic Markdown rendering
- [x] Syntax-highlighted code blocks
- [x] Tables
- [x] Dark / light theme
- [x] Live reload via filesystem watcher
- [ ] LaTeX math (stretch goal)
- [ ] Command palette

---

## Blockquote

> "The best programs are the ones written when the programmer is supposed to be working on something else."
>
> — Melinda Varian

---

## Inline Formatting

Text can be **bold**, _italic_, ~~strikethrough~~, or `monospaced`.
You can also combine **_bold italic_** or `inline code with backticks`.

---

## Nested Lists

1. First item
   - Sub-item A
   - Sub-item B
     - Deep item
2. Second item
3. Third item

---

## Links

- [GitHub](https://github.com)
- [egui docs](https://docs.rs/egui)
- [Rust book](https://doc.rust-lang.org/book/)

---

## Footnotes

Here is a claim that needs a citation.[^1]

[^1]: This is the footnote content.

---

*End of demo document.*

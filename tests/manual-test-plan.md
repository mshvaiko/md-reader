# md-reader — Manual Test Plan

Work through each suite and record the result. Mark `[x]` for pass; on failure leave `[ ]` and write what happened in the Notes line at the end of each suite.

**Test run header**

- Build: ________   (debug / release, version/commit) 
- Platform / OS: ________   (Windows / macOS / Linux + version)
- egui version: 0.34   ·   Date: ________   ·   Tester: ________

---

## 0. Setup — build & test fixtures

- [ ] `cargo run` builds and launches without errors.
- [ ] `cargo build --release` succeeds; run the release binary at least once.
- [ ] Create a folder `qa/` containing the fixtures below.

**Fixture A — `rich.md`** (covers most rendering): paste this:

```markdown
# Rich Sample
Intro paragraph with **bold**, *italic*, ~~strike~~, `inline code`, and a [link](https://example.com).

## Lists
- bullet one
- bullet two
  - nested
1. ordered
2. ordered

## Tasks
- [x] done
- [ ] not done

## Table
| Col A | Col B |
| --- | --- |
| 1 | 2 |
| 3 | 4 |

## Code
\`\`\`rust
fn main() { println!("hello"); }
\`\`\`

## Quote & rule
> a blockquote
---

## Image
![alt](./img/test.png)

## Footnotes
Some text with a footnote.[^1]

[^1]: The footnote body.
```

- [ ] **`long.md`** — a document with ~30+ headings and several screens of text (for TOC-scroll and active-section tests). Put a unique marker line under the last heading, e.g. `LAST-HEADING-MARKER`.
- [ ] **`img/test.png`** — any small PNG placed in `qa/img/` so `rich.md`'s relative image resolves.
- [ ] **`empty.md`** — zero bytes.
- [ ] **`huge.md`** — a very large file (e.g. 1–5 MB of generated markdown) for performance.
- [ ] **`notes.txt`** — a `.txt` to confirm accepted non-`.md` extensions.
- [ ] **`not-markdown.bin`** — a binary/non-text file for the reject/error path.

---

## 1. Launch & CLI

- [ ] **No-args launch** — run with no arguments. *Expected:* opens; restores last session or shows the empty state.
- [ ] **CLI file (relative)** — from `qa/`, run `md-reader ./rich.md`. *Expected:* GUI opens with `rich.md` already rendered.
- [ ] **CLI file (absolute)** — run with an absolute path. *Expected:* opens that file.
- [ ] **CLI directory** — run `md-reader ./qa`. *Expected:* opens the folder as the tree root; auto-opens `README.md` if present (else empty state with tree populated).
- [ ] **CLI overrides last-file** — with a persisted last file, launch with a different path arg. *Expected:* the arg wins; theme/zoom/window geometry still restore.
- [ ] **Bad path** — `md-reader ./does-not-exist.md`. *Expected:* GUI launches with an inline error/empty state — **no panic/crash**.
- [ ] **`--help`** — prints short usage and exits 0.
- [ ] **`--version`** — prints version and exits 0.

Notes: ______________________________________________

---

## 2. Opening files

- [ ] **Open via dialog** — open `rich.md`. *Expected:* renders within ~1s; window shows file name/path.
- [ ] **Drag-and-drop** — drag `rich.md` onto the window. *Expected:* renders.
- [ ] **`.txt` accepted** — open `notes.txt`. *Expected:* renders as markdown/plain text.
- [ ] **Switch files** — open `long.md`, then open `rich.md`. *Expected:* content + TOC fully replaced; no leftover state from the previous doc.
- [ ] **Reject/handle non-text** — open/drop `not-markdown.bin`. *Expected:* graceful inline error, no crash.

Notes: ______________________________________________

---

## 3. Open folder & file tree

- [ ] **Open folder** — open `qa/`. *Expected:* sidebar tree lists the markdown files.
- [ ] **Click entry** — click `long.md` in the tree. *Expected:* it renders; selected item is visually indicated.
- [ ] **Nested folders** — if `qa/` has a subfolder with `.md`, tree expands/collapses correctly.
- [ ] **Non-md filtering** — confirm the tree shows markdown (and `.txt` if intended) and hides unrelated files as designed.
- [ ] **Empty folder** — open a folder with no markdown. *Expected:* tree empty / sensible message, no crash.

Notes: ______________________________________________

---

## 4. Markdown rendering (`rich.md`)

- [ ] Headings render with a clear size hierarchy.
- [ ] Bold / italic / strikethrough / inline code render correctly.
- [ ] Bulleted, nested, and ordered lists render correctly.
- [ ] Task list shows checked and unchecked boxes.
- [ ] Table renders with aligned columns and borders.
- [ ] Code block has syntax highlighting (Rust keywords colored).
- [ ] Blockquote and horizontal rule render.
- [ ] Image from `./img/test.png` displays (see suite 9 for relative-path detail).
- [ ] Footnote reference is clickable and footnote body renders at the bottom.

Notes: ______________________________________________

---

## 5. Live reload (`notify`, 300ms debounce)

- [ ] **Edit & save** — open `rich.md`, edit it in an external editor, save. *Expected:* preview updates within ~1s without re-opening.
- [ ] **Debounce** — save rapidly several times. *Expected:* updates settle without flicker storms or crashes.
- [ ] **Scroll preserved (reasonable)** — after reload, scroll position is not violently reset (acceptable if minor).
- [ ] **File deleted while open** — delete the watched file. *Expected:* inline message, no crash; recovers if recreated.
- [ ] **Folder mode reload** — with a folder open, edit the currently displayed file. *Expected:* it reloads.

Notes: ______________________________________________

---

## 6. TOC sidebar — extraction, scroll, highlight

- [ ] **Extraction** — open `long.md`. *Expected:* TOC lists all headings in order with correct nesting/indent by level.
- [ ] **Click scrolls to heading (the fix)** — click a TOC entry partway down. *Expected:* reading pane scrolls so that heading lands at the **top** of the viewport — not just a highlight change.
- [ ] **Click bottom heading** — click the last TOC entry. *Expected:* scrolls so `LAST-HEADING-MARKER` is visible at top.
- [ ] **Active-section highlight on scroll** — scroll the reading pane manually. *Expected:* the TOC highlights the heading currently in view and updates as you scroll.
- [ ] **Highlight matches click** — clicking an entry both scrolls and highlights it.
- [ ] **Footnote/ref-link doc** — open a doc using footnotes; confirm sectioned rendering didn't break footnotes (bodies still render, references still resolve).

Notes: ______________________________________________

---

## 7. Search (Ctrl+F)

- [ ] **Open search** — Ctrl+F focuses the search field.
- [ ] **Match count** — type a term present multiple times. *Expected:* correct match count shown.
- [ ] **Next / Prev** — navigate matches forward and backward; view scrolls to each match.
- [ ] **No matches** — type a nonexistent term. *Expected:* shows 0 / "no matches", no crash.
- [ ] **Case behavior** — confirm case sensitivity behaves as designed (document expected behavior).
- [ ] **Close search** — Esc (or close control) dismisses the search bar and clears highlights.

Notes: ______________________________________________

---

## 8. Theme, zoom, layout, word count

- [ ] **Theme toggle** — switch dark ↔ light. *Expected:* immediate, fully themed (text, code blocks, sidebar, scrollbars), good contrast.
- [ ] **Zoom in/out/reset** — Ctrl++ / Ctrl+- / Ctrl+0. *Expected:* text scales smoothly; reset returns to default.
- [ ] **Zoom persists across reload** — zoom, then reload file; zoom retained.
- [ ] **Reading column** — confirm content is centered with a max width ~760px and comfortable line height; resize the window wide — text stays in the centered column.
- [ ] **Word count + reading time** — toolbar shows counts; they update after a live-reload edit that changes length.

Notes: ______________________________________________

---

## 9. Links & images (if implemented)

- [ ] **External link** — click the `https://example.com` link. *Expected:* opens in the system browser; app stays put.
- [ ] **In-document anchor** — a `[text](#some-heading)` link scrolls to that heading in-pane.
- [ ] **Relative image** — `./img/test.png` displays (resolves against the file's directory, not the cwd).
- [ ] **Image after moving file** — open the same file via a different working directory (CLI from elsewhere); image still resolves.
- [ ] **Missing image** — reference a nonexistent image. *Expected:* graceful placeholder/no crash.

Notes: ______________________________________________

---

## 10. Persistence

- [ ] **Window geometry** — resize/move window, quit, relaunch. *Expected:* size & position restored.
- [ ] **Theme** restored after relaunch.
- [ ] **Zoom** restored after relaunch.
- [ ] **Sidebar width** restored after relaunch.
- [ ] **Last file/folder** reopened on no-args launch.
- [ ] **First-run** — clear persisted state (or run fresh profile). *Expected:* sane defaults, no crash.

Notes: ______________________________________________

---

## 11. Empty state & edge cases

- [ ] **Empty state** — launch fresh / close the doc. *Expected:* "drop here" prompt shown and styled.
- [ ] **`empty.md`** — open it. *Expected:* renders blank cleanly; TOC empty; word count 0; no crash.
- [ ] **Doc with no headings** — TOC shows empty/placeholder; everything else works.
- [ ] **Unicode / emoji** — a doc with non-ASCII text and emoji renders correctly.
- [ ] **Very long lines / wide table** — horizontal handling is sane (wrap or scroll), no layout break.

Notes: ______________________________________________

---

## 12. Performance (`huge.md`)

- [ ] **Cold start** — launch feels instant (subjectively < ~1s to interactive).
- [ ] **Open large file** — `huge.md` opens without freezing the UI for long; if it must take time, the UI doesn't hard-lock.
- [ ] **Idle CPU** — when sitting on a rendered doc with no interaction, CPU usage is low (confirms it's not re-parsing/re-rendering every frame).
- [ ] **Scroll smoothness** — scrolling `huge.md` stays smooth.
- [ ] **Release binary size** — note the stripped release binary size: ________.

Notes: ______________________________________________

---

## 13. Cross-platform (run the suites on each target you ship)

- [ ] Windows: launch, open dialog, drag-drop, links open in browser.
- [ ] macOS: launch, dialog, drag-drop; (if implemented) double-clicking a `.md` in Finder opens it via the open-file event.
- [ ] Linux: launch, dialog, drag-drop; confirm required dev libs present (gtk/x11/wayland for rfd/winit).

Notes: ______________________________________________

---

## 14. Regression sweep (final pass)

After all fixes, re-verify the original shipped feature set in one quick pass:

- [ ] Open file (dialog + drag-drop)
- [ ] Open folder → tree
- [ ] Rendering (tables, tasks, code highlight, footnotes)
- [ ] Live reload
- [ ] TOC extract + **scroll** + highlight
- [ ] Search
- [ ] Theme toggle
- [ ] Zoom
- [ ] Persistence
- [ ] Centered reading column
- [ ] Word count + reading time
- [ ] Empty state
- [ ] No new console errors/warnings during a full session

Notes: ______________________________________________

---

## Defect log

| ID | Suite | Severity | Description | Repro steps | Status |
| --- | --- | --- | --- | --- | --- |
|  |  |  |  |  |  |
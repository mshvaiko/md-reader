use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::time::Instant;

use egui::Context;
use egui_commonmark::CommonMarkCache;
use serde::{Deserialize, Serialize};

use crate::cli::LaunchArg;
use crate::document::Document;
use crate::watcher::FileWatcher;

const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SidebarTab {
    FileTree,
    #[default]
    Outline,
}

/// State for the Ctrl+Shift+P command palette.
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
        }
    }
}

pub struct MdApp {
    // ── Persisted ────────────────────────────────────────────────────────────
    pub dark_mode: bool,
    pub zoom: f32,
    pub sidebar_open: bool,
    pub sidebar_width: f32,
    pub sidebar_tab: SidebarTab,
    pub last_file: Option<PathBuf>,
    pub last_folder: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,

    // ── Runtime ──────────────────────────────────────────────────────────────
    pub document: Option<Document>,
    pub folder_entries: Vec<PathBuf>,
    pub cm_cache: CommonMarkCache,

    // ── File watching ────────────────────────────────────────────────────────
    _watcher: Option<FileWatcher>,
    reload_rx: Receiver<()>,
    reload_tx: Sender<()>,
    pub last_reload: Instant,

    // ── Search ───────────────────────────────────────────────────────────────
    pub search_open: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub search_idx: usize,

    // ── TOC ──────────────────────────────────────────────────────────────────
    pub active_toc_idx: Option<usize>,
    /// Section index to scroll to on the next frame (set by TOC/anchor click).
    /// Section 0 = preamble; section i+1 = TOC entry i.
    pub pending_scroll: Option<usize>,
    /// Whether the YAML frontmatter panel is expanded.
    pub frontmatter_expanded: bool,

    // ── Command palette ───────────────────────────────────────────────────────
    pub cmd_palette: CommandPaletteState,

    // ── Text to speech ───────────────────────────────────────────────────────
    pub tts_busy: bool,
    tts_stop: Option<Arc<AtomicBool>>,
    tts_rx: Receiver<Result<crate::tts::SpeechOutcome, String>>,
    tts_tx: Sender<Result<crate::tts::SpeechOutcome, String>>,

    // ── Status bar ───────────────────────────────────────────────────────────
    pub status: Option<(String, Instant)>,
}

impl MdApp {
    pub fn new(cc: &eframe::CreationContext<'_>, launch: LaunchArg) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let s = cc.storage;
        let dark_mode: bool = s
            .and_then(|s| eframe::get_value(s, "dark_mode"))
            .unwrap_or(true);
        let zoom: f32 = s.and_then(|s| eframe::get_value(s, "zoom")).unwrap_or(1.0);
        let sidebar_open: bool = s
            .and_then(|s| eframe::get_value(s, "sidebar_open"))
            .unwrap_or(true);
        let sidebar_width: f32 = s
            .and_then(|s| eframe::get_value(s, "sidebar_width"))
            .unwrap_or(260.0);
        let sidebar_tab: SidebarTab = s
            .and_then(|s| eframe::get_value(s, "sidebar_tab"))
            .unwrap_or_default();
        let last_file: Option<PathBuf> = s.and_then(|s| eframe::get_value(s, "last_file"));
        let last_folder: Option<PathBuf> = s.and_then(|s| eframe::get_value(s, "last_folder"));
        let recent_files: Vec<PathBuf> = s
            .and_then(|s| eframe::get_value(s, "recent_files"))
            .unwrap_or_default();

        let (reload_tx, reload_rx) = mpsc::channel();
        let (tts_tx, tts_rx) = mpsc::channel();

        crate::theme::apply(&cc.egui_ctx, dark_mode, zoom);

        let mut app = Self {
            dark_mode,
            zoom,
            sidebar_open,
            sidebar_width,
            sidebar_tab,
            last_file: None,
            last_folder: None,
            recent_files,
            document: None,
            folder_entries: Vec::new(),
            cm_cache: CommonMarkCache::default(),
            _watcher: None,
            reload_rx,
            reload_tx,
            last_reload: Instant::now(),
            search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_idx: 0,
            active_toc_idx: None,
            pending_scroll: None,
            frontmatter_expanded: false,
            cmd_palette: CommandPaletteState::default(),
            tts_busy: false,
            tts_stop: None,
            tts_rx,
            tts_tx,
            status: None,
        };

        // CLI launch takes priority over persisted session.
        match launch {
            LaunchArg::File(p) => {
                app.open_file_path(p, &cc.egui_ctx);
                // still restore folder if any
                if let Some(folder) = last_folder {
                    if folder.is_dir() {
                        app.scan_folder(folder);
                    }
                }
            }
            LaunchArg::Dir(p) => {
                app.sidebar_tab = SidebarTab::FileTree;
                app.sidebar_open = true;
                // auto-open README.md if present
                let readme = ["README.md", "readme.md", "Readme.md"]
                    .iter()
                    .map(|n| p.join(n))
                    .find(|f| f.exists());
                app.scan_folder(p);
                if let Some(f) = readme {
                    app.open_file_path(f, &cc.egui_ctx);
                }
            }
            LaunchArg::Error(msg) => {
                app.set_status(msg);
                // still restore session
                if let Some(folder) = last_folder {
                    if folder.is_dir() {
                        app.scan_folder(folder);
                    }
                }
                if let Some(file) = last_file {
                    if file.exists() {
                        app.open_file_path(file, &cc.egui_ctx);
                    }
                }
            }
            LaunchArg::None => {
                if let Some(folder) = last_folder {
                    if folder.is_dir() {
                        app.scan_folder(folder);
                    }
                }
                if let Some(file) = last_file {
                    if file.exists() {
                        app.open_file_path(file, &cc.egui_ctx);
                    }
                }
            }
        }

        app
    }

    // ── File operations ───────────────────────────────────────────────────────

    pub fn open_file_dialog(&mut self, ctx: &Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .pick_file()
        {
            self.open_file_path(path, ctx);
        }
    }

    pub fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.sidebar_tab = SidebarTab::FileTree;
            self.sidebar_open = true;
            self.scan_folder(path);
        }
    }

    pub fn open_file_path(&mut self, path: PathBuf, ctx: &Context) {
        match Document::load(&path) {
            Ok(doc) => {
                self.last_file = Some(path.clone());
                self.active_toc_idx = None;
                self.pending_scroll = None;
                self.search_results.clear();
                self.cm_cache = CommonMarkCache::default();
                self.document = Some(doc);
                self.push_recent(path.clone());
                self.start_watching(path, ctx.clone());
                if self.sidebar_tab != SidebarTab::FileTree {
                    self.sidebar_tab = SidebarTab::Outline;
                }
            }
            Err(e) => self.set_status(format!("Cannot open file: {e}")),
        }
    }

    pub fn scan_folder(&mut self, path: PathBuf) {
        self.folder_entries = collect_md_files(&path, 0, 4);
        self.last_folder = Some(path);
    }

    fn start_watching(&mut self, path: PathBuf, ctx: Context) {
        let tx = self.reload_tx.clone();
        match FileWatcher::new(&path, move || {
            tx.send(()).ok();
            ctx.request_repaint();
        }) {
            Ok(w) => self._watcher = Some(w),
            Err(e) => self.set_status(format!("Watch error: {e}")),
        }
    }

    // ── Recent files ─────────────────────────────────────────────────────────

    fn push_recent(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(MAX_RECENT);
    }

    // ── Search ────────────────────────────────────────────────────────────────

    pub fn run_search(&mut self) {
        self.search_results.clear();
        self.search_idx = 0;
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        if let Some(doc) = &self.document {
            let lower = doc.content.to_lowercase();
            let mut pos = 0;
            while let Some(idx) = lower[pos..].find(&query) {
                self.search_results.push(pos + idx);
                pos += idx + query.len().max(1);
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    pub fn word_count(&self) -> usize {
        self.document
            .as_ref()
            .map(|d| d.content.split_whitespace().count())
            .unwrap_or(0)
    }

    pub fn read_aloud(&mut self) {
        if self.tts_busy {
            return;
        }

        let Some(doc) = &self.document else {
            self.set_status("Open a file before using text to speech");
            return;
        };

        let (model_path, config_path) = crate::tts::default_model_paths();
        if !model_path.exists() || !config_path.exists() {
            self.set_status(format!(
                "Missing Piper model files in {}",
                model_path
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "models".to_string())
            ));
            return;
        }

        let text = doc.content.clone();
        let tx = self.tts_tx.clone();
        let stop_requested = Arc::new(AtomicBool::new(false));
        self.tts_stop = Some(stop_requested.clone());
        self.tts_busy = true;
        self.set_status("Reading aloud...");

        std::thread::spawn(move || {
            let result =
                crate::tts::speak_markdown(&text, &model_path, &config_path, stop_requested);
            tx.send(result).ok();
        });
    }

    pub fn stop_reading_aloud(&mut self) {
        if let Some(stop_requested) = &self.tts_stop {
            stop_requested.store(true, Ordering::Relaxed);
            self.set_status("Stopping speech...");
        }
    }
}

impl eframe::App for MdApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // ── Reload ────────────────────────────────────────────────────────────
        if self.reload_rx.try_recv().is_ok() && self.last_reload.elapsed().as_millis() > 300 {
            self.last_reload = Instant::now();
            if let Some(path) = self.last_file.clone() {
                if let Ok(doc) = Document::load(&path) {
                    self.cm_cache = CommonMarkCache::default();
                    self.document = Some(doc);
                }
            }
        }

        if let Ok(result) = self.tts_rx.try_recv() {
            self.tts_busy = false;
            self.tts_stop = None;
            match result {
                Ok(crate::tts::SpeechOutcome::Finished) => {
                    self.set_status("Finished reading aloud")
                }
                Ok(crate::tts::SpeechOutcome::Stopped) => self.set_status("Stopped reading aloud"),
                Err(e) => self.set_status(format!("Text to speech error: {e}")),
            }
        }

        // ── Drag and drop ─────────────────────────────────────────────────────
        let dropped: Vec<_> = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped {
            if let Some(path) = f.path {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "md" | "markdown" | "txt") {
                    self.open_file_path(path, &ctx);
                    break;
                }
            }
        }

        // ── Keyboard shortcuts ────────────────────────────────────────────────
        let (open, search_toggle, cmd_palette, zoom_in, zoom_out, zoom_reset) = ctx.input(|i| {
            let ctrl = i.modifiers.ctrl;
            let shift = i.modifiers.shift;
            (
                ctrl && !shift && i.key_pressed(egui::Key::O),
                ctrl && !shift && i.key_pressed(egui::Key::F),
                ctrl && shift && i.key_pressed(egui::Key::P),
                ctrl && i.key_pressed(egui::Key::Equals),
                ctrl && i.key_pressed(egui::Key::Minus),
                ctrl && i.key_pressed(egui::Key::Num0),
            )
        });

        if open {
            self.open_file_dialog(&ctx);
        }
        if search_toggle {
            self.search_open = !self.search_open;
            if !self.search_open {
                self.search_results.clear();
            }
        }
        if cmd_palette {
            self.cmd_palette.open = !self.cmd_palette.open;
            if self.cmd_palette.open {
                self.cmd_palette.query.clear();
            }
        }
        if zoom_in {
            self.zoom = (self.zoom + 0.1).min(3.0);
            crate::theme::apply(&ctx, self.dark_mode, self.zoom);
        }
        if zoom_out {
            self.zoom = (self.zoom - 0.1).max(0.5);
            crate::theme::apply(&ctx, self.dark_mode, self.zoom);
        }
        if zoom_reset {
            self.zoom = 1.0;
            crate::theme::apply(&ctx, self.dark_mode, self.zoom);
        }

        // ── Expire status ─────────────────────────────────────────────────────
        if self
            .status
            .as_ref()
            .map(|(_, t)| t.elapsed().as_secs() > 4)
            .unwrap_or(false)
        {
            self.status = None;
        }

        // ── Visuals ───────────────────────────────────────────────────────────
        ctx.set_visuals(if self.dark_mode {
            crate::theme::dark_visuals()
        } else {
            crate::theme::light_visuals()
        });

        // ── Layout ────────────────────────────────────────────────────────────
        crate::ui::topbar::show(self, ui);
        if self.sidebar_open {
            crate::ui::sidebar::show(self, ui);
        }
        crate::ui::reading_pane::show(self, ui);
        if self.search_open {
            crate::ui::search::show(self, ui);
        }
        if self.cmd_palette.open {
            crate::ui::command_palette::show(self, ui);
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "dark_mode", &self.dark_mode);
        eframe::set_value(storage, "zoom", &self.zoom);
        eframe::set_value(storage, "sidebar_open", &self.sidebar_open);
        eframe::set_value(storage, "sidebar_width", &self.sidebar_width);
        eframe::set_value(storage, "sidebar_tab", &self.sidebar_tab);
        eframe::set_value(storage, "recent_files", &self.recent_files);
        if let Some(f) = &self.last_file {
            eframe::set_value(storage, "last_file", f);
        }
        if let Some(f) = &self.last_folder {
            eframe::set_value(storage, "last_folder", f);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn collect_md_files(dir: &PathBuf, depth: usize, max: usize) -> Vec<PathBuf> {
    if depth > max {
        return vec![];
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        (!is_dir, e.file_name())
    });
    let mut result = Vec::new();
    for e in entries {
        let path = e.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') {
                result.extend(collect_md_files(&path, depth + 1, max));
            }
        } else {
            let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
            if matches!(ext, "md" | "markdown" | "txt") {
                result.push(path);
            }
        }
    }
    result
}

use std::path::Path;

use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};

/// Holds a live file watcher. Drop to stop watching.
pub struct FileWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(path: &Path, on_change: impl Fn() + Send + 'static) -> notify::Result<Self> {
        let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    on_change();
                }
            }
        })?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        Ok(Self { _watcher: watcher })
    }
}

use egui::{RichText, ScrollArea};

use crate::app::{MdApp, SidebarTab};

pub fn show(app: &mut MdApp, ui: &mut egui::Ui) {
    let mut new_width = app.sidebar_width;

    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(app.sidebar_width)
        .size_range(160.0..=500.0)
        .show_inside(ui, |ui| {
            new_width = ui.available_width();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.sidebar_tab, SidebarTab::Outline, "Outline");
                ui.selectable_value(&mut app.sidebar_tab, SidebarTab::FileTree, "Files");
            });
            ui.separator();

            match app.sidebar_tab {
                SidebarTab::Outline => show_outline(app, ui),
                SidebarTab::FileTree => {
                    let ctx = ui.ctx().clone();
                    show_file_tree(app, ui, &ctx);
                }
            }
        });

    app.sidebar_width = new_width;
}

fn show_outline(app: &mut MdApp, ui: &mut egui::Ui) {
    let toc = match app.document.as_ref() {
        Some(d) if !d.toc.is_empty() => d.toc.clone(),
        Some(_) => {
            ui.label(RichText::new("No headings found").italics().small());
            return;
        }
        None => {
            ui.label(
                RichText::new("Open a file to see its outline")
                    .italics()
                    .small(),
            );
            return;
        }
    };

    let active = app.active_toc_idx;
    let link_col = ui.visuals().hyperlink_color;
    let weak_col = ui.visuals().weak_text_color();

    let mut clicked: Option<usize> = None;

    ScrollArea::vertical().id_salt("toc").show(ui, |ui| {
        ui.add_space(4.0);
        for (i, entry) in toc.iter().enumerate() {
            let indent = entry.level.saturating_sub(1) as f32 * 12.0;
            let is_active = active == Some(i);
            let size = match entry.level {
                1 => 14.0_f32,
                2 => 13.0,
                _ => 12.0,
            };
            let color = if is_active { link_col } else { weak_col };
            let text = if is_active {
                RichText::new(&entry.title).size(size).color(color).strong()
            } else {
                RichText::new(&entry.title).size(size).color(color)
            };

            let resp = ui.horizontal(|ui| {
                ui.add_space(indent + 4.0);
                ui.selectable_label(is_active, text)
            });

            if resp.inner.clicked() {
                clicked = Some(i);
            }
        }
        ui.add_space(8.0);
    });

    if let Some(idx) = clicked {
        app.active_toc_idx = Some(idx);
        // section_ranges[0] = preamble, section_ranges[i+1] = TOC entry i
        app.pending_scroll = Some(idx + 1);
        ui.ctx().request_repaint();
    }
}

fn show_file_tree(app: &mut MdApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    if app.folder_entries.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Open a folder to browse files")
                .italics()
                .small(),
        );
        ui.add_space(8.0);
        if ui.button("📁 Open folder").clicked() {
            app.open_folder_dialog();
        }
        return;
    }

    let folder_root = app.last_folder.clone();
    let current_file = app.last_file.clone();
    let entries = app.folder_entries.clone();
    let mut to_open: Option<std::path::PathBuf> = None;

    ScrollArea::vertical().id_salt("tree").show(ui, |ui| {
        ui.add_space(4.0);
        for path in &entries {
            let is_current = current_file.as_deref() == Some(path.as_path());
            let depth = folder_root
                .as_ref()
                .and_then(|r| path.strip_prefix(r).ok())
                .map(|rel| rel.components().count().saturating_sub(1))
                .unwrap_or(0);

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let color = if is_current {
                ui.visuals().hyperlink_color
            } else {
                ui.visuals().text_color()
            };

            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 14.0 + 4.0);
                if ui
                    .selectable_label(is_current, RichText::new(name).size(13.0).color(color))
                    .on_hover_text(path.to_string_lossy().as_ref())
                    .clicked()
                {
                    to_open = Some(path.clone());
                }
            });
        }
        ui.add_space(8.0);
    });

    if let Some(p) = to_open {
        app.open_file_path(p, ctx);
    }
}

use egui::{Color32, RichText, ScrollArea};
use egui_commonmark::CommonMarkViewer;

use crate::app::MdApp;

pub fn show(app: &mut MdApp, ui: &mut egui::Ui) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if app.document.is_none() {
            show_empty_state(ui, app);
            return;
        }

        // Clone only what's needed; release the immutable borrow before we
        // mutate app fields (pending_scroll, cm_cache) inside the scroll area.
        let display_content = app.document.as_ref().unwrap().display_content.clone();
        let section_ranges = app.document.as_ref().unwrap().section_ranges.clone();
        let frontmatter = app.document.as_ref().unwrap().frontmatter.clone();

        // ── Search match ribbon ───────────────────────────────────────────
        if app.search_open && !app.search_results.is_empty() {
            let n = app.search_results.len();
            let i = app.search_idx + 1;
            let q = app.search_query.clone();
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(0, 122, 204, 24))
                .inner_margin(egui::Margin::symmetric(12, 3))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("\"{}\"  {}/{} matches", q, i, n))
                            .small()
                            .color(ui.visuals().hyperlink_color),
                    );
                });
        }

        ScrollArea::vertical()
            .id_salt("reading")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(760.0);

                    // ── YAML frontmatter panel ────────────────────────────
                    if !frontmatter.is_empty() {
                        show_frontmatter(ui, app, &frontmatter);
                        ui.add_space(12.0);
                    }

                    // ── Sectioned render — one CommonMarkViewer per section ──
                    // A zero-size anchor precedes each section so we can call
                    // scroll_to_rect() inside this ScrollArea closure (the only
                    // place egui will honour the scroll request this frame).
                    for (i, &(start, end)) in section_ranges.iter().enumerate() {
                        let anchor = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
                        if app.pending_scroll == Some(i) {
                            ui.scroll_to_rect(anchor.rect, Some(egui::Align::TOP));
                            app.pending_scroll = None;
                        }
                        let section = &display_content[start..end];
                        if !section.is_empty() {
                            // Push a unique ID per section to avoid widget-id
                            // collisions when two sections happen to share content.
                            ui.push_id(i, |ui| {
                                CommonMarkViewer::new().show(ui, &mut app.cm_cache, section);
                            });
                        }
                    }
                });
                ui.add_space(48.0);
            });

        // ── Intercept link clicks ─────────────────────────────────────────
        // In egui 0.34, open_url moved into PlatformOutput::commands as
        // OutputCommand::OpenUrl.  eframe processes it after app.ui() returns,
        // so we can inspect and cancel internal (#anchor) entries here.
        let pending_url = ui.ctx().output(|o| {
            o.commands.iter().find_map(|cmd| {
                if let egui::OutputCommand::OpenUrl(u) = cmd {
                    Some(u.url.clone())
                } else {
                    None
                }
            })
        });
        if let Some(url) = pending_url {
            if url.starts_with('#') {
                // Cancel the browser-open command and handle internally.
                let cancel_url = url.clone();
                ui.ctx().output_mut(|o| {
                    o.commands.retain(|cmd| {
                        if let egui::OutputCommand::OpenUrl(u) = cmd {
                            u.url != cancel_url
                        } else {
                            true
                        }
                    });
                });
                let anchor = &url[1..];
                activate_toc_for_anchor(app, anchor);
            }
            // External URLs: leave the command for eframe to open the browser.
        }
    });
}

// ── Frontmatter panel ─────────────────────────────────────────────────────────

fn show_frontmatter(
    ui: &mut egui::Ui,
    app: &mut MdApp,
    entries: &[crate::document::FrontmatterEntry],
) {
    let accent = ui.visuals().hyperlink_color;
    let weak = ui.visuals().weak_text_color();
    let bg = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(45, 45, 45, 220)
    } else {
        Color32::from_rgba_unmultiplied(230, 235, 248, 200)
    };

    egui::Frame::new()
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let label = if app.frontmatter_expanded {
                    RichText::new("\u{25be} Metadata").color(accent).small()
                } else {
                    RichText::new("\u{25b8} Metadata").color(accent).small()
                };
                if ui.label(label).interact(egui::Sense::click()).clicked() {
                    app.frontmatter_expanded = !app.frontmatter_expanded;
                }
            });

            if app.frontmatter_expanded {
                ui.add_space(6.0);
                egui::Grid::new("fm_grid")
                    .num_columns(2)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        for entry in entries {
                            ui.label(RichText::new(&entry.key).small().color(weak).strong());
                            ui.label(RichText::new(&entry.value).small());
                            ui.end_row();
                        }
                    });
            }
        });
}

// ── Anchor / TOC helpers ──────────────────────────────────────────────────────

fn activate_toc_for_anchor(app: &mut MdApp, anchor: &str) {
    if let Some(doc) = &app.document {
        let toc = doc.toc.clone();
        for (i, entry) in toc.iter().enumerate() {
            if entry.anchor == anchor {
                app.active_toc_idx = Some(i);
                // section_ranges[i+1] corresponds to TOC entry i
                app.pending_scroll = Some(i + 1);
                if app.sidebar_open {
                    app.sidebar_tab = crate::app::SidebarTab::Outline;
                }
                return;
            }
        }
    }
}

// ── Empty state ───────────────────────────────────────────────────────────────

fn show_empty_state(ui: &mut egui::Ui, app: &mut MdApp) {
    let ctx = ui.ctx().clone();
    let dim = ui.visuals().weak_text_color();
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.25);
        ui.label(RichText::new("No file open").size(26.0).color(dim));
        ui.add_space(10.0);
        ui.label(
            RichText::new("Drop a .md file here, or use the toolbar above")
                .size(14.0)
                .color(dim),
        );
        ui.add_space(28.0);
        ui.horizontal(|ui| {
            let btn_w = 130.0;
            ui.add_space((ui.available_width() - btn_w * 2.0 - 8.0) / 2.0);
            if ui.button("📄  Open file\u{2026}").clicked() {
                app.open_file_dialog(&ctx);
            }
            ui.add_space(8.0);
            if ui.button("📁  Open folder\u{2026}").clicked() {
                app.open_folder_dialog();
            }
        });
    });
}

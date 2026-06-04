use egui::RichText;

use crate::app::MdApp;

pub fn show(app: &mut MdApp, ui: &mut egui::Ui) {
    egui::Panel::top("topbar")
        .size_range(38.0..=38.0)
        .show_inside(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(4.0);

                // Sidebar toggle
                let sidebar_toggle = ui
                    .add(egui::Button::new("").min_size(egui::vec2(26.0, 22.0)))
                    .on_hover_text("Toggle sidebar");
                paint_sidebar_toggle_icon(ui, sidebar_toggle.rect, app.sidebar_open);
                if sidebar_toggle.clicked() {
                    app.sidebar_open = !app.sidebar_open;
                }

                ui.separator();

                // Open file
                let ctx = ui.ctx().clone();
                if ui
                    .button("📄 Open")
                    .on_hover_text("Open file  Ctrl+O")
                    .clicked()
                {
                    app.open_file_dialog(&ctx);
                }

                // Open folder
                if ui
                    .button("📁 Folder")
                    .on_hover_text("Open folder")
                    .clicked()
                {
                    app.open_folder_dialog();
                }

                // Recent files menu
                if !app.recent_files.is_empty() {
                    let recents = app.recent_files.clone();
                    let recent_menu = ui.menu_button("Recent    ", |ui| {
                        let ctx = ui.ctx().clone();
                        for path in &recents {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            if ui
                                .button(name)
                                .on_hover_text(path.to_string_lossy().as_ref())
                                .clicked()
                            {
                                // Take a local copy to avoid the borrow conflict
                                let p = path.clone();
                                ui.close();
                                app.open_file_path(p, &ctx);
                            }
                        }
                    });
                    paint_dropdown_triangle(ui, recent_menu.response.rect);
                }

                ui.separator();

                // Search toggle
                let link_col = ui.visuals().hyperlink_color;
                let txt_col = ui.visuals().text_color();
                let s_col = if app.search_open { link_col } else { txt_col };
                if ui
                    .button(RichText::new("🔍").color(s_col))
                    .on_hover_text("Search  Ctrl+F")
                    .clicked()
                {
                    app.search_open = !app.search_open;
                    if !app.search_open {
                        app.search_results.clear();
                    }
                }

                if app.tts_busy {
                    if ui
                        .button("Stop")
                        .on_hover_text("Stop reading aloud")
                        .clicked()
                    {
                        app.stop_reading_aloud();
                    }
                } else if ui
                    .add_enabled(app.document.is_some(), egui::Button::new("Read"))
                    .on_hover_text("Read markdown aloud with Piper")
                    .clicked()
                {
                    app.read_aloud();
                }

                // Command palette
                let cp_col = if app.cmd_palette.open {
                    link_col
                } else {
                    txt_col
                };
                if ui
                    .button(RichText::new("\u{2318}").color(cp_col))
                    .on_hover_text("Command palette  Ctrl+Shift+P")
                    .clicked()
                {
                    app.cmd_palette.open = !app.cmd_palette.open;
                    if app.cmd_palette.open {
                        app.cmd_palette.query.clear();
                    }
                }

                // ── Right-aligned controls ────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    let ctx = ui.ctx().clone();
                    let (icon, tip) = if app.dark_mode {
                        ("\u{2600} Light", "Switch to light theme")
                    } else {
                        ("\u{1f319} Dark", "Switch to dark theme")
                    };
                    if ui.button(icon).on_hover_text(tip).clicked() {
                        app.dark_mode = !app.dark_mode;
                        crate::theme::apply(&ctx, app.dark_mode, app.zoom);
                    }

                    ui.separator();

                    let zoom_pct = format!("{:.0}%", app.zoom * 100.0);
                    if ui
                        .small_button("+")
                        .on_hover_text("Zoom in  Ctrl++")
                        .clicked()
                    {
                        app.zoom = (app.zoom + 0.1).min(3.0);
                        crate::theme::apply(&ctx, app.dark_mode, app.zoom);
                    }
                    if ui
                        .small_button(&zoom_pct)
                        .on_hover_text("Reset zoom  Ctrl+0")
                        .clicked()
                    {
                        app.zoom = 1.0;
                        crate::theme::apply(&ctx, app.dark_mode, app.zoom);
                    }
                    if ui
                        .small_button("-")
                        .on_hover_text("Zoom out  Ctrl+-")
                        .clicked()
                    {
                        app.zoom = (app.zoom - 0.1).max(0.5);
                        crate::theme::apply(&ctx, app.dark_mode, app.zoom);
                    }

                    // Status / word count
                    if app.tts_busy {
                        ui.separator();
                        ui.label(
                            RichText::new("Reading aloud...")
                                .small()
                                .color(egui::Color32::from_rgb(206, 145, 120)),
                        );
                    } else if let Some((msg, _)) = &app.status {
                        ui.separator();
                        ui.label(
                            RichText::new(msg)
                                .small()
                                .color(egui::Color32::from_rgb(206, 145, 120)),
                        );
                    } else if app.document.is_some() {
                        ui.separator();
                        let wc = app.word_count();
                        let mins = (wc / 200).max(1);
                        ui.label(
                            RichText::new(format!("{wc} words \u{00b7} ~{mins} min read"))
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                });
            });
        });
}

fn paint_dropdown_triangle(ui: &egui::Ui, rect: egui::Rect) {
    let center = egui::pos2(rect.right() - 13.0, rect.center().y + 1.0);
    let color = ui.visuals().text_color();
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(center.x - 4.0, center.y - 2.0),
            egui::pos2(center.x + 4.0, center.y - 2.0),
            egui::pos2(center.x, center.y + 3.0),
        ],
        color,
        egui::Stroke::NONE,
    ));
}

fn paint_sidebar_toggle_icon(ui: &egui::Ui, rect: egui::Rect, sidebar_open: bool) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    let stroke = egui::Stroke::new(1.5, color);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(16.0, 13.0));

    if sidebar_open {
        painter.rect_stroke(icon_rect, 2.0, stroke, egui::StrokeKind::Inside);
        let divider_x = icon_rect.left() + 5.0;
        painter.line_segment(
            [
                egui::pos2(divider_x, icon_rect.top()),
                egui::pos2(divider_x, icon_rect.bottom()),
            ],
            stroke,
        );
        painter.rect_filled(
            egui::Rect::from_min_max(
                icon_rect.left_top() + egui::vec2(1.5, 1.5),
                egui::pos2(divider_x - 1.5, icon_rect.bottom() - 1.5),
            ),
            1.0,
            color.linear_multiply(0.25),
        );
    } else {
        let left = icon_rect.left() + 2.0;
        let right = icon_rect.right() - 2.0;
        for y in [
            icon_rect.center().y - 4.0,
            icon_rect.center().y,
            icon_rect.center().y + 4.0,
        ] {
            painter.line_segment([egui::pos2(left, y), egui::pos2(right, y)], stroke);
        }
    }
}

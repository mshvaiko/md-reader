use egui::{Key, RichText};

use crate::app::MdApp;

pub fn show(app: &mut MdApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();

    egui::Window::new("##search")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 46.0])
        .min_width(340.0)
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                let input_id = egui::Id::new("search_input");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut app.search_query)
                        .hint_text("Search\u{2026}")
                        .min_size(egui::vec2(220.0, 0.0))
                        .id(input_id),
                );

                // Auto-focus when search is opened
                if !ctx.memory(|m| m.has_focus(input_id)) {
                    ctx.memory_mut(|m| m.request_focus(input_id));
                }

                if resp.changed() {
                    app.run_search();
                }

                // Enter to navigate forward
                if resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if !app.search_results.is_empty() {
                        app.search_idx = (app.search_idx + 1) % app.search_results.len();
                    }
                    ctx.memory_mut(|m| m.request_focus(input_id));
                }

                if !app.search_results.is_empty() {
                    let n = app.search_results.len();
                    let i = app.search_idx + 1;
                    ui.label(
                        RichText::new(format!("{i}/{n}"))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    if ui
                        .small_button("\u{25c4}")
                        .on_hover_text("Previous")
                        .clicked()
                    {
                        if app.search_idx == 0 {
                            app.search_idx = app.search_results.len() - 1;
                        } else {
                            app.search_idx -= 1;
                        }
                    }
                    if ui.small_button("\u{25ba}").on_hover_text("Next").clicked() {
                        app.search_idx = (app.search_idx + 1) % app.search_results.len();
                    }
                } else if !app.search_query.is_empty() {
                    ui.label(
                        RichText::new("No matches")
                            .small()
                            .color(egui::Color32::from_rgb(244, 135, 113)),
                    );
                }

                if ui.small_button("\u{2715}").clicked()
                    || ctx.input(|i| i.key_pressed(Key::Escape))
                {
                    app.search_open = false;
                    app.search_results.clear();
                    app.search_query.clear();
                }
            });
        });
}

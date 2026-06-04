use egui::{Key, RichText};

use crate::app::{MdApp, SidebarTab};

struct Command {
    label: &'static str,
    hint: &'static str,
}

const COMMANDS: &[Command] = &[
    Command {
        label: "Open File",
        hint: "Ctrl+O",
    },
    Command {
        label: "Open Folder",
        hint: "",
    },
    Command {
        label: "Toggle Theme",
        hint: "",
    },
    Command {
        label: "Toggle Search",
        hint: "Ctrl+F",
    },
    Command {
        label: "Zoom In",
        hint: "Ctrl++",
    },
    Command {
        label: "Zoom Out",
        hint: "Ctrl+-",
    },
    Command {
        label: "Reset Zoom",
        hint: "Ctrl+0",
    },
    Command {
        label: "Show Outline",
        hint: "",
    },
    Command {
        label: "Show Files",
        hint: "",
    },
    Command {
        label: "Toggle Sidebar",
        hint: "",
    },
];

pub fn show(app: &mut MdApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();

    // Dim background overlay
    let screen = ctx.viewport_rect();
    let overlay_id = egui::Id::new("cp_overlay");
    egui::Area::new(overlay_id)
        .fixed_pos(screen.min)
        .order(egui::Order::Background)
        .show(&ctx, |ui| {
            ui.painter().rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(100),
            );
        });

    egui::Window::new("##cmd_palette")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
        .fixed_size([480.0, 320.0])
        .show(&ctx, |ui| {
            // Search input
            let input_id = egui::Id::new("cp_input");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.cmd_palette.query)
                    .hint_text("Type a command\u{2026}")
                    .min_size(egui::vec2(ui.available_width(), 0.0))
                    .id(input_id),
            );

            if !ctx.memory(|m| m.has_focus(input_id)) {
                ctx.memory_mut(|m| m.request_focus(input_id));
            }

            ui.separator();

            // Close on Escape
            if resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Escape)) {
                app.cmd_palette.open = false;
                return;
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                app.cmd_palette.open = false;
                return;
            }

            // Filtered command list
            let query_lower = app.cmd_palette.query.to_lowercase();
            let weak_col = ui.visuals().weak_text_color();
            let link_col = ui.visuals().hyperlink_color;

            let mut executed = false;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for cmd in COMMANDS {
                    if !query_lower.is_empty() && !cmd.label.to_lowercase().contains(&query_lower) {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        let resp =
                            ui.selectable_label(false, RichText::new(cmd.label).color(link_col));
                        if !cmd.hint.is_empty() {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(RichText::new(cmd.hint).small().color(weak_col));
                                },
                            );
                        }
                        if resp.clicked() {
                            execute(cmd.label, app, &ctx);
                            executed = true;
                        }
                    });
                }
            });

            if executed {
                app.cmd_palette.open = false;
            }
        });
}

fn execute(label: &str, app: &mut MdApp, ctx: &egui::Context) {
    match label {
        "Open File" => app.open_file_dialog(ctx),
        "Open Folder" => app.open_folder_dialog(),
        "Toggle Theme" => {
            app.dark_mode = !app.dark_mode;
            crate::theme::apply(ctx, app.dark_mode, app.zoom);
        }
        "Toggle Search" => {
            app.search_open = !app.search_open;
            if !app.search_open {
                app.search_results.clear();
            }
        }
        "Zoom In" => {
            app.zoom = (app.zoom + 0.1).min(3.0);
            crate::theme::apply(ctx, app.dark_mode, app.zoom);
        }
        "Zoom Out" => {
            app.zoom = (app.zoom - 0.1).max(0.5);
            crate::theme::apply(ctx, app.dark_mode, app.zoom);
        }
        "Reset Zoom" => {
            app.zoom = 1.0;
            crate::theme::apply(ctx, app.dark_mode, app.zoom);
        }
        "Show Outline" => {
            app.sidebar_tab = SidebarTab::Outline;
            app.sidebar_open = true;
        }
        "Show Files" => {
            app.sidebar_tab = SidebarTab::FileTree;
            app.sidebar_open = true;
        }
        "Toggle Sidebar" => app.sidebar_open = !app.sidebar_open,
        _ => {}
    }
}

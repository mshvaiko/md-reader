use egui::{Color32, Context, CornerRadius, FontFamily, FontId, Margin, TextStyle, Visuals};

pub fn apply(ctx: &Context, dark: bool, zoom: f32) {
    ctx.set_visuals(if dark {
        dark_visuals()
    } else {
        light_visuals()
    });
    ctx.set_pixels_per_point(zoom);
    setup_style(ctx);
}

pub fn dark_visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.panel_fill = Color32::from_rgb(20, 22, 27);
    v.window_fill = Color32::from_rgb(20, 22, 27);
    v.faint_bg_color = Color32::from_rgb(26, 28, 35);
    v.extreme_bg_color = Color32::from_rgb(14, 15, 18);
    v.code_bg_color = Color32::from_rgb(28, 30, 40);
    v.override_text_color = Some(Color32::from_rgb(212, 216, 225));
    v.hyperlink_color = Color32::from_rgb(100, 180, 255);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(64, 120, 220, 80);

    set_widget_corners(&mut v, CornerRadius::same(6));

    v.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 30, 38);
    v.widgets.inactive.bg_fill = Color32::from_rgb(34, 37, 46);
    v.widgets.hovered.bg_fill = Color32::from_rgb(44, 48, 60);
    v.widgets.active.bg_fill = Color32::from_rgb(54, 58, 72);

    v.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(100, 106, 122);
    v.widgets.inactive.fg_stroke.color = Color32::from_rgb(160, 166, 182);
    v.widgets.hovered.fg_stroke.color = Color32::from_rgb(212, 216, 225);
    v.widgets.active.fg_stroke.color = Color32::WHITE;

    v
}

pub fn light_visuals() -> Visuals {
    let mut v = Visuals::light();
    v.panel_fill = Color32::from_rgb(248, 249, 251);
    v.window_fill = Color32::WHITE;
    v.faint_bg_color = Color32::from_rgb(242, 244, 248);
    v.extreme_bg_color = Color32::WHITE;
    v.code_bg_color = Color32::from_rgb(238, 240, 248);
    v.override_text_color = Some(Color32::from_rgb(28, 32, 40));
    v.hyperlink_color = Color32::from_rgb(0, 102, 204);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 102, 255, 50);

    set_widget_corners(&mut v, CornerRadius::same(6));

    v.widgets.noninteractive.bg_fill = Color32::from_rgb(235, 237, 245);
    v.widgets.inactive.bg_fill = Color32::from_rgb(225, 228, 238);
    v.widgets.hovered.bg_fill = Color32::from_rgb(210, 215, 230);
    v.widgets.active.bg_fill = Color32::from_rgb(190, 200, 220);

    v.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(140, 146, 160);
    v.widgets.inactive.fg_stroke.color = Color32::from_rgb(80, 88, 104);
    v.widgets.hovered.fg_stroke.color = Color32::from_rgb(28, 32, 40);
    v.widgets.active.fg_stroke.color = Color32::BLACK;

    v
}

fn set_widget_corners(v: &mut Visuals, r: CornerRadius) {
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.inactive.corner_radius = r;
    v.widgets.hovered.corner_radius = r;
    v.widgets.active.corner_radius = r;
    v.widgets.open.corner_radius = r;
}

fn setup_style(ctx: &Context) {
    let mut style = (*ctx.global_style()).clone();

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(21.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.5, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(11.5, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(6.0, 5.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin = Margin::same(10);

    ctx.set_global_style(style);
}

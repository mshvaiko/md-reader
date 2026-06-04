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
    v.panel_fill = Color32::from_rgb(37, 37, 38);
    v.window_fill = Color32::from_rgb(30, 30, 30);
    v.faint_bg_color = Color32::from_rgb(45, 45, 45);
    v.extreme_bg_color = Color32::from_rgb(24, 24, 24);
    v.code_bg_color = Color32::from_rgb(30, 30, 30);
    v.override_text_color = Some(Color32::from_rgb(212, 212, 212));
    v.hyperlink_color = Color32::from_rgb(55, 148, 255);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0, 122, 204, 90);

    set_widget_corners(&mut v, CornerRadius::same(3));

    v.widgets.noninteractive.bg_fill = Color32::from_rgb(45, 45, 45);
    v.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
    v.widgets.hovered.bg_fill = Color32::from_rgb(42, 45, 46);
    v.widgets.active.bg_fill = Color32::from_rgb(55, 55, 61);

    v.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(133, 133, 133);
    v.widgets.inactive.fg_stroke.color = Color32::from_rgb(204, 204, 204);
    v.widgets.hovered.fg_stroke.color = Color32::from_rgb(212, 212, 212);
    v.widgets.active.fg_stroke.color = Color32::from_rgb(255, 255, 255);

    v.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(60, 60, 60);
    v.widgets.inactive.bg_stroke.color = Color32::from_rgb(86, 86, 86);
    v.widgets.hovered.bg_stroke.color = Color32::from_rgb(0, 122, 204);
    v.widgets.active.bg_stroke.color = Color32::from_rgb(0, 122, 204);

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

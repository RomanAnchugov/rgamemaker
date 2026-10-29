//! Windows XP / Delphi / Game Maker 8 look: classic gray panels, sharp
//! corners, Tahoma-ish system font, XP selection blue.

use eframe::egui;

pub const WINDOW_BG: egui::Color32 = egui::Color32::from_rgb(0xEC, 0xE9, 0xD8); // XP dialog
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xD0, 0xC8); // classic gray
pub const BUTTON_FACE: egui::Color32 = egui::Color32::from_rgb(0xF4, 0xF2, 0xEC);
pub const BUTTON_HOVER: egui::Color32 = egui::Color32::from_rgb(0xFB, 0xF0, 0xD0);
pub const BUTTON_ACTIVE: egui::Color32 = egui::Color32::from_rgb(0xD6, 0xD2, 0xC4);
pub const BORDER_DARK: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x86, 0x7A);
pub const SEL_BLUE: egui::Color32 = egui::Color32::from_rgb(0x31, 0x6A, 0xC5);
pub const SEL_TEXT: egui::Color32 = egui::Color32::WHITE;
pub const CODE_BG: egui::Color32 = egui::Color32::WHITE;
pub const TEXT_DARK: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);

/// Install fonts + visuals. Call once at startup.
/// Returns (ui_font_name, mono_font_name) actually installed.
pub fn apply(ctx: &egui::Context) -> (String, String) {
    let names = install_fonts(ctx);
    let style = classic_style();
    ctx.set_style(style);
    names
}

fn install_fonts(ctx: &egui::Context) -> (String, String) {
    let mut installed = ("default".to_string(), "default".to_string());
    let ui_candidates = [
        "/System/Library/Fonts/Supplemental/Tahoma.ttf",
        "/System/Library/Fonts/Supplemental/Verdana.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    let home = std::env::var("HOME").unwrap_or_default();
    let mono_candidates = [
        // JetBrains Mono (the preferred code font)
        &format!("{}/Library/Fonts/JetBrainsMonoNL-Regular.ttf", home),
        &format!("{}/Library/Fonts/JetBrainsMono[wght].ttf", home),
        "C:\\Windows\\Fonts\\JetBrainsMonoNerdFont-Regular.ttf",
        "/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf",
        "/usr/share/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf",
        // fallbacks
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Supplemental/Courier New.ttf",
        "C:\\Windows\\Fonts\\consola.ttf",
        "C:\\Windows\\Fonts\\cour.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    ];
    let mut fonts = egui::FontDefinitions::default();
    if let Some(path) = ui_candidates.iter().find(|p| std::fs::read(p).is_ok()) {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("xp_ui".into(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "xp_ui".into());
            installed.0 = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("xp_ui")
                .to_string();
        }
    }
    if let Some(path) = mono_candidates.iter().find(|p| std::fs::read(p).is_ok()) {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("xp_mono".into(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "xp_mono".into());
            installed.1 = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("xp_mono")
                .to_string();
        }
    }
    ctx.set_fonts(fonts);
    installed
}

fn classic_style() -> egui::Style {
    let mut style = egui::Style::default();
    let v = &mut style.visuals;

    v.dark_mode = false;
    v.override_text_color = Some(TEXT_DARK);
    v.panel_fill = PANEL_BG;
    v.window_fill = WINDOW_BG;
    v.extreme_bg_color = CODE_BG; // code editor / text fields background
    v.faint_bg_color = egui::Color32::from_rgb(0xE6, 0xE3, 0xDA);
    v.hyperlink_color = egui::Color32::from_rgb(0x00, 0x00, 0xEE);
    v.warn_fg_color = egui::Color32::from_rgb(0x8A, 0x4B, 0x08);
    v.error_fg_color = egui::Color32::from_rgb(0xB0, 0x20, 0x20);

    // classic selection
    v.selection.bg_fill = SEL_BLUE;
    v.selection.stroke = egui::Stroke::new(1.0, SEL_TEXT);

    // sharp corners, classic button faces
    let w = &mut v.widgets;
    w.inactive.bg_fill = BUTTON_FACE;
    w.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    w.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_DARK);
    w.inactive.rounding = 0.0.into();
    w.hovered.bg_fill = BUTTON_HOVER;
    w.hovered.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    w.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_DARK);
    w.hovered.rounding = 0.0.into();
    w.active.bg_fill = BUTTON_ACTIVE;
    w.active.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    w.active.fg_stroke = egui::Stroke::new(1.0, TEXT_DARK);
    w.active.rounding = 0.0.into();
    w.open.bg_fill = BUTTON_HOVER;
    w.open.bg_stroke = egui::Stroke::new(1.0, BORDER_DARK);
    w.open.rounding = 0.0.into();
    v.widgets.noninteractive.bg_fill = PANEL_BG;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(190));

    style.spacing.button_padding = egui::vec2(6.0, 2.0);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.menu_margin = egui::Margin::same(4.0);
    style
}

/// Sunken field border (like an XP edit box / tree view).
pub fn sunken_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(WINDOW_BG)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(0x7A, 0x96, 0xDF),
        ))
        .inner_margin(2.0)
}

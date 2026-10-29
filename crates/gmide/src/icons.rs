//! Tiny pixel-art icons drawn with the painter (14x14 grid) — no unicode
//! glyphs, so nothing renders as tofu. Style matches GM8 toolbars.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Icon {
    New,
    Open,
    Save,
    Check,
    Run,
    Plus,
    Minus,
    Cross,
    Rename,
    Left,
    Right,
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
    Picker,
    Folder,
    Sprite,
    Sound,
    Background,
    Path,
    Script,
    Timeline,
    Font,
    Object,
    Room,
}

impl Icon {
    pub fn name(self) -> &'static str {
        match self {
            Icon::New => "New project",
            Icon::Open => "Open project…",
            Icon::Save => "Save (Ctrl+S)",
            Icon::Check => "Check code (F7)",
            Icon::Run => "Run game (F5)",
            Icon::Plus => "Add",
            Icon::Minus => "Remove",
            Icon::Cross => "Delete",
            Icon::Rename => "Rename",
            Icon::Left => "Previous",
            Icon::Right => "Next",
            Icon::Pencil => "Pencil (draw)",
            Icon::Eraser => "Eraser",
            Icon::Fill => "Flood fill",
            Icon::Line => "Line",
            Icon::Rect => "Rectangle outline",
            Icon::Picker => "Pick color",
            Icon::Folder => "Folder",
            Icon::Sprite => "Sprite",
            Icon::Sound => "Sound",
            Icon::Background => "Background",
            Icon::Path => "Path",
            Icon::Script => "Script",
            Icon::Timeline => "Timeline",
            Icon::Font => "Font",
            Icon::Object => "Object",
            Icon::Room => "Room",
        }
    }

    /// Draw the icon inside the given square rect (fits to size).
    pub fn draw(self, painter: &egui::Painter, rect: egui::Rect) {
        let s = rect.width() / 14.0; // pixel scale
        let o = rect.min;
        let px = |x: i32, y: i32, w: u32, h: u32, c: egui::Color32| {
            let r = egui::Rect::from_min_size(
                o + egui::vec2(x as f32 * s, y as f32 * s),
                egui::vec2(w as f32 * s, h as f32 * s),
            );
            painter.rect_filled(r, 0.0, c);
        };
        let k = egui::Color32::BLACK; // main ink
        let w = egui::Color32::WHITE;
        let b = egui::Color32::from_rgb(0x2A, 0x5A, 0xD8); // blue accent
        let g = egui::Color32::from_rgb(0x3A, 0x8A, 0x3A); // green accent
        let y = egui::Color32::from_rgb(0xE8, 0xB4, 0x30); // yellow accent
        match self {
            Icon::New => {
                // white page with folded corner
                px(3, 1, 7, 12, w);
                px(3, 1, 7, 1, k);
                px(3, 13, 7, 1, k);
                px(3, 1, 1, 13, k);
                px(9, 1, 1, 13, k);
                px(7, 1, 2, 2, egui::Color32::from_gray(200));
                px(5, 5, 4, 1, b);
                px(5, 7, 4, 1, b);
                px(5, 9, 3, 1, b);
            }
            Icon::Open => {
                // yellow folder, open
                px(1, 3, 5, 2, y);
                px(1, 3, 5, 1, k);
                px(1, 5, 5, 1, k);
                px(1, 3, 1, 3, k);
                px(5, 4, 1, 2, k);
                px(1, 5, 11, 7, y);
                px(1, 11, 11, 1, egui::Color32::from_rgb(0xA8, 0x78, 0x10));
                px(1, 5, 1, 7, k);
                px(11, 5, 1, 7, k);
                px(1, 5, 11, 1, k);
                px(3, 8, 7, 1, egui::Color32::from_rgb(0xC8, 0x98, 0x20));
            }
            Icon::Save => {
                // floppy disk
                px(2, 1, 10, 12, egui::Color32::from_rgb(0x2A, 0x2A, 0x6A));
                px(2, 1, 10, 1, k);
                px(2, 12, 10, 1, k);
                px(2, 1, 1, 12, k);
                px(11, 1, 1, 12, k);
                px(5, 1, 4, 5, w);
                px(5, 1, 4, 1, k);
                px(4, 8, 6, 5, w);
                px(4, 8, 6, 1, k);
                px(5, 9, 1, 3, b);
            }
            Icon::Check => {
                let pts = [
                    (2, 7),
                    (3, 8),
                    (4, 9),
                    (5, 10),
                    (6, 9),
                    (7, 8),
                    (8, 7),
                    (9, 6),
                    (10, 5),
                    (11, 4),
                ];
                for (x, y) in pts {
                    px(x, y, 2, 2, g);
                }
                px(6, 8, 1, 2, g);
            }
            Icon::Run => {
                // green play triangle: column x = 4+i grows from top/bottom toward center
                for i in 0..6 {
                    let h = (8i32 - 2 * i).max(1) as u32;
                    px(4 + i, 3 + i, 1, h, g);
                }
            }
            Icon::Plus => {
                px(6, 2, 2, 10, k);
                px(2, 6, 10, 2, k);
                px(6, 3, 1, 8, w);
                px(3, 6, 8, 1, w);
            }
            Icon::Minus => {
                px(2, 6, 10, 2, k);
                px(3, 6, 8, 1, w);
            }
            Icon::Cross => {
                for i in 0..5 {
                    px(
                        3 + i,
                        3 + i,
                        2,
                        2,
                        egui::Color32::from_rgb(0xA0, 0x20, 0x20),
                    );
                    px(
                        3 + i,
                        11 - i,
                        2,
                        2,
                        egui::Color32::from_rgb(0xA0, 0x20, 0x20),
                    );
                }
            }
            Icon::Rename => {
                // pencil with label
                px(1, 11, 5, 2, k);
                px(7, 2, 2, 2, egui::Color32::from_rgb(0xE8, 0xB4, 0x30));
                px(6, 4, 2, 2, egui::Color32::from_rgb(0xE8, 0xB4, 0x30));
                px(5, 6, 2, 2, egui::Color32::from_rgb(0xE8, 0xB4, 0x30));
                px(4, 8, 2, 2, egui::Color32::from_rgb(0xE8, 0xB4, 0x30));
                px(3, 10, 2, 2, egui::Color32::from_rgb(0xD0, 0x8A, 0x20));
            }
            Icon::Left => {
                // ◄ — apex on the left, vertically centered
                for i in 0..4 {
                    let h = 2 + 2 * i;
                    px(4 + i, 7 - h / 2, 1, h as u32, k);
                }
            }
            Icon::Right => {
                // ► — apex on the right, vertically centered
                for i in 0..4 {
                    let h = 2 + 2 * i;
                    px(9 - i, 7 - h / 2, 1, h as u32, k);
                }
            }
            Icon::Pencil => {
                px(10, 2, 2, 2, y);
                px(9, 4, 2, 2, y);
                px(8, 6, 2, 2, y);
                px(7, 8, 2, 2, y);
                px(6, 10, 2, 2, egui::Color32::from_rgb(0xC0, 0x80, 0x1A));
                px(4, 12, 3, 1, k);
                px(2, 12, 1, 1, k);
                px(2, 11, 1, 1, k);
                px(3, 11, 1, 1, k);
            }
            Icon::Eraser => {
                px(3, 8, 8, 4, egui::Color32::from_rgb(0xF0, 0xA8, 0xC8));
                px(3, 8, 8, 1, k);
                px(3, 11, 8, 1, k);
                px(3, 8, 1, 4, k);
                px(10, 8, 1, 4, k);
                px(2, 12, 10, 1, k);
            }
            Icon::Fill => {
                // tilted bucket
                px(8, 2, 3, 3, b);
                px(4, 5, 5, 5, egui::Color32::from_gray(160));
                px(3, 7, 5, 5, egui::Color32::from_gray(120));
                px(3, 10, 3, 2, b);
                px(9, 10, 2, 3, b);
                px(11, 11, 2, 2, b);
            }
            Icon::Line => {
                for i in 0..8 {
                    px(3 + i, 11 - i, 2, 2, k);
                }
            }
            Icon::Rect => {
                px(2, 3, 10, 1, k);
                px(2, 11, 10, 1, k);
                px(2, 3, 1, 9, k);
                px(11, 3, 1, 9, k);
            }
            Icon::Picker => {
                px(9, 2, 2, 3, egui::Color32::from_gray(120));
                px(8, 5, 2, 2, egui::Color32::from_gray(120));
                px(6, 7, 2, 2, egui::Color32::from_gray(150));
                px(5, 9, 2, 2, egui::Color32::from_gray(170));
                px(3, 11, 3, 1, k);
                px(3, 10, 1, 1, k);
            }
            Icon::Folder => {
                px(1, 3, 5, 2, y);
                px(1, 5, 12, 7, y);
                px(1, 3, 12, 1, k);
                px(1, 11, 12, 1, k);
                px(1, 3, 1, 9, k);
                px(12, 3, 1, 9, k);
                px(3, 7, 8, 1, egui::Color32::from_rgb(0xC8, 0x98, 0x20));
            }
            Icon::Sprite => {
                // little smiley sprite
                px(4, 3, 6, 1, k);
                px(3, 4, 8, 1, k);
                px(2, 5, 10, 5, y);
                px(2, 10, 10, 1, k);
                px(2, 5, 1, 6, k);
                px(11, 5, 1, 6, k);
                px(4, 6, 2, 2, k);
                px(8, 6, 2, 2, k);
                px(5, 9, 4, 1, k);
            }
            Icon::Sound => {
                px(2, 5, 2, 4, k);
                px(4, 4, 2, 6, k);
                px(6, 3, 2, 8, k);
                px(9, 4, 1, 2, b);
                px(10, 6, 1, 2, b);
                px(11, 3, 1, 2, b);
                px(12, 8, 1, 2, b);
            }
            Icon::Background => {
                px(2, 2, 10, 10, egui::Color32::from_rgb(0x6A, 0x9A, 0xE8));
                px(2, 2, 10, 1, k);
                px(2, 11, 10, 1, k);
                px(2, 2, 1, 10, k);
                px(11, 2, 1, 10, k);
                px(4, 4, 3, 3, w);
                px(8, 7, 2, 2, egui::Color32::from_rgb(0x9A, 0xC0, 0xF0));
                px(3, 9, 2, 1, w);
            }
            Icon::Path => {
                for (x, y) in [(2, 11), (4, 8), (6, 6), (8, 4), (11, 2)] {
                    px(x, y, 2, 2, b);
                }
                for (x, y) in [(4, 11), (6, 9), (8, 6), (10, 4)] {
                    px(x, y, 1, 1, b);
                }
                px(4, 9, 1, 1, b);
                px(7, 5, 1, 1, b);
                px(9, 4, 1, 1, b);
            }
            Icon::Script => {
                px(3, 2, 8, 10, w);
                px(3, 2, 8, 1, k);
                px(3, 11, 8, 1, k);
                px(3, 2, 1, 10, k);
                px(10, 2, 1, 10, k);
                px(5, 4, 4, 1, b);
                px(5, 6, 4, 1, b);
                px(5, 8, 2, 1, egui::Color32::from_rgb(0xA0, 0x30, 0x30));
            }
            Icon::Timeline => {
                px(2, 12, 10, 1, k);
                for i in 0..10 {
                    if i % 2 == 0 {
                        px(2 + i, 10, 1, 2, egui::Color32::from_gray(120));
                    }
                }
                px(3, 3, 2, 7, b);
                px(7, 6, 2, 4, g);
                px(10, 1, 2, 9, egui::Color32::from_rgb(0xA0, 0x30, 0xA0));
            }
            Icon::Font => {
                px(3, 2, 2, 10, k);
                px(3, 2, 8, 1, k);
                px(6, 2, 1, 2, w);
                px(9, 5, 2, 7, k);
                px(9, 5, 3, 1, k);
                px(7, 8, 3, 1, k);
            }
            Icon::Object => {
                // blue ball
                px(5, 2, 4, 1, k);
                px(3, 3, 8, 1, k);
                px(2, 4, 10, 6, b);
                px(3, 10, 8, 1, k);
                px(5, 11, 4, 1, k);
                px(4, 4, 2, 2, w);
            }
            Icon::Room => {
                px(2, 3, 10, 8, egui::Color32::from_rgb(0xB4, 0xD8, 0xB4));
                px(2, 3, 10, 1, k);
                px(2, 10, 10, 1, k);
                px(2, 3, 1, 8, k);
                px(11, 3, 1, 8, k);
                px(4, 5, 2, 2, b);
                px(8, 7, 2, 2, y);
                px(4, 5, 2, 2, b);
            }
        }
    }
}

/// A small toolbar icon button (XP-style flat button with hover).
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, toggled: bool) -> bool {
    let size = egui::vec2(24.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let _visuals = ui.style().interact(&response);
    let bg = if toggled {
        crate::theme::BUTTON_ACTIVE
    } else if response.hovered() {
        crate::theme::BUTTON_HOVER
    } else {
        egui::Color32::TRANSPARENT
    };
    if bg != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, bg);
        ui.painter()
            .rect_stroke(rect, 0.0, egui::Stroke::new(1.0, crate::theme::BORDER_DARK));
    }
    let inner = egui::Rect::from_min_size(rect.min + egui::vec2(4.0, 4.0), egui::vec2(14.0, 14.0));
    icon.draw(ui.painter(), inner);
    response.on_hover_text(icon.name()).clicked()
}

/// Draw an icon inline (16px) followed by text as a rich label.
pub fn icon_and_text(ui: &mut egui::Ui, icon: Icon, text: &str, selected: bool) -> egui::Response {
    let color = if selected {
        crate::theme::SEL_TEXT
    } else {
        crate::theme::TEXT_DARK
    };
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 0.0, crate::theme::SEL_BLUE);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(0xE6, 0xE3, 0xD5));
    }
    icon.draw(
        ui.painter(),
        egui::Rect::from_min_size(rect.min + egui::vec2(2.0, 1.0), egui::vec2(14.0, 14.0)),
    );
    ui.painter().text(
        rect.min + egui::vec2(20.0, 8.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(12.5),
        color,
    );
    response
}

/// A tree row with a live sprite thumbnail instead of an abstract icon.
pub fn thumb_and_text(
    ui: &mut egui::Ui,
    tex: egui::TextureId,
    tex_size: (f32, f32),
    text: &str,
    selected: bool,
    fallback: Option<egui::Color32>,
) -> egui::Response {
    let color = if selected {
        crate::theme::SEL_TEXT
    } else {
        crate::theme::TEXT_DARK
    };
    let box_size = 16.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), box_size + 2.0),
        egui::Sense::click(),
    );
    if selected {
        ui.painter().rect_filled(rect, 0.0, crate::theme::SEL_BLUE);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(0xE6, 0xE3, 0xD5));
    }
    let img_box = egui::Rect::from_min_size(
        rect.min + egui::vec2(2.0, 1.0),
        egui::vec2(box_size, box_size),
    );
    if tex_size.0 > 0.0 && tex_size.1 > 0.0 {
        // fit into the box, keep aspect
        let scale = (box_size / tex_size.0).min(box_size / tex_size.1).min(1.0);
        let dw = tex_size.0 * scale;
        let dh = tex_size.1 * scale;
        let drect = egui::Rect::from_center_size(img_box.center(), egui::vec2(dw, dh));
        // checkerboard behind transparent pixels
        ui.painter()
            .rect_filled(drect, 0.0, egui::Color32::from_rgb(0xCC, 0xCC, 0xCC));
        ui.painter().image(
            tex,
            drect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else if let Some(f) = fallback {
        ui.painter().rect_filled(img_box.shrink(1.0), 0.0, f);
        ui.painter()
            .rect_stroke(img_box, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
    }
    ui.painter().text(
        rect.min + egui::vec2(22.0, (box_size + 2.0) / 2.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(12.5),
        color,
    );
    response
}

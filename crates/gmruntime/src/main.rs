use gmengine::backend::Backend;
use gmengine::Game;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

include!(concat!(env!("OUT_DIR"), "/embedded_project.rs"));
use macroquad::text::{load_ttf_font_from_bytes, Font};
use std::collections::HashSet;

const KEYMAP: &[(KeyCode, i64)] = &[
    (KeyCode::Space, 32),
    (KeyCode::Enter, 13),
    (KeyCode::Escape, 27),
    (KeyCode::Backspace, 8),
    (KeyCode::Tab, 9),
    (KeyCode::Insert, 45),
    (KeyCode::Delete, 46),
    (KeyCode::Home, 36),
    (KeyCode::End, 35),
    (KeyCode::PageUp, 33),
    (KeyCode::PageDown, 34),
    (KeyCode::Left, 37),
    (KeyCode::Right, 39),
    (KeyCode::Up, 38),
    (KeyCode::Down, 40),
    (KeyCode::LeftShift, 16),
    (KeyCode::RightShift, 16),
    (KeyCode::LeftControl, 17),
    (KeyCode::RightControl, 17),
    (KeyCode::LeftAlt, 18),
    (KeyCode::RightAlt, 18),
    (KeyCode::GraveAccent, 192),
    (KeyCode::Minus, 189),
    (KeyCode::Equal, 187),
    (KeyCode::Backslash, 220),
    (KeyCode::LeftBracket, 219),
    (KeyCode::RightBracket, 221),
    (KeyCode::Semicolon, 186),
    (KeyCode::Apostrophe, 222),
    (KeyCode::Comma, 188),
    (KeyCode::Period, 190),
    (KeyCode::Slash, 191),
    (KeyCode::Key0, 48),
    (KeyCode::Key1, 49),
    (KeyCode::Key2, 50),
    (KeyCode::Key3, 51),
    (KeyCode::Key4, 52),
    (KeyCode::Key5, 53),
    (KeyCode::Key6, 54),
    (KeyCode::Key7, 55),
    (KeyCode::Key8, 56),
    (KeyCode::Key9, 57),
    (KeyCode::A, 65),
    (KeyCode::B, 66),
    (KeyCode::C, 67),
    (KeyCode::D, 68),
    (KeyCode::E, 69),
    (KeyCode::F, 70),
    (KeyCode::G, 71),
    (KeyCode::H, 72),
    (KeyCode::I, 73),
    (KeyCode::J, 74),
    (KeyCode::K, 75),
    (KeyCode::L, 76),
    (KeyCode::M, 77),
    (KeyCode::N, 78),
    (KeyCode::O, 79),
    (KeyCode::P, 80),
    (KeyCode::Q, 81),
    (KeyCode::R, 82),
    (KeyCode::S, 83),
    (KeyCode::T, 84),
    (KeyCode::U, 85),
    (KeyCode::V, 86),
    (KeyCode::W, 87),
    (KeyCode::X, 88),
    (KeyCode::Y, 89),
    (KeyCode::Z, 90),
    (KeyCode::F1, 112),
    (KeyCode::F2, 113),
    (KeyCode::F3, 114),
    (KeyCode::F4, 115),
    (KeyCode::F5, 116),
    (KeyCode::F6, 117),
    (KeyCode::F7, 118),
    (KeyCode::F8, 119),
    (KeyCode::F9, 120),
    (KeyCode::F10, 121),
    (KeyCode::F11, 122),
    (KeyCode::F12, 123),
    (KeyCode::Kp0, 96),
    (KeyCode::Kp1, 97),
    (KeyCode::Kp2, 98),
    (KeyCode::Kp3, 99),
    (KeyCode::Kp4, 100),
    (KeyCode::Kp5, 101),
    (KeyCode::Kp6, 102),
    (KeyCode::Kp7, 103),
    (KeyCode::Kp8, 104),
    (KeyCode::Kp9, 105),
    (KeyCode::KpMultiply, 106),
    (KeyCode::KpAdd, 107),
    (KeyCode::KpSubtract, 109),
    (KeyCode::KpDecimal, 110),
    (KeyCode::KpDivide, 111),
];

fn gm_color(c: u32, alpha: f32) -> Color {
    Color::new(
        (c & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        alpha.clamp(0.0, 1.0),
    )
}

struct MqBackend {
    textures: Vec<Texture2D>,
    sounds: Vec<Option<Sound>>,
    sound_counter: u32,
    /// 0 = built-in font; others are loaded TTFs.
    fonts: Vec<Option<Font>>,
    start: std::time::Instant,
}

impl MqBackend {
    fn new(sounds: Vec<Option<Sound>>) -> Self {
        MqBackend {
            textures: vec![],
            sounds,
            sound_counter: 0,
            fonts: vec![None],
            start: std::time::Instant::now(),
        }
    }
}

impl Backend for MqBackend {
    fn load_image(&mut self, _name: &str, rgba: &[u8], w: u32, h: u32) -> u32 {
        let tex = Texture2D::from_rgba8(w as u16, h as u16, rgba);
        tex.set_filter(FilterMode::Nearest);
        self.textures.push(tex);
        (self.textures.len() - 1) as u32
    }
    fn load_sound(&mut self, _name: &str, _bytes: &[u8]) -> u32 {
        // sounds are preloaded by main() in project order; keep ids aligned
        let id = self.sound_counter;
        self.sound_counter += 1;
        id
    }
    fn load_font(&mut self, _name: &str, bytes: &[u8], _size: f64) -> u32 {
        if bytes.is_empty() {
            self.fonts.push(None);
        } else {
            match load_ttf_font_from_bytes(bytes) {
                Ok(f) => self.fonts.push(Some(f)),
                Err(_) => self.fonts.push(None),
            }
        }
        (self.fonts.len() - 1) as u32
    }
    fn play_sound(&mut self, id: u32, looped: bool, volume: f64) {
        if let Some(Some(s)) = self.sounds.get(id as usize) {
            play_sound(
                s,
                PlaySoundParams {
                    looped,
                    volume: volume.clamp(0.0, 1.0) as f32,
                },
            );
        }
    }
    fn stop_sound(&mut self, id: u32) {
        if let Some(Some(s)) = self.sounds.get(id as usize) {
            stop_sound(s);
        }
    }
    fn sound_playing(&self, id: u32) -> bool {
        matches!(self.sounds.get(id as usize), Some(Some(_)))
    }
    #[allow(clippy::too_many_arguments)]
    fn draw_sprite(
        &mut self,
        tex: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rot_deg: f32,
        alpha: f32,
        color_bgr: u32,
        flip_x: bool,
        flip_y: bool,
        src: Option<(f32, f32, f32, f32)>,
    ) {
        if let Some(t) = self.textures.get(tex as usize) {
            let source = src.map(|(l, tp, sw, sh)| Rect::new(l, tp, sw, sh));
            draw_texture_ex(
                t,
                x,
                y,
                gm_color(color_bgr, alpha),
                DrawTextureParams {
                    dest_size: Some(Vec2::new(w, h)),
                    rotation: rot_deg.to_radians(),
                    flip_x,
                    flip_y,
                    source,
                    ..Default::default()
                },
            );
        }
    }
    fn draw_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        filled: bool,
        color_bgr: u32,
        alpha: f32,
    ) {
        let c = gm_color(color_bgr, alpha);
        if filled {
            draw_rectangle(x, y, w, h, c);
        } else {
            draw_rectangle_lines(x, y, w, h, 1.0, c);
        }
    }
    fn draw_circle(&mut self, x: f32, y: f32, r: f32, filled: bool, color_bgr: u32, alpha: f32) {
        let c = gm_color(color_bgr, alpha);
        if filled {
            draw_circle(x, y, r, c);
        } else {
            draw_circle_lines(x, y, r, 1.0, c);
        }
    }
    fn draw_ellipse(
        &mut self,
        x: f32,
        y: f32,
        rx: f32,
        ry: f32,
        filled: bool,
        color_bgr: u32,
        alpha: f32,
    ) {
        let c = gm_color(color_bgr, alpha);
        if filled {
            draw_ellipse(x, y, rx, ry, 0.0, c);
        } else {
            // approximate outline with line segments
            let n = 24;
            for i in 0..n {
                let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
                draw_line(
                    x + a0.cos() * rx,
                    y + a0.sin() * ry,
                    x + a1.cos() * rx,
                    y + a1.sin() * ry,
                    1.0,
                    c,
                );
            }
        }
    }
    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color_bgr: u32,
        alpha: f32,
    ) {
        draw_line(x1, y1, x2, y2, width, gm_color(color_bgr, alpha));
    }
    fn draw_text(
        &mut self,
        font: u32,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        color_bgr: u32,
        alpha: f32,
    ) {
        let font_ref = self.fonts.get(font as usize).and_then(|f| f.as_ref());
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: font_ref,
                font_size: size.max(4.0) as u16,
                color: gm_color(color_bgr, alpha),
                ..Default::default()
            },
        );
    }
    fn text_width(&mut self, font: u32, text: &str, size: f32) -> f32 {
        let font_ref = self.fonts.get(font as usize).and_then(|f| f.as_ref());
        measure_text(text, font_ref, size.max(4.0) as u16, 1.0).width
    }
    fn text_height(&mut self, _font: u32, size: f32) -> f32 {
        size
    }
    fn clear(&mut self, color_bgr: u32) {
        clear_background(gm_color(color_bgr, 1.0));
    }
    fn now_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

/// Project to run: the CLI argument, otherwise the embedded one (if any).
fn startup_project() -> Result<gmformat::Project, String> {
    if let Some(p) = std::env::args().nth(1) {
        return gmformat::load(std::path::Path::new(&p));
    }
    if let Some(bytes) = EMBEDDED_PROJECT {
        let text = std::str::from_utf8(bytes).map_err(|e| format!("embedded project: {}", e))?;
        return gmformat::parse(text);
    }
    Err("no project given (usage: gmruntime <project.gmproj>)".into())
}

fn window_conf() -> Conf {
    let (w, h, title) = startup_project()
        .ok()
        .map(|proj| {
            let first = proj
                .rooms
                .first()
                .cloned()
                .unwrap_or_else(|| gmformat::Room::new("room0", 640, 480));
            let vw = first.view.as_ref().map(|v| v.w).unwrap_or(first.width);
            let vh = first.view.as_ref().map(|v| v.h).unwrap_or(first.height);
            (vw as i32, vh as i32, proj.name)
        })
        .unwrap_or((640, 480, "RGM Runtime".to_string()));
    Conf {
        window_title: title,
        window_width: w.max(200),
        window_height: h.max(200),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let project = match startup_project() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    // preload sounds (async in macroquad)
    let mut sounds: Vec<Option<Sound>> = Vec::new();
    for s in &project.sounds {
        let bytes = gmformat::decode_rgba(&s.data);
        match load_sound_from_bytes(&bytes).await {
            Ok(snd) => sounds.push(Some(snd)),
            Err(e) => {
                eprintln!("warning: cannot load sound '{}': {}", s.name, e);
                sounds.push(None);
            }
        }
    }
    let backend = MqBackend::new(sounds);
    let mut game = match Game::new(project, Box::new(backend)) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let mut last = get_time();
    let mut acc = 0.0;
    loop {
        // ---- gather input ----
        let mut down = HashSet::new();
        let mut pressed = HashSet::new();
        let mut released = HashSet::new();
        for (kc, vk) in KEYMAP {
            if is_key_down(*kc) {
                down.insert(*vk);
            }
            if is_key_pressed(*kc) {
                pressed.insert(*vk);
            }
            if is_key_released(*kc) {
                released.insert(*vk);
            }
        }
        let (mx, my) = mouse_position();
        let mut bits = 0u8;
        let mut pbits = 0u8;
        let mut rbits = 0u8;
        if is_mouse_button_down(MouseButton::Left) {
            bits |= 1;
        }
        if is_mouse_button_down(MouseButton::Right) {
            bits |= 2;
        }
        if is_mouse_button_down(MouseButton::Middle) {
            bits |= 4;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            pbits |= 1;
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            pbits |= 2;
        }
        if is_mouse_button_pressed(MouseButton::Middle) {
            pbits |= 4;
        }
        if is_mouse_button_released(MouseButton::Left) {
            rbits |= 1;
        }
        if is_mouse_button_released(MouseButton::Right) {
            rbits |= 2;
        }
        if is_mouse_button_released(MouseButton::Middle) {
            rbits |= 4;
        }
        let wheel = mouse_wheel().1;
        game.push_input(
            down,
            pressed,
            released,
            mx as f64,
            my as f64,
            bits,
            pbits,
            rbits,
            if wheel > 0.0 {
                1
            } else if wheel < 0.0 {
                -1
            } else {
                0
            },
        );

        // ---- fixed-step update ----
        let now = get_time();
        acc += now - last;
        last = now;
        let step = 1.0 / game.room_speed_val.clamp(1.0, 240.0);
        let mut guard = 0;
        while acc >= step && guard < 8 {
            game.step();
            acc -= step;
            guard += 1;
            if game.ended() {
                break;
            }
        }
        if acc > step * 8.0 {
            acc = 0.0;
        }

        // ---- draw ----
        game.draw();
        for e in game.take_errors() {
            eprintln!("{}", e);
        }
        if game.ended() {
            break;
        }
        next_frame().await;
    }
    std::process::exit(0);
}

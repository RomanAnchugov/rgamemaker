//! Rendering/audio/time abstraction. The engine is headless logic; a
//! Backend (macroquad in `gmruntime`, Null in tests) does the work.

pub trait Backend {
    /// Upload a raw RGBA image, return an opaque texture id.
    fn load_image(&mut self, name: &str, rgba: &[u8], w: u32, h: u32) -> u32;
    /// Register a sound (bytes), return an opaque sound id.
    fn load_sound(&mut self, name: &str, bytes: &[u8]) -> u32;
    /// Load a TTF font (empty bytes = built-in font). Returns font id.
    fn load_font(&mut self, name: &str, bytes: &[u8], size: f64) -> u32;
    fn play_sound(&mut self, id: u32, looped: bool, volume: f64);
    fn stop_sound(&mut self, id: u32);
    fn sound_playing(&self, id: u32) -> bool;

    /// Draw a texture with destination size (w,h), top-left at (x,y),
    /// rotated by `rot_deg` degrees around its center. `color_bgr` is a
    /// GM color tint (0xBBGGRR). `src` optionally selects a sub-rectangle
    /// of the texture in pixels (left, top, w, h).
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
    );
    fn draw_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        filled: bool,
        color_bgr: u32,
        alpha: f32,
    );
    fn draw_circle(&mut self, x: f32, y: f32, r: f32, filled: bool, color_bgr: u32, alpha: f32);
    fn draw_ellipse(
        &mut self,
        x: f32,
        y: f32,
        rx: f32,
        ry: f32,
        filled: bool,
        color_bgr: u32,
        alpha: f32,
    );
    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color_bgr: u32,
        alpha: f32,
    );
    /// Draw text with the given font id (0 = built-in) at `size` px.
    fn draw_text(
        &mut self,
        font: u32,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        color_bgr: u32,
        alpha: f32,
    );
    fn text_width(&mut self, font: u32, text: &str, size: f32) -> f32;
    fn text_height(&mut self, font: u32, size: f32) -> f32;
    /// Clear the screen with a GM color.
    fn clear(&mut self, color_bgr: u32);
    /// Milliseconds since an arbitrary epoch (game start).
    fn now_ms(&self) -> f64;
}

/// Headless backend for tests and CI.
pub struct NullBackend {
    pub time_ms: f64,
    pub images: Vec<(u32, u32)>,
    pub sounds: Vec<u32>,
    pub fonts: Vec<u32>,
    pub texts_drawn: Vec<String>,
}

impl Default for NullBackend {
    fn default() -> Self {
        NullBackend {
            time_ms: 0.0,
            images: vec![],
            sounds: vec![],
            fonts: vec![],
            texts_drawn: vec![],
        }
    }
}

impl Backend for NullBackend {
    fn load_image(&mut self, _name: &str, _rgba: &[u8], w: u32, h: u32) -> u32 {
        self.images.push((w, h));
        (self.images.len() - 1) as u32
    }
    fn load_sound(&mut self, _name: &str, _bytes: &[u8]) -> u32 {
        self.sounds.push(0);
        (self.sounds.len() - 1) as u32
    }
    fn load_font(&mut self, _name: &str, _bytes: &[u8], size: f64) -> u32 {
        self.fonts.push(size as u32);
        (self.fonts.len() as u32).max(1) - 1
    }
    fn play_sound(&mut self, _id: u32, _looped: bool, _volume: f64) {}
    fn stop_sound(&mut self, _id: u32) {}
    fn sound_playing(&self, _id: u32) -> bool {
        false
    }
    fn draw_sprite(
        &mut self,
        _t: u32,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _r: f32,
        _a: f32,
        _c: u32,
        _fx: bool,
        _fy: bool,
        _s: Option<(f32, f32, f32, f32)>,
    ) {
    }
    fn draw_rect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _f: bool, _c: u32, _a: f32) {}
    fn draw_circle(&mut self, _x: f32, _y: f32, _r: f32, _f: bool, _c: u32, _a: f32) {}
    fn draw_ellipse(&mut self, _x: f32, _y: f32, _rx: f32, _ry: f32, _f: bool, _c: u32, _a: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _w: f32, _c: u32, _a: f32) {}
    fn draw_text(&mut self, _font: u32, _x: f32, _y: f32, text: &str, _s: f32, _c: u32, _a: f32) {
        self.texts_drawn.push(text.to_string());
    }
    fn text_width(&mut self, _font: u32, text: &str, size: f32) -> f32 {
        text.chars().count() as f32 * size * 0.5
    }
    fn text_height(&mut self, _font: u32, size: f32) -> f32 {
        size
    }
    fn clear(&mut self, _c: u32) {}
    fn now_ms(&self) -> f64 {
        self.time_ms
    }
}

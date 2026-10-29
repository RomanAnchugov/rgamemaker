//! Resource editors (sprites, sounds, backgrounds, paths, scripts,
//! timelines, fonts, objects, rooms) in XP/GM8 style.

use crate::codeedit::{code_editor, NameDb};
use crate::icons::{icon_button, Icon};
use crate::theme;
use crate::{canvas_zoom, import_image, object_color, AddKind, App, ResRef, Tool};
use eframe::egui;
use gmformat::*;

impl App {
    pub(crate) fn add_resource(&mut self, kind: AddKind) {
        match kind {
            AddKind::Sprite => {
                let name = App::unique_name(&self.project, "sprite0");
                self.project.sprites.push(Sprite::new(&name, 32, 32));
                self.sel = Some(ResRef::Sprite(self.project.sprites.len() - 1));
                self.spr_frame = 0;
            }
            AddKind::Sound => {
                self.sel = Some(ResRef::Sound(self.project.sounds.len()));
                self.project.sounds.push(Sound {
                    name: App::unique_name(&self.project, "sound0"),
                    kind: "wav".into(),
                    data: String::new(),
                    volume: 1.0,
                });
            }
            AddKind::Background => {
                self.sel = Some(ResRef::Background(self.project.backgrounds.len()));
                self.project.backgrounds.push(Background {
                    name: App::unique_name(&self.project, "background0"),
                    width: 1,
                    height: 1,
                    data: encode_rgba(&[0, 0, 0, 0]),
                });
            }
            AddKind::Path => {
                self.sel = Some(ResRef::Path(self.project.paths.len()));
                self.project
                    .paths
                    .push(Path::new(&App::unique_name(&self.project, "path0")));
            }
            AddKind::Script => {
                self.sel = Some(ResRef::Script(self.project.scripts.len()));
                self.project.scripts.push(Script {
                    name: App::unique_name(&self.project, "script0"),
                    code: String::new(),
                });
            }
            AddKind::Timeline => {
                self.sel = Some(ResRef::Timeline(self.project.timelines.len()));
                self.project
                    .timelines
                    .push(Timeline::new(&App::unique_name(&self.project, "timeline0")));
            }
            AddKind::Font => {
                self.sel = Some(ResRef::Font(self.project.fonts.len()));
                self.project
                    .fonts
                    .push(Font::new(&App::unique_name(&self.project, "font0"), 16.0));
            }
            AddKind::Object => {
                self.sel = Some(ResRef::Object(self.project.objects.len()));
                self.project
                    .objects
                    .push(GmObject::new(&App::unique_name(&self.project, "object0")));
            }
            AddKind::Room => {
                self.sel = Some(ResRef::Room(self.project.rooms.len()));
                self.project.rooms.push(Room::new(
                    &App::unique_name(&self.project, "room0"),
                    640,
                    480,
                ));
            }
        }
    }

    pub(crate) fn delete_resource(&mut self, res: ResRef) {
        match res {
            ResRef::Sprite(i) => {
                self.project.sprites.remove(i);
                self.frames_cache.remove(&i);
                self.sel = None;
            }
            ResRef::Sound(i) => {
                self.project.sounds.remove(i);
                self.sel = None;
            }
            ResRef::Background(i) => {
                self.project.backgrounds.remove(i);
                self.sel = None;
            }
            ResRef::Path(i) => {
                self.project.paths.remove(i);
                self.sel = None;
            }
            ResRef::Script(i) => {
                self.project.scripts.remove(i);
                self.sel = None;
            }
            ResRef::Timeline(i) => {
                self.project.timelines.remove(i);
                self.sel = None;
            }
            ResRef::Font(i) => {
                self.project.fonts.remove(i);
                self.sel = None;
            }
            ResRef::Object(i) => {
                self.project.objects.remove(i);
                for r in self.project.rooms.iter_mut() {
                    r.instances.retain(|ri| ri.object != i);
                    for ri in r.instances.iter_mut() {
                        if ri.object > i {
                            ri.object -= 1;
                        }
                    }
                }
                for o in self.project.objects.iter_mut() {
                    if o.parent == i as i64 {
                        o.parent = -1;
                    } else if o.parent > i as i64 {
                        o.parent -= 1;
                    }
                    o.events.retain(
                        |e| !matches!(e, EventDef::Collision { object, .. } if *object == i),
                    );
                    for e in o.events.iter_mut() {
                        if let EventDef::Collision { object, .. } = e {
                            if *object > i {
                                *object -= 1;
                            }
                        }
                    }
                }
                self.sel = None;
            }
            ResRef::Room(i) => {
                self.project.rooms.remove(i);
                self.sel = None;
            }
        }
    }

    // ------------------------------------------------ sprite editor

    const GM_PALETTE: [(u8, u8, u8); 42] = [
        (0, 0, 0),
        (128, 128, 128),
        (192, 192, 192),
        (255, 255, 255),
        (128, 0, 0),
        (192, 0, 0),
        (255, 0, 0),
        (255, 128, 128),
        (128, 64, 0),
        (192, 96, 0),
        (255, 128, 0),
        (255, 192, 128),
        (128, 128, 0),
        (192, 192, 0),
        (255, 255, 0),
        (255, 255, 128),
        (0, 128, 0),
        (0, 192, 0),
        (0, 255, 0),
        (128, 255, 128),
        (0, 128, 64),
        (0, 192, 96),
        (0, 255, 128),
        (128, 255, 192),
        (0, 128, 128),
        (0, 192, 192),
        (0, 255, 255),
        (128, 255, 255),
        (0, 0, 128),
        (0, 0, 192),
        (0, 0, 255),
        (128, 128, 255),
        (128, 0, 128),
        (192, 0, 192),
        (255, 0, 255),
        (255, 128, 255),
        (64, 0, 128),
        (96, 0, 192),
        (128, 0, 255),
        (192, 128, 255),
        (255, 255, 255),
        (0, 0, 0),
    ];

    pub(crate) fn sprite_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        let (w, h) = (
            self.project.sprites[idx].width,
            self.project.sprites[idx].height,
        );
        ui.heading(format!(
            "Sprite properties: {}",
            self.project.sprites[idx].name
        ));
        ui.separator();

        ui.horizontal(|ui| {
            // tool palette
            let tools = [
                (Tool::Pencil, Icon::Pencil),
                (Tool::Eraser, Icon::Eraser),
                (Tool::Fill, Icon::Fill),
                (Tool::Line, Icon::Line),
                (Tool::Rect, Icon::Rect),
                (Tool::Picker, Icon::Picker),
            ];
            for (t, icon) in tools {
                if icon_button(ui, icon, self.spr_tool == t) {
                    self.spr_tool = t;
                }
            }
            ui.separator();
            ui.label("zoom:");
            ui.add(egui::Slider::new(&mut self.spr_zoom, 1.0..=24.0).show_value(true));
        });

        ui.horizontal(|ui| {
            // GM palette
            let mut picked = None;
            for (i, (r, g, b)) in Self::GM_PALETTE.iter().enumerate() {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::click());
                let is_sel = self.spr_color == [*r, *g, *b];
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(*r, *g, *b));
                ui.painter().rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(
                        if is_sel { 2.0 } else { 1.0 },
                        if is_sel {
                            egui::Color32::RED
                        } else {
                            egui::Color32::BLACK
                        },
                    ),
                );
                if resp.clicked() {
                    picked = Some([*r, *g, *b]);
                }
                if (i + 1) % 14 == 0 {
                    ui.end_row();
                }
            }
            if let Some(c) = picked {
                self.spr_color = c;
            }
        });
        ui.horizontal(|ui| {
            ui.label("color:");
            let mut colf = [
                self.spr_color[0] as f32 / 255.0,
                self.spr_color[1] as f32 / 255.0,
                self.spr_color[2] as f32 / 255.0,
            ];
            ui.color_edit_button_rgb(&mut colf);
            self.spr_color = [
                (colf[0] * 255.0) as u8,
                (colf[1] * 255.0) as u8,
                (colf[2] * 255.0) as u8,
            ];
        });
        ui.separator();

        // frames bar
        ui.horizontal(|ui| {
            let n = self.project.sprites[idx].frames.len();
            if self.spr_frame >= n {
                self.spr_frame = n.saturating_sub(1);
            }
            ui.label(format!("Frame {}/{}", self.spr_frame + 1, n));
            if icon_button(ui, Icon::Left, false) {
                self.spr_frame = self.spr_frame.saturating_sub(1);
            }
            if icon_button(ui, Icon::Right, false) {
                self.spr_frame = (self.spr_frame + 1).min(n - 1);
            }
            if ui.button("Add blank").clicked() {
                self.frames_mut(idx).push(vec![0; (w * h * 4) as usize]);
                self.sync_frames(idx);
                self.spr_frame = n;
            }
            if ui.button("Duplicate").clicked() {
                let fr = self.spr_frame;
                let cur = self
                    .frames(idx)
                    .get(fr)
                    .cloned()
                    .unwrap_or_else(|| vec![0; (w * h * 4) as usize]);
                self.frames_mut(idx).push(cur);
                self.sync_frames(idx);
                self.spr_frame = n;
            }
            if ui.button("Delete frame").clicked() && n > 1 {
                let fr = self.spr_frame;
                self.frames_mut(idx).remove(fr);
                self.sync_frames(idx);
                self.spr_frame = self.spr_frame.min(n - 2);
            }
            if ui.button("Import image...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("image", &["png", "jpg", "bmp"])
                    .pick_file()
                {
                    match import_image(&path) {
                        Some((iw, ih, rgba)) => {
                            let spr = &mut self.project.sprites[idx];
                            spr.width = iw;
                            spr.height = ih;
                            self.frames_mut(idx).push(rgba);
                            self.sync_frames(idx);
                            self.spr_frame = self.project.sprites[idx].frames.len() - 1;
                            self.log(0, format!("imported {} ({}x{})", path.display(), iw, ih));
                        }
                        None => self.log(1, "cannot import image"),
                    }
                }
            }
        });

        // properties
        ui.horizontal(|ui| {
            let spr = &mut self.project.sprites[idx];
            ui.label("Origin (X, Y):");
            ui.add(egui::DragValue::new(&mut spr.xoffset).range(0..=2048));
            ui.add(egui::DragValue::new(&mut spr.yoffset).range(0..=2048));
            ui.separator();
            ui.label("Bounding box:");
            ui.add(egui::DragValue::new(&mut spr.bbox_left));
            ui.add(egui::DragValue::new(&mut spr.bbox_top));
            ui.add(egui::DragValue::new(&mut spr.bbox_right));
            ui.add(egui::DragValue::new(&mut spr.bbox_bottom));
            ui.separator();
            ui.checkbox(&mut spr.precise, "Precise collision checking");
        });

        // filmstrip: all frames, click to select (animation timeline)
        {
            let n = self.project.sprites[idx].frames.len();
            let (fw, fh) = (
                self.project.sprites[idx].width,
                self.project.sprites[idx].height,
            );
            let box_sz = 56.0f32;
            egui::ScrollArea::horizontal()
                .id_salt(format!("film_{}", idx))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for f in 0..n {
                            let is_sel = f == self.spr_frame;
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(box_sz, box_sz + 14.0),
                                egui::Sense::click(),
                            );
                            let painter = ui.painter_at(rect);
                            painter.rect_filled(rect, 0.0, theme::WINDOW_BG);
                            let border = if is_sel {
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(0x2A, 0x5A, 0xD8))
                            } else {
                                egui::Stroke::new(1.0, egui::Color32::from_gray(120))
                            };
                            let tex = self.sprite_texture(ui.ctx(), idx, f);
                            let scale = ((box_sz - 6.0) / fw as f32)
                                .min((box_sz - 6.0) / fh as f32)
                                .min(1.0);
                            let dw = fw as f32 * scale;
                            let dh = fh as f32 * scale;
                            let img_rect = egui::Rect::from_center_size(
                                rect.center() - egui::vec2(0.0, 7.0),
                                egui::vec2(dw, dh),
                            );
                            painter.rect_filled(
                                img_rect,
                                0.0,
                                egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                            );
                            painter.image(
                                tex,
                                img_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );
                            painter.rect_stroke(rect, 0.0, border);
                            painter.text(
                                rect.left_bottom() + egui::vec2(3.0, -2.0),
                                egui::Align2::LEFT_BOTTOM,
                                format!("{}", f + 1),
                                egui::FontId::monospace(10.0),
                                if is_sel {
                                    egui::Color32::from_rgb(0x2A, 0x5A, 0xD8)
                                } else {
                                    egui::Color32::from_gray(90)
                                },
                            );
                            if resp.clicked() {
                                self.spr_frame = f;
                            }
                            resp.on_hover_text(format!("Frame {}", f + 1));
                        }
                    });
                });
            ui.separator();
        }

        // canvas
        let zoom = self.spr_zoom;
        let cw = (w as f32 * zoom).max(64.0);
        let ch = (h as f32 * zoom).max(64.0);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(cw, ch), egui::Sense::click_and_drag());
        // pinch / wheel over the canvas zooms
        canvas_zoom(ui, rect, &mut self.spr_zoom, 1.0, 24.0, true);
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, theme::WINDOW_BG);
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        );
        let tex = self.sprite_texture(ui.ctx(), idx, self.spr_frame);
        painter.image(
            tex,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        if zoom >= 8.0 {
            let g = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30));
            for x in 1..w {
                let xx = rect.left() + x as f32 * zoom;
                painter.line_segment(
                    [egui::pos2(xx, rect.top()), egui::pos2(xx, rect.bottom())],
                    g,
                );
            }
            for y in 1..h {
                let yy = rect.top() + y as f32 * zoom;
                painter.line_segment(
                    [egui::pos2(rect.left(), yy), egui::pos2(rect.right(), yy)],
                    g,
                );
            }
        }
        // bbox overlay
        {
            let spr = &self.project.sprites[idx];
            let (ox, oy) = (spr.xoffset as f32, spr.yoffset as f32);
            let b = egui::Color32::from_rgb(200, 40, 40);
            let l = rect.left() + (spr.bbox_left as f32 + ox) * zoom;
            let t = rect.top() + (spr.bbox_top as f32 + oy) * zoom;
            let r = rect.left() + (spr.bbox_right as f32 + ox + 1.0) * zoom;
            let bm = rect.top() + (spr.bbox_bottom as f32 + oy + 1.0) * zoom;
            let st = egui::Stroke::new(1.0, b);
            painter.line_segment([egui::pos2(l, t), egui::pos2(r, t)], st);
            painter.line_segment([egui::pos2(r, t), egui::pos2(r, bm)], st);
            painter.line_segment([egui::pos2(r, bm), egui::pos2(l, bm)], st);
            painter.line_segment([egui::pos2(l, bm), egui::pos2(l, t)], st);
        }

        // painting
        let to_px = |p: egui::Pos2| -> (i32, i32) {
            (
                ((p.x - rect.left()) / zoom).floor() as i32,
                ((p.y - rect.top()) / zoom).floor() as i32,
            )
        };
        if let Some(p) = response.interact_pointer_pos() {
            let (px, py) = to_px(p);
            let inside = px >= 0 && py >= 0 && px < w as i32 && py < h as i32;
            let primary = response.dragged_by(egui::PointerButton::Primary)
                || response.clicked_by(egui::PointerButton::Primary);
            if primary && inside {
                match self.spr_tool {
                    Tool::Pencil => self.set_pixel(idx, px, py, true),
                    Tool::Eraser => self.set_pixel(idx, px, py, false),
                    Tool::Picker => {
                        if let Some(c) = self.pick_pixel(idx, px, py) {
                            self.spr_color = c;
                        }
                    }
                    Tool::Fill => {
                        self.flood_fill(idx, px, py);
                    }
                    Tool::Line | Tool::Rect => {
                        if self.spr_drag.is_none() {
                            self.spr_drag = Some((px, py));
                        }
                    }
                }
            }
            if !primary {
                if let Some((x0, y0)) = self.spr_drag.take() {
                    if self.spr_tool == Tool::Line {
                        self.draw_line(idx, x0, y0, px, py);
                    } else if self.spr_tool == Tool::Rect {
                        self.draw_rect(idx, x0, y0, px, py);
                    }
                }
            }
        }
    }

    fn set_pixel(&mut self, idx: usize, x: i32, y: i32, on: bool) {
        let w = self.project.sprites[idx].width as i32;
        let h = self.project.sprites[idx].height as i32;
        if x < 0 || y < 0 || x >= w || y >= h {
            return;
        }
        let [r, g, b] = self.spr_color;
        let frame = self.spr_frame;
        let f = self.frames_mut(idx);
        if let Some(buf) = f.get_mut(frame) {
            let i = ((y * w + x) * 4) as usize;
            buf[i] = r;
            buf[i + 1] = g;
            buf[i + 2] = b;
            buf[i + 3] = if on { 255 } else { 0 };
        }
        self.sync_frames(idx);
    }

    fn pick_pixel(&mut self, idx: usize, x: i32, y: i32) -> Option<[u8; 3]> {
        let w = self.project.sprites[idx].width as i32;
        let fr = self.spr_frame;
        let f = self.frames(idx);
        let buf = f.get(fr)?;
        let i = ((y * w + x) * 4) as usize;
        if i + 2 < buf.len() {
            Some([buf[i], buf[i + 1], buf[i + 2]])
        } else {
            None
        }
    }

    fn flood_fill(&mut self, idx: usize, x: i32, y: i32) {
        let (w, h) = (
            self.project.sprites[idx].width as i32,
            self.project.sprites[idx].height as i32,
        );
        let frame = self.spr_frame;
        let [r, g, b] = self.spr_color;
        // target = exact RGBA color at the click point
        let target: [u8; 4] = {
            let f = self.frames(idx);
            let Some(buf) = f.get(frame) else { return };
            if x < 0 || y < 0 || x >= w || y >= h {
                return;
            }
            let i = ((y * w + x) * 4) as usize;
            if i + 3 >= buf.len() {
                return;
            }
            [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
        };
        // filling with the same color would be a no-op anyway
        if target[0] == r && target[1] == g && target[2] == b && target[3] == 255 {
            return;
        }
        let mut stack = vec![(x, y)];
        let mut visited = std::collections::HashSet::new();
        let f = self.frames_mut(idx);
        let buf = match f.get_mut(frame) {
            Some(bb) => bb,
            None => return,
        };
        while let Some((cx, cy)) = stack.pop() {
            if cx < 0 || cy < 0 || cx >= w || cy >= h || !visited.insert((cx, cy)) {
                continue;
            }
            let i = ((cy * w + cx) * 4) as usize;
            if i + 3 >= buf.len() {
                continue;
            }
            // exact RGBA match (transparency counts as a color)
            let matches = buf[i] == target[0]
                && buf[i + 1] == target[1]
                && buf[i + 2] == target[2]
                && buf[i + 3] == target[3];
            if !matches {
                continue;
            }
            buf[i] = r;
            buf[i + 1] = g;
            buf[i + 2] = b;
            buf[i + 3] = 255;
            stack.extend_from_slice(&[(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)]);
        }
        self.sync_frames(idx);
    }

    fn draw_line(&mut self, idx: usize, x0: i32, y0: i32, x1: i32, y1: i32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let (sx, sy) = ((x0 < x1) as i32 * 2 - 1, (y0 < y1) as i32 * 2 - 1);
        let (mut err, mut x, mut y) = (dx + dy, x0, y0);
        loop {
            self.set_pixel(idx, x, y, true);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_rect(&mut self, idx: usize, x0: i32, y0: i32, x1: i32, y1: i32) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set_pixel(idx, x, y0, true);
            self.set_pixel(idx, x, y1, true);
        }
        for y in y0.min(y1)..=y0.max(y1) {
            self.set_pixel(idx, x0, y, true);
            self.set_pixel(idx, x1, y, true);
        }
    }

    // ------------------------------------------------ sound editor

    pub(crate) fn sound_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.heading(format!(
            "Sound properties: {}",
            self.project.sounds[idx].name
        ));
        ui.separator();
        let has_data = !self.project.sounds[idx].data.is_empty();
        ui.horizontal(|ui| {
            if ui.button("Load sound (wav/ogg)...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("audio", &["wav", "ogg", "mp3"])
                    .pick_file()
                {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let kind = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("wav")
                                .to_lowercase();
                            self.project.sounds[idx].kind = kind;
                            self.project.sounds[idx].data = encode_rgba(&bytes);
                            self.log(
                                0,
                                format!("imported {} ({} bytes)", path.display(), bytes.len()),
                            );
                        }
                        Err(e) => self.log(1, format!("cannot read: {}", e)),
                    }
                }
            }
            if has_data && ui.button("Play preview").clicked() {
                self.play_sound_preview(idx);
            }
            if has_data && ui.button("Stop").clicked() {
                self.stop_preview();
            }
        });
        if has_data {
            let bytes = decode_rgba(&self.project.sounds[idx].data);
            ui.label(format!(
                "Format: {}  ({} bytes)",
                self.project.sounds[idx].kind,
                bytes.len()
            ));
            ui.label("Use in GML: sound_play(idx), sound_loop(idx), sound_stop(idx)");
        } else {
            ui.weak("No audio data loaded yet.");
        }
    }

    // ------------------------------------------------ background editor

    pub(crate) fn background_editor(&mut self, ui: &mut egui::Ui, idx: usize, ctx: &egui::Context) {
        ui.heading(format!(
            "Background properties: {}",
            self.project.backgrounds[idx].name
        ));
        ui.separator();
        if ui.button("Load image (png/jpg/bmp)...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("image", &["png", "jpg", "bmp"])
                .pick_file()
            {
                match import_image(&path) {
                    Some((w, h, rgba)) => {
                        let bg = &mut self.project.backgrounds[idx];
                        bg.width = w;
                        bg.height = h;
                        bg.data = encode_rgba(&rgba);
                        self.log(0, format!("imported {} ({}x{})", path.display(), w, h));
                    }
                    None => self.log(1, "cannot import image"),
                }
            }
        }
        let (bw, bh, bdata) = {
            let bg = &self.project.backgrounds[idx];
            (bg.width, bg.height, decode_rgba(&bg.data))
        };
        let tex = self.image_texture(ctx, &format!("bg_{}", idx), &bdata, bw, bh);
        theme::sunken_frame().show(ui, |ui| {
            ui.image((tex, egui::vec2(bw as f32, bh as f32)));
        });
        ui.weak("Used in room backgrounds and as tilesets (room editor, Tiles mode).");
    }

    // ------------------------------------------------ path editor

    pub(crate) fn path_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.heading(format!("Path properties: {}", self.project.paths[idx].name));
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.project.paths[idx].closed, "Closed path");
            ui.label("grid:");
            let mut g = self.path_grid as i32;
            ui.add(egui::DragValue::new(&mut g).range(4..=128));
            self.path_grid = g as f64;
            ui.separator();
            ui.label(format!("{} points", self.project.paths[idx].points.len()));
            if ui.button("Clear points").clicked() {
                self.project.paths[idx].points.clear();
            }
            if ui.button("Reverse").clicked() {
                self.project.paths[idx].points.reverse();
            }
        });
        ui.separator();
        let zoom = 1.0f32;
        let rw = self
            .project
            .rooms
            .iter()
            .map(|r| r.width)
            .fold(640.0, f64::max);
        let canvas = egui::vec2(rw as f32 * zoom, 480.0);
        let (rect, response) = ui.allocate_exact_size(canvas, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, theme::WINDOW_BG);
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        );
        let g = self.path_grid as f32;
        let grid = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25));
        let mut x = rect.left() + g;
        while x < rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                grid,
            );
            x += g;
        }
        let mut y = rect.top() + g;
        while y < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                grid,
            );
            y += g;
        }
        let pts: Vec<egui::Pos2> = self.project.paths[idx]
            .points
            .iter()
            .map(|p| rect.min + egui::vec2(p.x as f32, p.y as f32))
            .collect();
        let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0x2A, 0x5A, 0xD8));
        for w in pts.windows(2) {
            painter.line_segment([w[0], w[1]], stroke);
        }
        if self.project.paths[idx].closed && pts.len() > 2 {
            painter.line_segment([*pts.last().unwrap(), pts[0]], stroke);
        }
        for (i, p) in pts.iter().enumerate() {
            painter.circle_filled(*p, 4.0, egui::Color32::from_rgb(230, 160, 30));
            painter.text(
                *p + egui::vec2(6.0, -6.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{}", i),
                egui::FontId::monospace(10.0),
                egui::Color32::from_rgb(0x00, 0x60, 0x00),
            );
        }
        if let Some(p) = response.interact_pointer_pos() {
            let rx = p.x - rect.left();
            let ry = p.y - rect.top();
            if response.clicked_by(egui::PointerButton::Primary) {
                let g = self.path_grid;
                let sx = (rx as f64 / g).round() * g;
                let sy = (ry as f64 / g).round() * g;
                self.project.paths[idx].points.push(PathPoint {
                    x: sx,
                    y: sy,
                    speed: 100.0,
                });
            }
            if response.clicked_by(egui::PointerButton::Secondary) {
                let mut best: Option<(f64, usize)> = None;
                for (i, pt) in self.project.paths[idx].points.iter().enumerate() {
                    let d = (pt.x - rx as f64).hypot(pt.y - ry as f64);
                    if best.map(|(bd, _)| d < bd).unwrap_or(true) {
                        best = Some((d, i));
                    }
                }
                if let Some((d, i)) = best {
                    if d < 24.0 {
                        self.project.paths[idx].points.remove(i);
                    }
                }
            }
        }
        ui.separator();
        ui.strong("Points (speed % = traversal cost of the segment)");
        egui::ScrollArea::vertical()
            .max_height(140.0)
            .show(ui, |ui| {
                for (i, pt) in self.project.paths[idx].points.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.monospace(format!("{:2}: ({:4},{:4})", i, pt.x as i32, pt.y as i32));
                        ui.add(egui::DragValue::new(&mut pt.speed).range(1..=1000).speed(1));
                    });
                }
            });
    }

    // ------------------------------------------------ script editor

    pub(crate) fn script_editor(&mut self, ui: &mut egui::Ui, idx: usize, db: &NameDb) {
        ui.heading(format!("Script: {}", self.project.scripts[idx].name));
        ui.separator();
        let mut buf: String = self.project.scripts[idx].code.clone();
        let cs = self.comp(&format!("script_{}", idx));
        let resp = code_editor(
            ui,
            &mut buf,
            &format!("script_{}", idx),
            ui.available_height() - 8.0,
            db,
            cs,
        );
        if resp.changed() {
            self.project.scripts[idx].code = buf;
        }
    }

    // ------------------------------------------------ timeline editor

    pub(crate) fn timeline_editor(&mut self, ui: &mut egui::Ui, idx: usize, db: &NameDb) {
        ui.heading(format!("Timeline: {}", self.project.timelines[idx].name));
        ui.label(
            "Moments run on instances with timeline_index = this and timeline_running = true.",
        );
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Add moment").clicked() {
                let next = self.project.timelines[idx]
                    .moments
                    .last()
                    .map(|m| m.step + 30)
                    .unwrap_or(30);
                self.project.timelines[idx].moments.push(TimelineMoment {
                    step: next,
                    code: String::new(),
                });
            }
            if ui.button("Sort by step").clicked() {
                self.project.timelines[idx].moments.sort_by_key(|m| m.step);
            }
        });
        ui.separator();
        let mut remove = None;
        let moments = self.project.timelines[idx].moments.clone();
        for (mi, m) in moments.iter().enumerate() {
            egui::CollapsingHeader::new(format!("Step {}", m.step))
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("step:");
                        let mut step = self.project.timelines[idx].moments[mi].step;
                        ui.add(egui::DragValue::new(&mut step).range(0..=1_000_000));
                        self.project.timelines[idx].moments[mi].step = step;
                        if ui.button("Remove moment").clicked() {
                            remove = Some(mi);
                        }
                    });
                    let mut buf = self.project.timelines[idx].moments[mi].code.clone();
                    let cs = self.comp(&format!("tl_{}_{}", idx, mi));
                    let resp =
                        code_editor(ui, &mut buf, &format!("tl_{}_{}", idx, mi), 200.0, db, cs);
                    if resp.changed() {
                        self.project.timelines[idx].moments[mi].code = buf;
                    }
                });
        }
        if let Some(mi) = remove {
            self.project.timelines[idx].moments.remove(mi);
        }
    }

    // ------------------------------------------------ font editor

    pub(crate) fn font_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.heading(format!("Font: {}", self.project.fonts[idx].name));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Size:");
            let mut size = self.project.fonts[idx].size;
            ui.add(egui::DragValue::new(&mut size).range(4..=200));
            self.project.fonts[idx].size = size;
            if ui.button("Load TTF...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("font", &["ttf", "otf"])
                    .pick_file()
                {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            self.project.fonts[idx].data = encode_rgba(&bytes);
                            self.log(
                                0,
                                format!("imported {} ({} bytes)", path.display(), bytes.len()),
                            );
                        }
                        Err(e) => self.log(1, format!("cannot read: {}", e)),
                    }
                }
            }
            if !self.project.fonts[idx].data.is_empty() && ui.button("Use built-in").clicked() {
                self.project.fonts[idx].data.clear();
            }
        });
        let is_custom = !self.project.fonts[idx].data.is_empty();
        ui.label(if is_custom {
            "Custom TTF loaded. In GML: draw_set_font(index) — string_width respects it."
        } else {
            "Built-in font at the chosen size."
        });
        ui.separator();
        ui.label("Preview:");
        let mut preview = String::from("The quick brown fox jumps over the lazy dog 0123456789");
        ui.text_edit_singleline(&mut preview);
    }

    // ------------------------------------------------ object editor

    pub(crate) fn object_editor(&mut self, ui: &mut egui::Ui, idx: usize, db: &NameDb) {
        let sprite_names: Vec<String> = self
            .project
            .sprites
            .iter()
            .map(|s| s.name.clone())
            .collect();
        let object_names: Vec<String> = self
            .project
            .objects
            .iter()
            .map(|o| o.name.clone())
            .collect();

        ui.heading(format!(
            "Object properties: {}",
            self.project.objects[idx].name
        ));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Sprite:");
            let cur = {
                let si = self.project.objects[idx].sprite_index;
                if si >= 0 && (si as usize) < sprite_names.len() {
                    sprite_names[si as usize].clone()
                } else {
                    "<no sprite>".into()
                }
            };
            ui.menu_button(cur, |ui| {
                if ui.button("<no sprite>").clicked() {
                    self.project.objects[idx].sprite_index = -1;
                }
                for (si, name) in sprite_names.iter().enumerate() {
                    if ui.button(name).clicked() {
                        self.project.objects[idx].sprite_index = si as i64;
                    }
                }
            });
            ui.checkbox(&mut self.project.objects[idx].visible, "Visible");
            ui.checkbox(&mut self.project.objects[idx].solid, "Solid");
            ui.checkbox(&mut self.project.objects[idx].persistent, "Persistent");
            ui.label("Depth:");
            ui.add(egui::DragValue::new(&mut self.project.objects[idx].depth));
        });
        ui.horizontal(|ui| {
            ui.label("Mask:");
            let mcur = {
                let mi = self.project.objects[idx].mask_index;
                if mi >= 0 && (mi as usize) < sprite_names.len() {
                    sprite_names[mi as usize].clone()
                } else {
                    "<same as sprite>".into()
                }
            };
            ui.menu_button(mcur, |ui| {
                if ui.button("<same as sprite>").clicked() {
                    self.project.objects[idx].mask_index = -1;
                }
                for (si, name) in sprite_names.iter().enumerate() {
                    if ui.button(name).clicked() {
                        self.project.objects[idx].mask_index = si as i64;
                    }
                }
            });
            ui.label("Parent:");
            let pcur = {
                let pi = self.project.objects[idx].parent;
                if pi >= 0 && (pi as usize) < object_names.len() {
                    object_names[pi as usize].clone()
                } else {
                    "<no parent>".into()
                }
            };
            ui.menu_button(pcur, |ui| {
                if ui.button("<no parent>").clicked() {
                    self.project.objects[idx].parent = -1;
                }
                for (oi, name) in object_names.iter().enumerate() {
                    if oi != idx && ui.button(name).clicked() {
                        self.project.objects[idx].parent = oi as i64;
                    }
                }
            });
            if ui.button("Add event...").clicked() {
                self.add_event_for = Some(idx);
            }
        });
        ui.separator();

        let mut selected = *self.sel_event.entry(idx).or_insert(0);
        let mut remove: Option<usize> = None;
        let labels: Vec<String> = self.project.objects[idx]
            .events
            .iter()
            .map(|e| e.label(&self.project.objects))
            .collect();
        if selected >= labels.len() {
            selected = labels.len().saturating_sub(1);
        }
        let mut pick = None;
        egui::SidePanel::left("events")
            .resizable(true)
            .default_width(180.0)
            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(2.0))
            .show_inside(ui, |ui| {
                ui.strong("Events");
                theme::sunken_frame().show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            let is_sel = selected == i;
                            let row = crate::icons::icon_and_text(ui, Icon::Script, label, is_sel);
                            if row.clicked() {
                                pick = Some(i);
                            }
                            row.context_menu(|ui| {
                                if ui.button("Delete event").clicked() {
                                    remove = Some(i);
                                    ui.close_menu();
                                }
                            });
                        }
                    });
                });
            });
        if let Some(p) = pick {
            selected = p;
        }
        if let Some(r) = remove {
            self.project.objects[idx].events.remove(r);
            if selected >= self.project.objects[idx].events.len() {
                selected = self.project.objects[idx].events.len().saturating_sub(1);
            }
        }
        self.sel_event.insert(idx, selected);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::PANEL_BG))
            .show_inside(ui, |ui| {
                let ev_count = self.project.objects[idx].events.len();
                let sel = *self.sel_event.entry(idx).or_insert(0);
                if ev_count == 0 {
                    ui.label("No events yet — click \u{201c}Add event...\u{201d} above.");
                    return;
                }
                if sel >= ev_count {
                    return;
                }
                match &self.project.objects[idx].events[sel] {
                    EventDef::Actions { .. } => {
                        // GM8 layout: actions list | library palette
                        if self.dnd_open_for != Some((idx, sel)) {
                            self.dnd_sel_action = None;
                            self.dnd_open_for = Some((idx, sel));
                        }
                        egui::SidePanel::left("dnd_actions")
                            .resizable(true)
                            .default_width(280.0)
                            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(2.0))
                            .show_inside(ui, |ui| {
                                crate::dndui::action_list(
                                    ui,
                                    &mut self.project,
                                    idx,
                                    sel,
                                    &mut self.dnd,
                                    &mut self.dnd_sel_action,
                                );
                            });
                        egui::CentralPanel::default()
                            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(2.0))
                            .show_inside(ui, |ui| {
                                crate::dndui::palette(
                                    ui,
                                    &mut self.dnd,
                                    &mut self.project,
                                    idx,
                                    sel,
                                );
                            });
                        return;
                    }
                    _ => {}
                }
                let label = self.project.objects[idx].events[sel].label(&self.project.objects);
                ui.label(egui::RichText::new(label).strong());
                let mut buf: String = self.project.objects[idx].events[sel].code().to_string();
                let cs = self.comp(&format!("obj_{}_ev_{}", idx, sel));
                let resp = code_editor(
                    ui,
                    &mut buf,
                    &format!("obj_{}_ev_{}", idx, sel),
                    ui.available_height() - 8.0,
                    db,
                    cs,
                );
                if resp.changed() {
                    self.project.objects[idx].events[sel]
                        .code_mut()
                        .clone_from(&buf);
                }
            });
    }

    // ------------------------------------------------ room editor

    pub(crate) fn room_editor(&mut self, ui: &mut egui::Ui, ri: usize, ctx: &egui::Context) {
        ui.heading(format!("Room properties: {}", self.project.rooms[ri].name));
        ui.separator();
        ui.horizontal(|ui| {
            let room = &mut self.project.rooms[ri];
            ui.label("Size:");
            let mut wd = room.width as i32;
            let mut ht = room.height as i32;
            ui.add(egui::DragValue::new(&mut wd).range(64..=8000));
            ui.add(egui::DragValue::new(&mut ht).range(64..=8000));
            room.width = wd as f64;
            room.height = ht as f64;
            ui.label("Speed:");
            let mut sp = room.speed as i32;
            ui.add(egui::DragValue::new(&mut sp).range(1..=1000));
            room.speed = sp as f64;
            ui.separator();
            ui.label("Back color:");
            let mut rgb = [
                (room.back_color & 0xFF) as f32 / 255.0,
                ((room.back_color >> 8) & 0xFF) as f32 / 255.0,
                ((room.back_color >> 16) & 0xFF) as f32 / 255.0,
            ];
            ui.color_edit_button_rgb(&mut rgb);
            room.back_color = (rgb[0] * 255.0) as u32
                | (((rgb[1] * 255.0) as u32) << 8)
                | (((rgb[2] * 255.0) as u32) << 16);
            ui.checkbox(&mut room.show_color, "show color");
        });
        // views
        ui.horizontal(|ui| {
            ui.strong("Views:");
            for vi in 0..8 {
                let label = format!("{}", vi);
                let visible = self.project.rooms[ri]
                    .views
                    .get(vi)
                    .map(|v| v.visible)
                    .unwrap_or(false);
                let rich = if visible {
                    egui::RichText::new(label)
                        .strong()
                        .color(egui::Color32::from_rgb(0x1E, 0x5E, 0xC0))
                } else {
                    egui::RichText::new(label)
                };
                if ui.selectable_label(visible, rich).clicked() {
                    let room = &mut self.project.rooms[ri];
                    while room.views.len() <= vi {
                        room.views.push(RoomView {
                            visible: false,
                            x: 0.0,
                            y: 0.0,
                            w: room.width.min(640.0),
                            h: room.height.min(480.0),
                            object: -1,
                            hborder: 32.0,
                            vborder: 32.0,
                            hspeed: -1.0,
                            vspeed: -1.0,
                        });
                    }
                    room.views[vi].visible = !room.views[vi].visible;
                }
            }
            if ui.button("clear").clicked() {
                self.project.rooms[ri].views.clear();
            }
        });
        if let Some(vi) = self.project.rooms[ri].views.iter().rposition(|v| v.visible) {
            ui.horizontal(|ui| {
                let object_names: Vec<String> = std::iter::once("<none>".to_string())
                    .chain(self.project.objects.iter().map(|o| o.name.clone()))
                    .collect();
                {
                    let room = &mut self.project.rooms[ri];
                    let v = &mut room.views[vi];
                    ui.label(format!("view {}:", vi));
                    ui.add(egui::DragValue::new(&mut v.x).range(0..=8000))
                        .on_hover_text("X");
                    ui.add(egui::DragValue::new(&mut v.y).range(0..=8000))
                        .on_hover_text("Y");
                    ui.add(egui::DragValue::new(&mut v.w).range(16..=8000))
                        .on_hover_text("W");
                    ui.add(egui::DragValue::new(&mut v.h).range(16..=8000))
                        .on_hover_text("H");
                    ui.add(egui::DragValue::new(&mut v.hborder).range(0..=2000))
                        .on_hover_text("hborder");
                    ui.add(egui::DragValue::new(&mut v.vborder).range(0..=2000))
                        .on_hover_text("vborder");
                }
                ui.label("follow:");
                let cur = {
                    let v = &self.project.rooms[ri].views[vi];
                    if v.object >= 0 && ((v.object + 1) as usize) < object_names.len() {
                        object_names[(v.object + 1) as usize].clone()
                    } else {
                        "<none>".to_string()
                    }
                };
                let r = ui
                    .menu_button(cur, |ui| {
                        if ui.button("<none>").clicked() {
                            self.project.rooms[ri].views[vi].object = -1;
                        }
                        for (oi, name) in object_names.iter().enumerate().skip(1) {
                            if ui.button(name).clicked() {
                                self.project.rooms[ri].views[vi].object = (oi - 1) as i64;
                            }
                        }
                    })
                    .response;
                r.on_hover_text("object to follow");
            });
        }
        ui.separator();

        egui::SidePanel::left("room_objects")
            .resizable(true)
            .default_width(190.0)
            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(2.0))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(!self.room_mode_tiles, "Instances")
                        .clicked()
                    {
                        self.room_mode_tiles = false;
                    }
                    if ui.selectable_label(self.room_mode_tiles, "Tiles").clicked() {
                        self.room_mode_tiles = true;
                    }
                });
                ui.separator();
                if !self.room_mode_tiles {
                    ui.strong("Objects (LMB place / RMB delete)");
                    theme::sunken_frame().show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let obj_names: Vec<String> = self
                                .project
                                .objects
                                .iter()
                                .map(|o| o.name.clone())
                                .collect();
                            for (oi, o) in obj_names.iter().enumerate() {
                                let selected = self.room_sel_obj == oi as i64;
                                let row = match self.object_thumb(ctx, oi) {
                                    Some((tex, tw, th)) => crate::icons::thumb_and_text(
                                        ui,
                                        tex,
                                        (tw, th),
                                        o,
                                        selected,
                                        None,
                                    ),
                                    None => crate::icons::thumb_and_text(
                                        ui,
                                        egui::TextureId::default(),
                                        (0.0, 0.0),
                                        o,
                                        selected,
                                        Some(crate::object_color(oi)),
                                    ),
                                };
                                if row.clicked() {
                                    self.room_sel_obj = oi as i64;
                                }
                            }
                        });
                    });
                } else {
                    ui.strong("Tileset");
                    let bg_names: Vec<String> = self
                        .project
                        .backgrounds
                        .iter()
                        .map(|b| b.name.clone())
                        .collect();
                    let cur = if self.room_tile_bg >= 0
                        && (self.room_tile_bg as usize) < bg_names.len()
                    {
                        bg_names[self.room_tile_bg as usize].clone()
                    } else {
                        "<none>".into()
                    };
                    ui.menu_button(cur, |ui| {
                        if ui.button("<none>").clicked() {
                            self.room_tile_bg = -1;
                        }
                        for (i, n) in bg_names.iter().enumerate() {
                            if ui.button(n).clicked() {
                                self.room_tile_bg = i as i64;
                            }
                        }
                    })
                    .response
                    .on_hover_text("tileset background");
                    ui.label("tile size:");
                    let mut ts = self.room_tile_size;
                    ui.add(egui::DragValue::new(&mut ts).range(4..=256));
                    self.room_tile_size = ts;
                    ui.label("depth:");
                    let mut td = self.room_tile_depth;
                    ui.add(egui::DragValue::new(&mut td));
                    self.room_tile_depth = td;
                    ui.label(format!(
                        "tiles: {} (LMB place / RMB delete)",
                        self.project.rooms[ri].tiles.len()
                    ));
                    if ui.button("clear all tiles").clicked() {
                        self.project.rooms[ri].tiles.clear();
                    }
                }
                ui.separator();
                ui.strong("Room backgrounds");
                let nbs = self.project.rooms[ri].backgrounds.len();
                let mut remove_layer = None;
                for bi in 0..nbs {
                    let bg_names: Vec<String> = self
                        .project
                        .backgrounds
                        .iter()
                        .map(|b| b.name.clone())
                        .collect();
                    let cur = {
                        let idx = self.project.rooms[ri].backgrounds[bi].index;
                        if idx >= 0 && (idx as usize) < bg_names.len() {
                            bg_names[idx as usize].clone()
                        } else {
                            "<none>".into()
                        }
                    };
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.project.rooms[ri].backgrounds[bi].visible, "");
                        let r = ui
                            .menu_button(cur, |ui| {
                                if ui.button("<none>").clicked() {
                                    self.project.rooms[ri].backgrounds[bi].index = -1;
                                }
                                for (i, n) in bg_names.iter().enumerate() {
                                    if ui.button(n).clicked() {
                                        self.project.rooms[ri].backgrounds[bi].index = i as i64;
                                    }
                                }
                            })
                            .response;
                        r.on_hover_text("background image");
                        ui.checkbox(
                            &mut self.project.rooms[ri].backgrounds[bi].tiled_h,
                            "tile H",
                        );
                        ui.checkbox(
                            &mut self.project.rooms[ri].backgrounds[bi].tiled_v,
                            "tile V",
                        );
                        if icon_button(ui, Icon::Cross, false) {
                            remove_layer = Some(bi);
                        }
                    });
                }
                if let Some(bi) = remove_layer {
                    self.project.rooms[ri].backgrounds.remove(bi);
                }
                if ui.button("+ background layer").clicked() {
                    self.project.rooms[ri].backgrounds.push(RoomBackground {
                        visible: true,
                        index: -1,
                        x: 0.0,
                        y: 0.0,
                        tiled_h: true,
                        tiled_v: true,
                    });
                }
                ui.separator();
                ui.strong("Room creation code");
                let mut buf = self.project.rooms[ri].creation_code.clone();
                let db = NameDb::build(&self.project);
                let cs = self.comp(&format!("room_{}", ri));
                let resp = code_editor(ui, &mut buf, &format!("room_{}", ri), 140.0, &db, cs);
                if resp.changed() {
                    self.project.rooms[ri].creation_code = buf;
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::PANEL_BG))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("zoom:");
                    ui.add(egui::Slider::new(&mut self.room_zoom, 0.1..=3.0));
                    ui.label("snap:");
                    let mut snap = self.room_snap as i32;
                    ui.add(egui::DragValue::new(&mut snap).range(1..=256));
                    self.room_snap = snap as f64;
                });
                let zoom = self.room_zoom;
                let (rw, rh) = (self.project.rooms[ri].width, self.project.rooms[ri].height);
                let canvas = egui::vec2(rw as f32 * zoom, rh as f32 * zoom);
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(canvas, egui::Sense::click_and_drag());
                        // pinch / ctrl+wheel over the canvas zooms (plain wheel scrolls)
                        canvas_zoom(ui, rect, &mut self.room_zoom, 0.1, 3.0, false);
                        let painter = ui.painter_at(rect);
                        let bc = self.project.rooms[ri].back_color;
                        let col = egui::Color32::from_rgb(
                            (bc & 0xFF) as u8,
                            ((bc >> 8) & 0xFF) as u8,
                            ((bc >> 16) & 0xFF) as u8,
                        );
                        painter.rect_filled(rect, 0.0, col);
                        // background images
                        let bgs = self.project.rooms[ri].backgrounds.clone();
                        for bg in &bgs {
                            if !bg.visible
                                || bg.index < 0
                                || (bg.index as usize) >= self.project.backgrounds.len()
                            {
                                continue;
                            }
                            let (bw, bh, bdata) = {
                                let b = &self.project.backgrounds[bg.index as usize];
                                (b.width, b.height, decode_rgba(&b.data))
                            };
                            let tex = self.image_texture(
                                ctx,
                                &format!("roombg_{}", bg.index),
                                &bdata,
                                bw,
                                bh,
                            );
                            let tw = bw as f32 * zoom;
                            let th = bh as f32 * zoom;
                            let mut y = bg.y as f32 * zoom;
                            while y < rect.height() {
                                let mut x = bg.x as f32 * zoom;
                                while x < rect.width() {
                                    let drect = egui::Rect::from_min_size(
                                        rect.min + egui::vec2(x, y),
                                        egui::vec2(tw, th),
                                    );
                                    painter.image(
                                        tex,
                                        drect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                    if !bg.tiled_h {
                                        break;
                                    }
                                    x += tw;
                                }
                                if !bg.tiled_v {
                                    break;
                                }
                                y += th;
                            }
                        }
                        // grid
                        let snap = self.room_snap as f32 * zoom;
                        if snap > 4.0 {
                            let g = egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25),
                            );
                            let mut x = rect.left() + snap;
                            while x < rect.right() {
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    g,
                                );
                                x += snap;
                            }
                            let mut y = rect.top() + snap;
                            while y < rect.bottom() {
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                    g,
                                );
                                y += snap;
                            }
                        }
                        // tiles
                        let tiles = self.project.rooms[ri].tiles.clone();
                        let ctx2 = ui.ctx().clone();
                        for t in &tiles {
                            if t.bg >= 0 && (t.bg as usize) < self.project.backgrounds.len() {
                                let (bw, bh, bdata) = {
                                    let b = &self.project.backgrounds[t.bg as usize];
                                    (b.width, b.height, decode_rgba(&b.data))
                                };
                                let tex = self.image_texture(
                                    &ctx2,
                                    &format!("tset_{}", t.bg),
                                    &bdata,
                                    bw,
                                    bh,
                                );
                                let drect = egui::Rect::from_min_size(
                                    rect.min + egui::vec2(t.x as f32 * zoom, t.y as f32 * zoom),
                                    egui::vec2(t.width as f32 * zoom, t.height as f32 * zoom),
                                );
                                let uv = egui::Rect::from_min_size(
                                    egui::pos2(t.left as f32 / bw as f32, t.top as f32 / bh as f32),
                                    egui::vec2(
                                        t.width as f32 / bw as f32,
                                        t.height as f32 / bh as f32,
                                    ),
                                );
                                painter.image(tex, drect, uv, egui::Color32::WHITE);
                            }
                        }
                        // instances — drawn with their real sprites (origin-aligned,
                        // exactly like the engine's default draw)
                        let instances = self.project.rooms[ri].instances.clone();
                        for inst in &instances {
                            let Some(obj) = self.project.objects.get(inst.object) else {
                                continue;
                            };
                            let spr_idx = obj.sprite_index;
                            let has_spr = spr_idx >= 0
                                && (spr_idx as usize) < self.project.sprites.len()
                                && !self.project.sprites[spr_idx as usize].frames.is_empty();
                            if has_spr {
                                let su = spr_idx as usize;
                                let (sw, sh, xoff, yoff, data) = {
                                    let sp = &self.project.sprites[su];
                                    (
                                        sp.width,
                                        sp.height,
                                        sp.xoffset,
                                        sp.yoffset,
                                        self.frames(su).first().cloned().unwrap_or_default(),
                                    )
                                };
                                let tex = self.texture_static(
                                    &ctx2,
                                    &format!("thumb_spr_{}", su),
                                    &data,
                                    sw,
                                    sh,
                                );
                                let drect = egui::Rect::from_min_size(
                                    rect.min
                                        + egui::vec2(
                                            (inst.x - xoff as f64) as f32 * zoom,
                                            (inst.y - yoff as f64) as f32 * zoom,
                                        ),
                                    egui::vec2(sw as f32 * zoom, sh as f32 * zoom),
                                );
                                painter.image(
                                    tex,
                                    drect,
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                            } else {
                                let col = object_color(inst.object);
                                let p = rect.min
                                    + egui::vec2(inst.x as f32 * zoom, inst.y as f32 * zoom);
                                let half = egui::vec2(8.0 * zoom, 8.0 * zoom);
                                painter.rect_filled(
                                    egui::Rect::from_min_max(p - half, p + half),
                                    2.0,
                                    col,
                                );
                                if zoom > 0.5 {
                                    painter.text(
                                        p,
                                        egui::Align2::CENTER_CENTER,
                                        obj.name.chars().take(3).collect::<String>(),
                                        egui::FontId::monospace(9.0),
                                        egui::Color32::BLACK,
                                    );
                                }
                            }
                        }
                        // interactions
                        if let Some(p) = response.interact_pointer_pos() {
                            let rx = (p.x - rect.left()) / zoom;
                            let ry = (p.y - rect.top()) / zoom;
                            if self.room_mode_tiles {
                                if response.clicked_by(egui::PointerButton::Primary)
                                    && self.room_tile_bg >= 0
                                {
                                    let ts = self.room_tile_size as f32;
                                    let tx = ((rx / ts).floor() * ts) as f64;
                                    let ty = ((ry / ts).floor() * ts) as f64;
                                    self.project.rooms[ri].tiles.push(RoomTile {
                                        bg: self.room_tile_bg,
                                        left: 0,
                                        top: 0,
                                        width: self.room_tile_size,
                                        height: self.room_tile_size,
                                        x: tx,
                                        y: ty,
                                        depth: self.room_tile_depth,
                                        id: 0,
                                    });
                                }
                                if response.clicked_by(egui::PointerButton::Secondary) {
                                    let ts = self.room_tile_size as f32;
                                    let tx = ((rx / ts).floor() * ts) as f64;
                                    let ty = ((ry / ts).floor() * ts) as f64;
                                    self.project.rooms[ri].tiles.retain(|t| {
                                        !(t.x == tx && t.y == ty && t.depth == self.room_tile_depth)
                                    });
                                }
                            } else {
                                if response.clicked_by(egui::PointerButton::Primary)
                                    && self.room_sel_obj >= 0
                                    && (self.room_sel_obj as usize) < self.project.objects.len()
                                {
                                    let snap = self.room_snap as f32;
                                    let sx = ((rx / snap).round() * snap) as f64;
                                    let sy = ((ry / snap).round() * snap) as f64;
                                    self.project.rooms[ri].instances.push(RoomInstance {
                                        x: sx,
                                        y: sy,
                                        object: self.room_sel_obj as usize,
                                    });
                                }
                                if response.clicked_by(egui::PointerButton::Secondary) {
                                    let mut hit = None;
                                    for (i, inst) in
                                        self.project.rooms[ri].instances.iter().enumerate().rev()
                                    {
                                        if (inst.x - rx as f64).abs() <= 24.0
                                            && (inst.y - ry as f64).abs() <= 24.0
                                        {
                                            hit = Some(i);
                                            break;
                                        }
                                    }
                                    if let Some(i) = hit {
                                        self.project.rooms[ri].instances.remove(i);
                                    }
                                }
                            }
                        }
                    });
            });
    }
}

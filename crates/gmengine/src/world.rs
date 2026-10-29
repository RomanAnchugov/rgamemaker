use crate::backend::Backend;
use crate::instance::*;
use crate::particles::Particles;
use gmformat::Project;
use gml::{Program, Value};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub const FONT_SIZE: f32 = 16.0;

#[derive(Clone)]
pub struct ViewState {
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub hborder: f64,
    pub vborder: f64,
    pub hspeed: f64,
    pub vspeed: f64,
    pub object: i64,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            visible: false,
            x: 0.0,
            y: 0.0,
            w: 640.0,
            h: 480.0,
            hborder: 32.0,
            vborder: 32.0,
            hspeed: -1.0,
            vspeed: -1.0,
            object: -1,
        }
    }
}

pub struct FontState {
    pub backend_id: u32,
    pub size: f64,
}

enum PendingRoom {
    Goto(usize),
    Next,
    Previous,
    Restart,
}

#[derive(Clone)]
pub struct SoundState {
    pub backend_id: u32,
    pub volume: f64,
}

/// The complete runtime game: resources + instances + input + loop.
pub struct Game {
    pub project: Project,
    pub backend: Box<dyn Backend>,
    pub sprites: Vec<SpriteData>,
    pub(crate) scripts: HashMap<String, Rc<Program>>,
    obj_events: Vec<HashMap<EventKey, Rc<Program>>>,
    pub(crate) slots: Vec<Option<Instance>>,
    free: Vec<usize>,
    id_to_slot: HashMap<i64, usize>,
    pub(crate) next_id: i64,
    pub globals: HashMap<String, Value>,
    pub score: f64,
    pub lives: f64,
    pub health: f64,
    pub draw_color: u32,
    pub draw_alpha: f64,
    pub draw_font: usize,
    pub draw_halign: i64,
    pub draw_valign: i64,
    pub view_enabled: bool,
    pub view_current: usize,
    pub views: Vec<ViewState>,
    pub paths: Vec<PathData>,
    pub timelines: Vec<Vec<(i64, Rc<Program>)>>,
    pub fonts: Vec<FontState>,
    pub tiles: Vec<RTile>,
    pub next_tile_id: i64,
    pub particles: Particles,
    pub highscore: Vec<(String, f64)>,
    pub debug_log: Vec<String>,
    keys_down: HashSet<i64>,
    pub(crate) keys_pressed: HashSet<i64>,
    pub(crate) keys_released: HashSet<i64>,
    pub mouse_sx: f64,
    pub mouse_sy: f64,
    pub mouse_bits: u8,
    pub mouse_pressed_bits: u8,
    pub mouse_released_bits: u8,
    pub wheel: i32,
    pub room_idx: usize,
    pub room_speed_val: f64,
    pending_room: Option<PendingRoom>,
    pub ended: bool,
    pub rng: u64,
    pub sprite_tex: Vec<Vec<u32>>,
    pub bg_tex: Vec<u32>,
    pub sounds: Vec<SoundState>,
    pub ds_counter: u64,
    pub ds_lists: HashMap<u64, Vec<Value>>,
    pub ds_maps: HashMap<u64, Vec<(Value, Value)>>,
    pub ds_stacks: HashMap<u64, Vec<Value>>,
    pub errors: Vec<String>,
    pub keyboard_string: String,
    pub last_char: String,
    pub last_vk: i64,
    pub fps: f64,
    pub instance_deactivated: usize,
    last_lives: f64,
    last_health: f64,
    fps_frames: u64,
    fps_ms: f64,
}

impl Game {
    pub fn new(project: Project, mut backend: Box<dyn Backend>) -> Result<Game, String> {
        // ---- compile scripts ----
        let mut scripts = HashMap::new();
        let mut errors = Vec::new();
        for s in &project.scripts {
            match gml::compile(&s.code) {
                Ok(p) => {
                    scripts.insert(s.name.to_ascii_lowercase(), Rc::new(p));
                }
                Err(e) => errors.push(format!("script '{}': {}", s.name, e)),
            }
        }
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }

        // ---- merge object events with parents ----
        let n = project.objects.len();
        let mut obj_events: Vec<HashMap<EventKey, Rc<Program>>> = vec![HashMap::new(); n];
        for i in 0..n {
            let mut merged: HashMap<EventKey, Rc<Program>> = HashMap::new();
            let mut chain: Vec<usize> = Vec::new();
            let mut cur = i as i64;
            let mut guard = 0;
            while cur >= 0 && (cur as usize) < n && guard < 256 {
                chain.push(cur as usize);
                cur = project.objects[cur as usize].parent;
                guard += 1;
            }
            for oi in chain.iter().rev() {
                for def in &project.objects[*oi].events {
                    let key = EventKey::from_def(def);
                    let src = match def {
                        gmformat::EventDef::Actions { actions, .. } => {
                            crate::dnd::compile_event(actions, &project.objects)
                        }
                        _ => Ok(def.code().to_string()),
                    };
                    match src.and_then(|code| gml::compile(&code).map_err(|e| e.to_string())) {
                        Ok(p) => {
                            merged.insert(key, Rc::new(p));
                        }
                        Err(e) => {
                            errors.push(format!(
                                "object '{}': {} in {}\n{}",
                                project.objects[*oi].name,
                                e,
                                def.label(&project.objects),
                                ""
                            ));
                        }
                    }
                }
            }
            obj_events[i] = merged;
        }
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }

        // ---- upload resources ----
        let mut sprites = Vec::new();
        let mut sprite_tex = Vec::new();
        for s in &project.sprites {
            let data = SpriteData::from_def(s);
            let mut texs = Vec::new();
            for f in 0..s.frames.len() {
                let rgba = s
                    .decode_frame(f)
                    .unwrap_or_else(|| vec![0; (s.width * s.height * 4) as usize]);
                texs.push(backend.load_image(
                    &format!("{}_{}", s.name, f),
                    &rgba,
                    s.width,
                    s.height,
                ));
            }
            sprites.push(data);
            sprite_tex.push(texs);
        }
        let mut bg_tex = Vec::new();
        for b in &project.backgrounds {
            let rgba = gmformat::decode_rgba(&b.data);
            bg_tex.push(backend.load_image(&b.name, &rgba, b.width, b.height));
        }
        let mut sounds = Vec::new();
        for s in &project.sounds {
            let bytes = gmformat::decode_rgba(&s.data);
            let id = backend.load_sound(&s.name, &bytes);
            sounds.push(SoundState {
                backend_id: id,
                volume: s.volume,
            });
        }

        // ---- compile paths & timelines, load fonts ----
        let paths: Vec<PathData> = project.paths.iter().map(PathData::from_def).collect();
        let mut timelines = Vec::new();
        for t in &project.timelines {
            let mut moments: Vec<(i64, Rc<Program>)> = Vec::new();
            for m in &t.moments {
                match gml::compile(&m.code) {
                    Ok(prog) => moments.push((m.step, Rc::new(prog))),
                    Err(e) => {
                        errors.push(format!("timeline '{}': moment {}: {}", t.name, m.step, e))
                    }
                }
            }
            moments.sort_by_key(|(step, _)| *step);
            timelines.push(moments);
        }
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        let mut fonts = Vec::new();
        for f in &project.fonts {
            let bytes = gmformat::decode_rgba(&f.data);
            let id = backend.load_font(&f.name, &bytes, f.size);
            fonts.push(FontState {
                backend_id: id,
                size: f.size,
            });
        }

        let room0 = project
            .rooms
            .first()
            .cloned()
            .unwrap_or_else(|| gmformat::Room::new("room0", 640, 480));
        let mut game = Game {
            draw_font: 0,
            draw_halign: 0,
            draw_valign: 0,
            view_enabled: false,
            view_current: 0,
            views: vec![ViewState::default()],
            paths,
            timelines,
            fonts,
            tiles: Vec::new(),
            next_tile_id: 1,
            particles: Particles::new(),
            highscore: Vec::new(),
            debug_log: Vec::new(),
            project,
            backend,
            sprites,
            scripts,
            obj_events,
            slots: Vec::new(),
            free: Vec::new(),
            id_to_slot: HashMap::new(),
            next_id: ID_BASE,
            globals: HashMap::new(),
            score: 0.0,
            lives: 0.0,
            health: 100.0,
            draw_color: 0xFFFFFF,
            draw_alpha: 1.0,
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_sx: 0.0,
            mouse_sy: 0.0,
            mouse_bits: 0,
            mouse_pressed_bits: 0,
            mouse_released_bits: 0,
            wheel: 0,
            room_idx: 0,
            room_speed_val: room0.speed,
            pending_room: None,
            ended: false,
            rng: 0x9E3779B97F4A7C15,
            sprite_tex,
            bg_tex,
            sounds,
            ds_counter: 0,
            ds_lists: HashMap::new(),
            ds_maps: HashMap::new(),
            ds_stacks: HashMap::new(),
            errors: Vec::new(),
            keyboard_string: String::new(),
            last_char: String::new(),
            last_vk: 0,
            fps: 60.0,
            instance_deactivated: 0,
            last_lives: 1.0,
            last_health: 100.0,
            fps_frames: 0,
            fps_ms: 0.0,
        };
        game.enter_room(0, true);
        Ok(game)
    }

    // ---------- instance management ----------

    pub fn alloc_slot_pub(&mut self, inst: Instance) -> i64 {
        self.alloc_slot(inst)
    }

    fn alloc_slot(&mut self, inst: Instance) -> i64 {
        let id = inst.id;
        if let Some(idx) = self.free.pop() {
            self.id_to_slot.insert(id, idx);
            self.slots[idx] = Some(inst);
        } else {
            self.slots.push(Some(inst));
            self.id_to_slot.insert(id, self.slots.len() - 1);
        }
        id
    }

    pub fn inst(&self, id: i64) -> Option<&Instance> {
        self.id_to_slot
            .get(&id)
            .and_then(|&i| self.slots[i].as_ref())
            .filter(|i| i.alive)
    }
    pub fn inst_mut(&mut self, id: i64) -> Option<&mut Instance> {
        match self.id_to_slot.get(&id).copied() {
            Some(i) => self.slots[i].as_mut().filter(|i| i.alive).map(|i| i),
            None => None,
        }
    }

    pub fn alive_ids(&self) -> Vec<i64> {
        let mut ids: Vec<i64> = self
            .slots
            .iter()
            .flatten()
            .filter(|i| i.alive && i.active)
            .map(|i| i.id)
            .collect();
        ids.sort();
        ids
    }

    pub(crate) fn sweep(&mut self) {
        let mut to_remove = Vec::new();
        for (idx, s) in self.slots.iter().enumerate() {
            if let Some(i) = s {
                if !i.alive {
                    to_remove.push((i.id, idx));
                }
            }
        }
        for (id, idx) in to_remove {
            self.id_to_slot.remove(&id);
            self.slots[idx] = None;
            self.free.push(idx);
        }
    }

    /// Create an instance and run its Create event (GM semantics).
    pub fn create_instance(&mut self, x: f64, y: f64, obj: usize) -> Result<i64, String> {
        if obj >= self.project.objects.len() {
            return Err("instance_create: no such object".into());
        }
        let id = self.next_id;
        self.next_id += 1;
        let o = &self.project.objects[obj];
        let inst = Instance::new(id, obj, x, y, o);
        self.alloc_slot(inst);
        self.run_event(id, &EventKey::Create, NOONE);
        Ok(id)
    }

    /// Destroy: run the Destroy event, then mark for removal.
    pub fn destroy_instance(&mut self, id: i64) {
        if self.inst(id).is_some() {
            self.run_event(id, &EventKey::Destroy, NOONE);
            if let Some(i) = self.inst_mut(id) {
                i.alive = false;
            }
        }
    }

    /// True when `actual` object is a (transitive) child of `spec`.
    pub fn obj_matches(&self, spec: usize, actual: usize) -> bool {
        let mut cur = actual as i64;
        let mut guard = 0;
        while cur >= 0 && (cur as usize) < self.project.objects.len() && guard < 256 {
            if cur as usize == spec {
                return true;
            }
            cur = self.project.objects[cur as usize].parent;
            guard += 1;
        }
        false
    }

    /// Resolve a GM target value (id / object / all / self / other / noone)
    /// into instance ids.
    pub fn ids_for_target(&self, target: i64, self_id: i64, other_id: i64) -> Vec<i64> {
        let mut out = Vec::new();
        match target {
            ALL => {
                out = self.alive_ids();
            }
            SELF => {
                if let Some(i) = self.inst(self_id) {
                    if i.active {
                        out.push(self_id);
                    }
                }
            }
            OTHER => {
                if let Some(i) = self.inst(other_id) {
                    if i.active {
                        out.push(other_id);
                    }
                }
            }
            NOONE => {}
            t if t >= ID_BASE => {
                if let Some(i) = self.inst(t) {
                    if i.active {
                        out.push(t);
                    }
                }
            }
            t => {
                for i in self.slots.iter().flatten() {
                    if i.alive && i.active && self.obj_matches(t as usize, i.obj) {
                        out.push(i.id);
                    }
                }
            }
        }
        out
    }

    // ---------- events ----------

    pub fn run_event(&mut self, id: i64, key: &EventKey, other: i64) {
        let obj = match self.inst(id) {
            Some(i) => i.obj,
            None => return,
        };
        let prog = match self.obj_events.get(obj).and_then(|m| m.get(key)) {
            Some(p) => p.clone(),
            None => return,
        };
        let mut it = gml::Interp::new(self);
        if let Err(e) = it.run(&prog, vec![], id, other) {
            let name = self
                .project
                .objects
                .get(obj)
                .map(|o| o.name.clone())
                .unwrap_or_default();
            self.errors
                .push(format!("ERROR in {} ({}): {}", name, event_label(key), e));
        }
    }

    pub fn has_event(&self, obj: usize, key: &EventKey) -> bool {
        self.obj_events
            .get(obj)
            .map(|m| m.contains_key(key))
            .unwrap_or(false)
    }

    pub fn find_collision_event(&self, obj: usize, other_obj: usize) -> Option<EventKey> {
        let map = self.obj_events.get(obj)?;
        for k in map.keys() {
            if let EventKey::Collision(spec) = k {
                if self.obj_matches(*spec, other_obj) {
                    return Some(k.clone());
                }
            }
        }
        None
    }

    /// Run a GML snippet for tests/debugging in the given self context.
    pub fn run_code(&mut self, code: &str, self_id: i64) -> Result<Value, String> {
        let prog = gml::compile(code)?;
        let mut it = gml::Interp::new(self);
        it.run(&prog, vec![], self_id, NOONE)
    }

    pub fn run_script_by_name(
        &mut self,
        name: &str,
        args: Vec<Value>,
        self_id: i64,
        other_id: i64,
    ) -> Result<Value, String> {
        let prog = self
            .scripts
            .get(&name.to_ascii_lowercase())
            .cloned()
            .ok_or_else(|| format!("unknown script '{}'", name))?;
        let mut it = gml::Interp::new(self);
        it.run(&prog, args, self_id, other_id)
    }

    // ---------- rooms ----------

    pub(crate) fn apply_room_settings(&mut self) {
        let room = &self.project.rooms[self.room_idx];
        self.room_speed_val = room.speed;
        // views: up to 8, from the room definition
        self.views.clear();
        for i in 0..8 {
            let def = room.views.get(i);
            let mut v = ViewState::default();
            v.w = room.width.min(640.0);
            v.h = room.height.min(480.0);
            if let Some(d) = def {
                v.visible = d.visible;
                v.x = d.x;
                v.y = d.y;
                v.w = d.w;
                v.h = d.h;
                v.hborder = d.hborder;
                v.vborder = d.vborder;
                v.hspeed = if d.hspeed == 0.0 { -1.0 } else { d.hspeed };
                v.vspeed = if d.vspeed == 0.0 { -1.0 } else { d.vspeed };
                v.object = d.object;
            }
            self.views.push(v);
        }
        self.view_enabled = room.views.iter().any(|v| v.visible);
        if !self.view_enabled {
            self.views[0].x = 0.0;
            self.views[0].y = 0.0;
            self.views[0].w = room.width;
            self.views[0].h = room.height;
            self.views[0].visible = true;
        }
        // tiles
        self.tiles.clear();
        let mut next_id = 1i64;
        for t in &room.tiles {
            self.tiles.push(RTile {
                id: if t.id > 0 { t.id } else { next_id },
                bg: t.bg,
                left: t.left,
                top: t.top,
                width: t.width,
                height: t.height,
                x: t.x,
                y: t.y,
                depth: t.depth,
                visible: true,
            });
            next_id = next_id.max(if t.id > 0 { t.id } else { next_id }) + 1;
        }
        self.next_tile_id = next_id;
    }

    /// The view used for translating mouse/world coords (first visible).
    pub fn main_view(&self) -> &ViewState {
        if self.view_enabled {
            for v in &self.views {
                if v.visible {
                    return v;
                }
            }
        }
        &self.views[0]
    }

    fn enter_room(&mut self, idx: usize, game_start: bool) {
        // remove non-persistent instances (no destroy events on room change)
        for s in self.slots.iter_mut().flatten() {
            if !s.persistent {
                s.alive = false;
            }
        }
        self.sweep();
        self.room_idx = idx.min(self.project.rooms.len().saturating_sub(1));
        self.apply_room_settings();
        let instances = self.project.rooms[self.room_idx].instances.clone();
        for ri in &instances {
            if ri.object < self.project.objects.len() {
                let _ = self.create_instance(ri.x, ri.y, ri.object);
            }
        }
        // room creation code (no self instance context)
        let cc = self.project.rooms[self.room_idx].creation_code.clone();
        if !cc.trim().is_empty() {
            if let Err(e) = self.run_code(&cc, NOONE) {
                self.errors
                    .push(format!("ERROR in room creation code: {}", e));
            }
        }
        let ids = self.alive_ids();
        for id in &ids {
            self.run_event(*id, &EventKey::RoomStart, NOONE);
        }
        if game_start {
            let ids = self.alive_ids();
            for id in &ids {
                self.run_event(*id, &EventKey::GameStart, NOONE);
            }
        }
    }

    fn switch_room(&mut self) {
        let pending = match self.pending_room.take() {
            Some(p) => p,
            None => return,
        };
        let n = self.project.rooms.len();
        let ids = self.alive_ids();
        match pending {
            PendingRoom::Goto(i) => {
                if i < n {
                    for id in &ids {
                        self.run_event(*id, &EventKey::RoomEnd, NOONE);
                    }
                    self.enter_room(i, false);
                }
            }
            PendingRoom::Next => {
                for id in &ids {
                    self.run_event(*id, &EventKey::RoomEnd, NOONE);
                }
                self.enter_room((self.room_idx + 1).min(n.saturating_sub(1)), false);
            }
            PendingRoom::Previous => {
                for id in &ids {
                    self.run_event(*id, &EventKey::RoomEnd, NOONE);
                }
                self.enter_room(self.room_idx.saturating_sub(1), false);
            }
            PendingRoom::Restart => {
                for id in &ids {
                    self.run_event(*id, &EventKey::RoomEnd, NOONE);
                }
                // on restart, persistent instances are removed too
                for s in self.slots.iter_mut().flatten() {
                    s.alive = false;
                }
                self.sweep();
                self.enter_room(self.room_idx, false);
            }
        }
    }

    // ---------- input ----------

    pub fn push_input(
        &mut self,
        keys_down: HashSet<i64>,
        keys_pressed: HashSet<i64>,
        keys_released: HashSet<i64>,
        mouse_sx: f64,
        mouse_sy: f64,
        mouse_bits: u8,
        mouse_pressed_bits: u8,
        mouse_released_bits: u8,
        wheel: i32,
    ) {
        self.keys_down = keys_down;
        for vk in keys_pressed.iter() {
            self.last_vk = *vk;
        }
        self.keys_pressed.extend(keys_pressed);
        self.keys_released.extend(keys_released);
        self.mouse_sx = mouse_sx;
        self.mouse_sy = mouse_sy;
        self.mouse_bits = mouse_bits;
        self.mouse_pressed_bits |= mouse_pressed_bits;
        self.mouse_released_bits |= mouse_released_bits;
        self.wheel += wheel;
    }

    /// Feed typed characters (for keyboard_string / keyboard_lastchar).
    pub fn push_chars(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\u{8}' {
                self.keyboard_string.pop();
            } else {
                self.keyboard_string.push(ch);
                self.last_char = ch.to_string();
            }
        }
    }

    pub fn key_down(&self, vk: i64) -> bool {
        self.keys_down.contains(&vk)
    }
    pub fn key_pressed(&self, vk: i64) -> bool {
        self.keys_pressed.contains(&vk)
    }
    pub fn key_released(&self, vk: i64) -> bool {
        self.keys_released.contains(&vk)
    }
    pub fn clear_key(&mut self, vk: i64) {
        self.keys_down.remove(&vk);
    }

    /// Room-coordinates mouse position.
    pub fn mouse_world(&self) -> (f64, f64) {
        if self.view_enabled {
            let v = self.main_view();
            (self.mouse_sx + v.x, self.mouse_sy + v.y)
        } else {
            (self.mouse_sx, self.mouse_sy)
        }
    }

    // ---------- collision helpers ----------

    /// bbox (left, top, right, bottom) in room coords for the instance's
    /// current mask/sprite. GM bbox values ignore scaling (as in GM8).
    pub fn bbox(&self, id: i64) -> (f64, f64, f64, f64) {
        let i = match self.inst(id) {
            Some(i) => i,
            None => return (0.0, 0.0, 0.0, 0.0),
        };
        if let Some(bb) = self.mask_bbox(i) {
            (
                i.x + bb.0 as f64,
                i.y + bb.1 as f64,
                i.x + bb.2 as f64,
                i.y + bb.3 as f64,
            )
        } else {
            (i.x, i.y, i.x, i.y)
        }
    }

    fn mask_bbox(&self, i: &Instance) -> Option<(i32, i32, i32, i32)> {
        let spr = if i.mask_index >= 0 {
            i.mask_index
        } else {
            i.sprite_index
        };
        if spr >= 0 && (spr as usize) < self.sprites.len() {
            Some(self.sprites[spr as usize].bbox)
        } else {
            None
        }
    }

    fn frame_index(&self, i: &Instance) -> usize {
        let spr = i.sprite_index;
        if spr < 0 || (spr as usize) >= self.sprites.len() {
            return 0;
        }
        let n = self.sprites[spr as usize].masks.len().max(1);
        let f = i.image_index.floor() as i64;
        ((f % n as i64 + n as i64) % n as i64) as usize
    }

    /// Do two instances (a at overridden position) overlap, using their
    /// bboxes and, when both sprites are precise, per-pixel masks?
    pub fn overlap_at(&self, a: &Instance, ax: f64, ay: f64, b: &Instance) -> bool {
        let (ab, bb) = match (self.mask_bbox(a), self.mask_bbox(b)) {
            (Some(x), Some(y)) => (x, y),
            _ => return false,
        };
        let al = ax + ab.0 as f64;
        let at = ay + ab.1 as f64;
        let ar = ax + ab.2 as f64;
        let abot = ay + ab.3 as f64;
        let bl = b.x + bb.0 as f64;
        let bt = b.y + bb.1 as f64;
        let br = b.x + bb.2 as f64;
        let bb2 = b.y + bb.3 as f64;
        if ar < bl || br < al || abot < bt || bb2 < at {
            return false;
        }
        let a_spr = if a.mask_index >= 0 {
            a.mask_index
        } else {
            a.sprite_index
        };
        let b_spr = if b.mask_index >= 0 {
            b.mask_index
        } else {
            b.sprite_index
        };
        let a_precise = a_spr >= 0
            && (a_spr as usize) < self.sprites.len()
            && self.sprites[a_spr as usize].precise;
        let b_precise = b_spr >= 0
            && (b_spr as usize) < self.sprites.len()
            && self.sprites[b_spr as usize].precise;
        if !a_precise || !b_precise {
            return true; // bbox collision
        }
        // per-pixel test in the intersection rectangle
        let a_data = &self.sprites[a_spr as usize];
        let b_data = &self.sprites[b_spr as usize];
        let fa = self.frame_index_pub(a);
        let fb = self.frame_index_pub(b);
        let x0 = al.max(bl).ceil() as i64;
        let x1 = ar.min(br).floor() as i64;
        let y0 = at.max(bt).ceil() as i64;
        let y1 = abot.min(bb2).floor() as i64;
        let mut y = y0;
        while y <= y1 {
            let mut x = x0;
            while x <= x1 {
                let axp = x - (ax + ab.0 as f64).round() as i64;
                let ayp = y - (ay + ab.1 as f64).round() as i64;
                let bxp = x - (b.x + bb.0 as f64).round() as i64;
                let byp = y - (b.y + bb.1 as f64).round() as i64;
                if a_data.mask_bit(fa, axp as i32, ayp as i32)
                    && b_data.mask_bit(fb, bxp as i32, byp as i32)
                {
                    return true;
                }
                x += 1;
            }
            y += 1;
        }
        false
    }

    pub fn frame_index_pub(&self, i: &Instance) -> usize {
        self.frame_index(i)
    }

    /// Would the instance (at x,y) overlap any matching instance?
    pub fn place_meeting_target(
        &self,
        id: i64,
        x: f64,
        y: f64,
        target: i64,
        self_id: i64,
        other_id: i64,
        solid_only: bool,
    ) -> bool {
        let a = match self.inst(id) {
            Some(i) => i,
            None => return false,
        };
        let targets = self.ids_for_target(target, self_id, other_id);
        for tid in targets {
            if tid == id {
                continue;
            }
            if let Some(b) = self.inst(tid) {
                if solid_only && !b.solid {
                    continue;
                }
                if self.overlap_at(a, x, y, b) {
                    return true;
                }
            }
        }
        false
    }

    fn point_in_instance(&self, id: i64, x: f64, y: f64) -> bool {
        let i = match self.inst(id) {
            Some(i) => i,
            None => return false,
        };
        if let Some(bb) = self.mask_bbox(i) {
            let l = i.x + bb.0 as f64;
            let t = i.y + bb.1 as f64;
            let r = i.x + bb.2 as f64;
            let b = i.y + bb.3 as f64;
            x >= l && x <= r && y >= t && y <= b
        } else {
            false
        }
    }

    // ---------- the game loop ----------

    pub fn step(&mut self) {
        // 1. begin step
        let ids = self.alive_ids();
        for id in &ids {
            self.run_event(*id, &EventKey::BeginStep, NOONE);
        }
        // 2. alarms
        let ids = self.alive_ids();
        for id in &ids {
            let alarms: Vec<(usize, bool)> = {
                let i = self.inst(*id).unwrap();
                i.alarm
                    .iter()
                    .enumerate()
                    .map(|(n, &v)| (n, v > 0.0))
                    .collect()
            };
            for (n, active) in alarms {
                if active {
                    let fire = {
                        let i = self.inst_mut(*id).unwrap();
                        i.alarm[n] -= 1.0;
                        i.alarm[n] <= 0.0
                    };
                    if fire {
                        if let Some(i) = self.inst_mut(*id) {
                            i.alarm[n] = -1.0;
                        }
                        self.run_event(*id, &EventKey::Alarm(n as u8), NOONE);
                    }
                }
            }
        }
        // 3. keyboard held (+ <no key>)
        let keys: Vec<i64> = self.keys_down.iter().copied().collect();
        let ids = self.alive_ids();
        for vk in keys {
            for id in &ids {
                self.run_event(*id, &EventKey::Keyboard(vk), NOONE);
            }
        }
        if self.keys_down.is_empty() {
            for id in &ids {
                self.run_event(*id, &EventKey::Keyboard(0), NOONE);
            }
        }
        // 4. key press / 5. key release
        let pressed: Vec<i64> = self.keys_pressed.iter().copied().collect();
        let ids = self.alive_ids();
        for vk in pressed {
            for id in &ids {
                self.run_event(*id, &EventKey::KeyPress(vk), NOONE);
            }
        }
        let released: Vec<i64> = self.keys_released.iter().copied().collect();
        let ids = self.alive_ids();
        for vk in released {
            for id in &ids {
                self.run_event(*id, &EventKey::KeyRelease(vk), NOONE);
            }
        }
        // 6. mouse events
        self.mouse_events();
        // 7. step
        let ids = self.alive_ids();
        for id in &ids {
            self.run_event(*id, &EventKey::Step, NOONE);
        }
        // 7.5 timelines (moments execute right after the step event)
        self.timeline_step();
        // 8. motion + animation
        self.motion();
        // 9. collisions
        self.collision_events();
        // 9.5 particles
        self.particles.update();
        // 10. end step
        let ids = self.alive_ids();
        for id in &ids {
            self.run_event(*id, &EventKey::EndStep, NOONE);
        }
        // 11. outside room / boundary
        let room_w = self.project.rooms[self.room_idx].width;
        let room_h = self.project.rooms[self.room_idx].height;
        let ids = self.alive_ids();
        for id in ids {
            let (l, t, r, b) = self.bbox(id);
            let outside = r < 0.0 || b < 0.0 || l > room_w || t > room_h;
            let boundary = l < 0.0 || t < 0.0 || r > room_w || b > room_h;
            if outside {
                self.run_event(id, &EventKey::OutsideRoom, NOONE);
            } else if boundary {
                self.run_event(id, &EventKey::IntersectBoundary, NOONE);
            }
        }
        // 12. no more lives / health
        if self.lives < 0.0 && self.last_lives >= 0.0 {
            let ids = self.alive_ids();
            for id in ids {
                self.run_event(id, &EventKey::NoMoreLives, NOONE);
            }
        }
        self.last_lives = self.lives;
        if self.health <= 0.0 && self.last_health > 0.0 {
            let ids = self.alive_ids();
            for id in ids {
                self.run_event(id, &EventKey::NoMoreHealth, NOONE);
            }
        }
        self.last_health = self.health;
        // 13. cleanup + room switch
        self.sweep();
        self.switch_room();
        // 14. clear per-step input
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed_bits = 0;
        self.mouse_released_bits = 0;
        self.wheel = 0;
        // fps
        self.fps_frames += 1;
        let now = self.backend.now_ms();
        if self.fps_ms == 0.0 {
            self.fps_ms = now;
        } else if now - self.fps_ms >= 500.0 {
            self.fps = self.fps_frames as f64 * 1000.0 / (now - self.fps_ms);
            self.fps_frames = 0;
            self.fps_ms = now;
        }
    }

    fn timeline_step(&mut self) {
        // collect (id, timeline_idx, old_pos, new_pos) then execute moments
        let mut jobs: Vec<(i64, usize, f64, f64, bool)> = Vec::new();
        for i in self.slots.iter().flatten() {
            if !(i.alive && i.active && i.timeline_running && i.timeline_index >= 0) {
                continue;
            }
            let ti = i.timeline_index as usize;
            if ti >= self.timelines.len() {
                continue;
            }
            let old = i.timeline_pos;
            let mut new = old + i.timeline_speed;
            let mut wrapped = false;
            let max_pos = self.timelines[ti]
                .last()
                .map(|(s, _)| *s as f64)
                .unwrap_or(0.0);
            if new > max_pos {
                if i.timeline_loop {
                    new = 0.0;
                    wrapped = true;
                } else {
                    new = max_pos;
                }
            }
            jobs.push((i.id, ti, old, new, wrapped));
        }
        for (id, ti, old, new, wrapped) in jobs {
            let moments: Vec<(i64, Rc<Program>)> = self.timelines[ti]
                .iter()
                .filter(|(s, _)| (*s as f64) > old && (*s as f64) <= new)
                .cloned()
                .collect();
            for (_, prog) in moments {
                if self.inst(id).is_none() {
                    break;
                }
                let mut it = gml::Interp::new(self);
                if let Err(e) = it.run(&prog, vec![], id, NOONE) {
                    self.errors.push(format!("ERROR in timeline moment: {}", e));
                }
            }
            let last_step = self.timelines[ti]
                .last()
                .map(|(s, _)| *s as f64)
                .unwrap_or(0.0);
            if let Some(i) = self.inst_mut(id) {
                i.timeline_pos = new;
                if wrapped {
                    // after wrapping, run moments from 0 up to the wrapped position
                    i.timeline_pos = 0.0;
                } else if new >= last_step {
                    i.timeline_running = false;
                }
            }
        }
    }

    /// Advance instances that follow a path (called from motion()).
    fn path_motion(&mut self, id: i64) {
        let (pi, pspeed, ppos, pend, pabs, pxs, pys) = {
            let i = match self.inst(id) {
                Some(i) => i,
                None => return,
            };
            (
                i.path_index,
                i.path_speed,
                i.path_pos,
                i.path_endaction,
                i.path_absolute,
                i.path_xstart,
                i.path_ystart,
            )
        };
        if pi < 0 || pspeed == 0.0 {
            return;
        }
        let pd = match self.paths.get(pi as usize) {
            Some(p) => (p.total_cost, p.length(), p.clone_points()),
            None => return,
        };
        let (total_cost, length, points) = pd;
        let first_pt = points.first().map(|p| (p.0, p.1)).unwrap_or((0.0, 0.0));
        if total_cost <= 0.0 {
            return;
        }
        let advance = pspeed / length; // fraction of the path per step
        let mut npos = ppos + advance;
        let mut running = true;
        if npos >= 1.0 {
            match pend {
                1 => npos %= 1.0, // restart from the beginning
                2 => {
                    // reverse: go backwards
                    npos = 2.0 - npos;
                    if let Some(i) = self.inst_mut(id) {
                        i.path_speed = -i.path_speed;
                    }
                }
                _ => {
                    npos = 1.0;
                    running = false;
                }
            }
        }
        if npos < 0.0 && pspeed < 0.0 {
            // travelling backwards
            if npos <= -1.0 + advance {
                match pend {
                    1 => npos = 1.0 + (npos % 1.0),
                    2 => {
                        npos = -npos;
                        if let Some(i) = self.inst_mut(id) {
                            i.path_speed = -i.path_speed;
                        }
                    }
                    _ => {
                        npos = 0.0;
                        running = false;
                    }
                }
            }
        }
        let (px, py, _dir) = path_point_at(&points, total_cost, npos.clamp(-1.0, 2.0));
        if let Some(i) = self.inst_mut(id) {
            i.xprev = i.x;
            i.yprev = i.y;
            i.path_posprev = i.path_pos;
            i.path_pos = npos;
            if pabs {
                i.x = px;
                i.y = py;
            } else {
                i.x = pxs + px - first_pt.0;
                i.y = pys + py - first_pt.1;
            }
            if !running {
                i.path_speed = 0.0;
            }
        }
    }

    fn mouse_events(&mut self) {
        let (mx, my) = self.mouse_world();
        let ids = self.alive_ids();
        // enter/leave tracking + local/global button events
        for id in &ids {
            let inside = self.point_in_instance(*id, mx, my);
            let was = self.inst(*id).map(|i| i.hovered).unwrap_or(false);
            if inside && !was {
                self.run_event(*id, &EventKey::Mouse(MouseEv::Enter), NOONE);
            }
            if !inside && was {
                self.run_event(*id, &EventKey::Mouse(MouseEv::Leave), NOONE);
            }
            if let Some(i) = self.inst_mut(*id) {
                i.hovered = inside;
            }
        }
        for button in 0u8..3 {
            let bit = 1u8 << button;
            let down = self.mouse_bits & bit != 0;
            let pressed = self.mouse_pressed_bits & bit != 0;
            let released = self.mouse_released_bits & bit != 0;
            if !down && !pressed && !released {
                continue;
            }
            let ids = self.alive_ids();
            for id in &ids {
                let inside = self.point_in_instance(*id, mx, my);
                if down {
                    if inside {
                        self.run_event(
                            *id,
                            &EventKey::Mouse(MouseEv::Button {
                                button,
                                down: true,
                                pressed: false,
                                global: false,
                            }),
                            NOONE,
                        );
                    }
                    self.run_event(
                        *id,
                        &EventKey::Mouse(MouseEv::Button {
                            button,
                            down: true,
                            pressed: false,
                            global: true,
                        }),
                        NOONE,
                    );
                }
                if pressed {
                    if inside {
                        self.run_event(
                            *id,
                            &EventKey::Mouse(MouseEv::Button {
                                button,
                                down: false,
                                pressed: true,
                                global: false,
                            }),
                            NOONE,
                        );
                    }
                    self.run_event(
                        *id,
                        &EventKey::Mouse(MouseEv::Button {
                            button,
                            down: false,
                            pressed: true,
                            global: true,
                        }),
                        NOONE,
                    );
                }
                if released {
                    if inside {
                        self.run_event(
                            *id,
                            &EventKey::Mouse(MouseEv::Button {
                                button,
                                down: false,
                                pressed: false,
                                global: false,
                            }),
                            NOONE,
                        );
                    }
                    self.run_event(
                        *id,
                        &EventKey::Mouse(MouseEv::Button {
                            button,
                            down: false,
                            pressed: false,
                            global: true,
                        }),
                        NOONE,
                    );
                }
            }
        }
        // no button (over instance, nothing pressed)
        if self.mouse_bits == 0 {
            let ids = self.alive_ids();
            for id in &ids {
                if self.point_in_instance(*id, mx, my) {
                    self.run_event(*id, &EventKey::Mouse(MouseEv::NoButton), NOONE);
                }
            }
        }
        // wheel
        if self.wheel != 0 {
            let ids = self.alive_ids();
            let ev = if self.wheel > 0 {
                MouseEv::WheelUp
            } else {
                MouseEv::WheelDown
            };
            for id in &ids {
                self.run_event(*id, &EventKey::Mouse(ev), NOONE);
            }
        }
    }

    fn motion(&mut self) {
        let ids = self.alive_ids();
        for id in ids {
            let n_frames = {
                let i = match self.inst(id) {
                    Some(i) => i,
                    None => continue,
                };
                let spr = i.sprite_index;
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    self.sprites[spr as usize].masks.len().max(1)
                } else {
                    0
                }
            };
            let mut wrap = false;
            {
                // path following overrides normal motion
                let path_active = {
                    let i = match self.inst(id) {
                        Some(i) => i,
                        None => continue,
                    };
                    i.path_index >= 0 && i.path_speed != 0.0
                };
                if path_active {
                    self.path_motion(id);
                    let anim_wrap_later = {
                        let i = match self.inst_mut(id) {
                            Some(i) => i,
                            None => continue,
                        };
                        animate_frame(i, n_frames)
                    };
                    if anim_wrap_later {
                        self.run_event(id, &EventKey::AnimationEnd, NOONE);
                    }
                    continue;
                }
                let i = match self.inst_mut(id) {
                    Some(i) => i,
                    None => continue,
                };
                // friction: reduce speed toward 0
                if i.friction != 0.0 {
                    let sp = i.speed();
                    if sp > 0.0 {
                        let ns = (sp - i.friction.abs()).max(0.0);
                        let dir = i.direction();
                        i.set_speed_dir(ns, dir);
                    }
                }
                // gravity
                if i.gravity != 0.0 {
                    i.hspeed += i.gravity * i.gravity_dir.to_radians().cos();
                    i.vspeed -= i.gravity * i.gravity_dir.to_radians().sin();
                    i.recalc_dir();
                }
                // integrate
                i.xprev = i.x;
                i.yprev = i.y;
                i.x += i.hspeed;
                i.y += i.vspeed;
                // animation
                if n_frames > 0 && i.image_speed != 0.0 {
                    wrap = animate_frame(i, n_frames);
                }
            }
            if wrap {
                self.run_event(id, &EventKey::AnimationEnd, NOONE);
            }
        }
    }

    fn collision_events(&mut self) {
        let ids = self.alive_ids();
        let n = ids.len();
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let a_id = ids[i];
                let b_id = ids[j];
                let (a_obj, b_obj, b_solid) = match (self.inst(a_id), self.inst(b_id)) {
                    (Some(a), Some(b)) => (a.obj, b.obj, b.solid),
                    _ => continue,
                };
                let ev_key = match self.find_collision_event(a_obj, b_obj) {
                    Some(k) => k,
                    None => continue,
                };
                let overlap = {
                    let a = self.inst(a_id).unwrap();
                    let b = self.inst(b_id).unwrap();
                    self.overlap_at(a, a.x, a.y, b)
                };
                if !overlap {
                    continue;
                }
                // GM solid semantics: revert to previous position first
                let revert = b_solid;
                if revert {
                    if let Some(a) = self.inst_mut(a_id) {
                        a.x = a.xprev;
                        a.y = a.yprev;
                    }
                }
                self.run_event(a_id, &ev_key, b_id);
                if revert {
                    let still = match (self.inst(a_id), self.inst(b_id)) {
                        (Some(a), Some(b)) => self.overlap_at(a, a.x, a.y, b),
                        _ => false, // one of them was destroyed in the event
                    };
                    if still {
                        if let Some(a) = self.inst_mut(a_id) {
                            a.x = a.xprev;
                            a.y = a.yprev;
                        }
                    }
                }
            }
        }
    }

    // ---------- drawing ----------

    pub fn draw(&mut self) {
        // update view positions (follow objects) once per frame
        self.update_views();
        let room_w = self.project.rooms[self.room_idx].width;
        let room_h = self.project.rooms[self.room_idx].height;
        let room = self.project.rooms[self.room_idx].clone();
        if room.show_color {
            self.backend.clear(room.back_color);
        } else {
            self.backend.clear(0);
        }
        // Determine which views to render. View 0 renders into the whole
        // window; extra visible views render after it (overlayed).
        let pass_count = if self.view_enabled {
            self.views.iter().filter(|v| v.visible).count().max(1)
        } else {
            1
        };
        let mut first = true;
        for vi in 0..self.views.len() {
            if self.view_enabled && !self.views[vi].visible {
                continue;
            }
            if !first {
                break; // render only the first visible view (see README)
            }
            first = false;
            self.view_current = vi;
            self.draw_pass(&room, room_w, room_h);
            let _ = pass_count;
        }
        if !self.view_enabled {
            self.view_current = 0;
        }
        // in-game debug overlay (show_debug_message)
        if !self.debug_log.is_empty() {
            let (vx, vy) = if self.view_enabled {
                (self.views[0].x, self.views[0].y)
            } else {
                (0.0, 0.0)
            };
            let _ = vy;
            let alpha = 200u8;
            let y0 = 4.0f32;
            let (color, a) = (0u32, alpha); // black bg
            self.backend.draw_rect(
                vx as f32,
                y0,
                400.0,
                self.debug_log.len() as f32 * 16.0 + 4.0,
                true,
                color,
                a as f32 / 255.0,
            );
            for (i, msg) in self.debug_log.iter().enumerate() {
                self.backend.draw_text(
                    0,
                    vx as f32 + 6.0,
                    y0 + 2.0 + i as f32 * 16.0,
                    msg,
                    crate::world::FONT_SIZE - 2.0,
                    0x00FF00, // green text
                    1.0,
                );
            }
        }
    }

    fn draw_pass(&mut self, room: &gmformat::Room, room_w: f64, room_h: f64) {
        let view = self.views[self.view_current.min(self.views.len() - 1)].clone();
        let (vw, vh) = if self.view_enabled {
            (view.w, view.h)
        } else {
            (room_w, room_h)
        };
        let (ox, oy) = if self.view_enabled {
            (view.x, view.y)
        } else {
            (0.0, 0.0)
        };
        // backgrounds
        for bg in &room.backgrounds {
            if !bg.visible || bg.index < 0 || (bg.index as usize) >= self.bg_tex.len() {
                continue;
            }
            let bidx = bg.index as usize;
            let (bw, bh) = {
                let b = &self.project.backgrounds[bidx];
                (b.width as f64, b.height as f64)
            };
            let tex = self.bg_tex[bidx];
            let start_x = if bg.tiled_h {
                -(((ox - bg.x) % bw + bw) % bw)
            } else {
                bg.x - ox
            };
            let start_y = if bg.tiled_v {
                -(((oy - bg.y) % bh + bh) % bh)
            } else {
                bg.y - oy
            };
            let mut ty = start_y;
            while ty < vh {
                let mut tx = start_x;
                while tx < vw {
                    self.backend.draw_sprite(
                        tex, tx as f32, ty as f32, bw as f32, bh as f32, 0.0, 1.0, 0xFFFFFF, false,
                        false, None,
                    );
                    if !bg.tiled_h {
                        break;
                    }
                    tx += bw;
                }
                if !bg.tiled_v {
                    break;
                }
                ty += bh;
            }
        }
        // tiles + instances + particle systems, interleaved by depth
        let mut order: Vec<(i32, u8, i64)> = Vec::new(); // (depth, kind, id) 0=instance, 1=tile, 2=ps
        for i in self.slots.iter().flatten() {
            if i.alive && i.active {
                order.push((i.depth, 0, i.id));
            }
        }
        for t in &self.tiles {
            if t.visible {
                order.push((t.depth, 1, t.id));
            }
        }
        for (psid, ps) in self.particles.systems.iter().enumerate() {
            order.push((ps.depth, 2, psid as i64));
        }
        order.sort_by_key(|&(d, kind, id)| (d, kind, id));
        for (_, kind, id) in order {
            match kind {
                0 => {
                    let (visible, _obj, has_draw) = {
                        let i = self.inst(id).unwrap();
                        (
                            i.visible,
                            i.obj,
                            self.obj_events
                                .get(obj_of(i))
                                .map(|m| m.contains_key(&EventKey::Draw))
                                .unwrap_or(false),
                        )
                    };
                    if !visible {
                        continue;
                    }
                    if has_draw {
                        self.run_event(id, &EventKey::Draw, NOONE);
                    } else {
                        self.default_draw(id);
                    }
                }
                1 => {
                    if let Some(t) = self.tiles.iter().find(|t| t.id == id) {
                        if t.bg >= 0 && (t.bg as usize) < self.bg_tex.len() {
                            let tex = self.bg_tex[t.bg as usize];
                            let (sx, sy) = self.view_screen(t.x, t.y);
                            self.backend.draw_sprite(
                                tex,
                                sx as f32,
                                sy as f32,
                                t.width as f32,
                                t.height as f32,
                                0.0,
                                1.0,
                                0xFFFFFF,
                                false,
                                false,
                                Some((
                                    t.left as f32,
                                    t.top as f32,
                                    t.width as f32,
                                    t.height as f32,
                                )),
                            );
                        }
                    }
                }
                _ => {
                    if let Some(sys) = self.particles.systems.get(id as usize) {
                        let snapshot: Vec<(f64, f64, f64, u32, f32, f64, i64, bool)> = sys
                            .particles
                            .iter()
                            .map(|p| {
                                (
                                    p.x,
                                    p.y,
                                    p.size,
                                    p.color,
                                    p.alpha as f32,
                                    p.angle,
                                    p.ty as i64,
                                    false,
                                )
                            })
                            .collect();
                        let types: Vec<crate::particles::PType> = sys.types.clone();
                        for (px, py, psize, pcolor, palpha, pangle, pty, _) in snapshot {
                            let t = match types.get(pty as usize) {
                                Some(t) => t.clone(),
                                None => continue,
                            };
                            let (sx, sy) = self.view_screen(px, py);
                            if t.sprite >= 0 && (t.sprite as usize) < self.sprites.len() {
                                self.draw_sprite_impl(
                                    t.sprite,
                                    -1.0,
                                    px,
                                    py,
                                    t.scale.0 * psize,
                                    t.scale.1 * psize,
                                    pangle,
                                    pcolor,
                                    palpha as f64,
                                );
                            } else {
                                crate::particles::draw_shape_pub(
                                    self,
                                    t.shape,
                                    sx,
                                    sy,
                                    psize,
                                    pcolor,
                                    palpha,
                                    pangle,
                                    t.blend_add,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Follow-view objects and clamp to room bounds.
    fn update_views(&mut self) {
        if !self.view_enabled {
            return;
        }
        let room_w = self.project.rooms[self.room_idx].width;
        let room_h = self.project.rooms[self.room_idx].height;
        for vi in 0..self.views.len() {
            let (obj, hb, vb, hsp, vsp) = {
                let v = &self.views[vi];
                (v.object, v.hborder, v.vborder, v.hspeed, v.vspeed)
            };
            if obj < 0 {
                continue;
            }
            let pos = self
                .ids_for_target(obj, NOONE, NOONE)
                .first()
                .and_then(|&id| self.inst(id))
                .map(|i| (i.x, i.y));
            if let Some((ix, iy)) = pos {
                let v = &mut self.views[vi];
                let target_x = ix - v.w / 2.0;
                let target_y = iy - v.h / 2.0;
                if hsp < 0.0 {
                    if ix - v.x > v.w - hb {
                        v.x = ix - (v.w - hb);
                    }
                    if ix - v.x < hb {
                        v.x = ix - hb;
                    }
                } else {
                    v.x += (target_x - v.x).clamp(-hsp, hsp);
                }
                if vsp < 0.0 {
                    if iy - v.y > v.h - vb {
                        v.y = iy - (v.h - vb);
                    }
                    if iy - v.y < vb {
                        v.y = iy - vb;
                    }
                } else {
                    v.y += (target_y - v.y).clamp(-vsp, vsp);
                }
            }
            let v = &mut self.views[vi];
            if v.w <= room_w {
                v.x = v.x.clamp(0.0, room_w - v.w);
            } else {
                v.x = (room_w - v.w) / 2.0; // view larger than room: center
            }
            if v.h <= room_h {
                v.y = v.y.clamp(0.0, room_h - v.h);
            } else {
                v.y = (room_h - v.h) / 2.0;
            }
        }
    }

    /// Shared sprite drawing (used by default draw and draw_sprite functions).
    pub fn draw_sprite_impl(
        &mut self,
        sprite: i64,
        subimg: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        rot_deg: f64,
        blend: u32,
        alpha: f64,
    ) {
        if sprite < 0 || (sprite as usize) >= self.sprites.len() {
            return;
        }
        let sidx = sprite as usize;
        let frame = if subimg < 0.0 { 0.0 } else { subimg };
        let n = self.sprites[sidx].masks.len().max(1);
        let f = ((frame.floor() as i64 % n as i64 + n as i64) % n as i64) as usize;
        let tex = match self.sprite_tex[sidx].get(f) {
            Some(&t) => t,
            None => return,
        };
        let (w, h, xoff, yoff) = {
            let s = &self.sprites[sidx];
            (s.w as f64, s.h as f64, s.xoff as f64, s.yoff as f64)
        };
        let dw = w * xscale.abs();
        let dh = h * yscale.abs();
        // Rotation happens around the sprite center (macroquad behavior);
        // place the quad so that the ORIGIN point lands at (x, y) rotated.
        let cx_rel = xoff * xscale - dw / 2.0;
        let cy_rel = yoff * yscale - dh / 2.0;
        let rad = rot_deg.to_radians();
        let (sin, cos) = rad.sin_cos();
        let cx = x - (cx_rel * cos - cy_rel * sin);
        let cy = y - (cx_rel * sin + cy_rel * cos);
        let (scx, scy) = self.view_screen(cx, cy);
        let _ = (x, y);
        self.backend.draw_sprite(
            tex,
            (scx - dw / 2.0) as f32,
            (scy - dh / 2.0) as f32,
            dw as f32,
            dh as f32,
            rot_deg as f32,
            alpha.clamp(0.0, 1.0) as f32,
            blend,
            xscale < 0.0,
            yscale < 0.0,
            None,
        );
    }

    fn default_draw(&mut self, id: i64) {
        let (sprite, image_index, x, y, xs, ys, angle, alpha, blend) = {
            let i = self.inst(id).unwrap();
            (
                i.sprite_index,
                i.image_index,
                i.x,
                i.y,
                i.image_xscale,
                i.image_yscale,
                i.image_angle,
                i.image_alpha,
                i.image_blend,
            )
        };
        if sprite < 0 {
            return;
        }
        self.draw_sprite_impl(sprite, image_index, x, y, xs, ys, angle, blend, alpha);
    }

    // ---------- misc ----------

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors)
    }

    pub fn game_restart(&mut self) {
        for s in self.slots.iter_mut().flatten() {
            s.alive = false;
        }
        self.sweep();
        self.particles = Particles::new();
        self.globals.clear();
        self.score = 0.0;
        self.lives = 0.0;
        self.health = 100.0;
        self.pending_room = None;
        self.enter_room(0, true);
    }

    pub fn game_end(&mut self) {
        let ids = self.alive_ids();
        for id in ids {
            self.run_event(id, &EventKey::GameEnd, NOONE);
        }
        self.ended = true;
    }

    pub fn room_goto(&mut self, idx: usize) {
        self.pending_room = Some(PendingRoom::Goto(idx));
    }

    pub fn room_goto_next(&mut self) {
        self.pending_room = Some(PendingRoom::Next);
    }

    pub fn room_goto_previous(&mut self) {
        self.pending_room = Some(PendingRoom::Previous);
    }

    pub fn room_restart_pending(&mut self) {
        self.pending_room = Some(PendingRoom::Restart);
    }

    pub fn rand(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        let v = x.wrapping_mul(0x2545F4914F6CDD1D);
        ((v >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

fn obj_of(i: &Instance) -> usize {
    i.obj
}

pub fn event_label(key: &EventKey) -> String {
    match key {
        EventKey::Create => "Create".into(),
        EventKey::Destroy => "Destroy".into(),
        EventKey::Alarm(n) => format!("Alarm {}", n),
        EventKey::Step => "Step".into(),
        EventKey::BeginStep => "Begin Step".into(),
        EventKey::EndStep => "End Step".into(),
        EventKey::Collision(_) => "Collision".into(),
        EventKey::Keyboard(k) => format!("Keyboard {}", k),
        EventKey::KeyPress(k) => format!("Key Press {}", k),
        EventKey::KeyRelease(k) => format!("Key Release {}", k),
        EventKey::Mouse(m) => format!("Mouse {:?}", m),
        EventKey::Draw => "Draw".into(),
        EventKey::AnimationEnd => "Animation End".into(),
        EventKey::OutsideRoom => "Outside Room".into(),
        EventKey::IntersectBoundary => "Intersect Boundary".into(),
        EventKey::GameStart => "Game Start".into(),
        EventKey::GameEnd => "Game End".into(),
        EventKey::RoomStart => "Room Start".into(),
        EventKey::RoomEnd => "Room End".into(),
        EventKey::NoMoreLives => "No More Lives".into(),
        EventKey::NoMoreHealth => "No More Health".into(),
    }
}

/// Advance image animation by one step; returns true when the animation wrapped.
fn animate_frame(i: &mut Instance, n_frames: usize) -> bool {
    if n_frames == 0 || i.image_speed == 0.0 {
        return false;
    }
    let n = n_frames as f64;
    i.image_index += i.image_speed;
    let mut wrap = false;
    if i.image_index >= n {
        i.image_index -= n;
        wrap = true;
    }
    while i.image_index < 0.0 {
        i.image_index += n;
    }
    wrap
}

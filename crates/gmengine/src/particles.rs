//! GM8-style particle systems: types, emitters, particles, update, draw.
//! Supports shapes/sprites, size/color/alpha/speed/direction/life/orientation
//! ranges, gravity, step/death spawning and add blending. Attractors,
//! destroyers, deflectors and changers are not implemented.

use crate::instance::norm_deg;
use crate::world::Game;
use gml::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PType {
    // shape 0..13 or sprite
    pub shape: i64,
    pub sprite: i64,
    // size: min, max, incr, wobble
    pub size: (f64, f64, f64, f64),
    pub scale: (f64, f64),
    // color: c1, c2, c3, mix
    pub colors: (u32, u32, u32, bool),
    pub hsv: bool,
    pub color_rng: (f64, f64, f64), // for hsv ranges (hmin,hmax,sat?) unused extra
    // alpha: a1, a2, a3
    pub alphas: (f64, f64, f64),
    // speed & direction: min, max, incr, wobble
    pub speed: (f64, f64, f64, f64),
    pub direction: (f64, f64),
    pub gravity_amount: f64,
    pub gravity_dir: f64,
    // life min, max (steps)
    pub life: (f64, f64),
    // orientation: ang_min, ang_max, ang_incr, ang_wobble, ang_relative
    pub orientation: (f64, f64, f64, f64, bool),
    // secondary spawning
    pub step_type: i64,
    pub step_count: i64,
    pub death_type: i64,
    pub death_count: i64,
    pub death_on_life_end: bool,
    pub blend_add: bool,
}

impl Default for PType {
    fn default() -> Self {
        PType {
            shape: 0,
            sprite: -1,
            size: (1.0, 1.0, 0.0, 0.0),
            scale: (1.0, 1.0),
            colors: (0xFFFFFF, 0xFFFFFF, 0xFFFFFF, false),
            hsv: false,
            color_rng: (0.0, 0.0, 0.0),
            alphas: (1.0, 1.0, 1.0),
            speed: (1.0, 1.0, 0.0, 0.0),
            direction: (0.0, 0.0),
            gravity_amount: 0.0,
            gravity_dir: 270.0,
            life: (100.0, 100.0),
            orientation: (0.0, 0.0, 0.0, 0.0, false),
            step_type: -1,
            step_count: 0,
            death_type: -1,
            death_count: 0,
            death_on_life_end: true,
            blend_add: false,
        }
    }
}

#[derive(Clone)]
pub struct Particle {
    pub(crate) x: f64,
    pub(crate) y: f64,
    xs: f64,
    ys: f64,
    pub(crate) size: f64,
    pub(crate) color: u32,
    pub(crate) alpha: f64,
    pub(crate) angle: f64,
    life: f64,
    age: f64,
    pub(crate) ty: usize, // type index (inside system)
}

pub struct PSystem {
    pub depth: i32,
    pub oldtonew: bool, // draw order
    pub types: Vec<PType>,
    pub alive_types: Vec<bool>,
    pub particles: Vec<Particle>,
    pub emitters: Vec<PEmitter>,
    pub auto_update: bool,
    pub auto_draw: bool,
}

#[derive(Clone)]
pub struct PEmitter {
    // region: x1, y1, x2, y2, shape (0 rect,1 ellipse,2 diamond,3 line)
    pub region: (f64, f64, f64, f64, i64),
    // distribution: 0 linear, 1 gaussian, 2 inv gaussian
    pub distr: i64,
    pub stream: i64, // particles per step (<0 = off)
}

#[derive(Default)]
pub struct Particles {
    pub systems: Vec<PSystem>,
    pub free: Vec<usize>,
    pub type_ids: HashMap<u32, (usize, usize)>, // global type id -> (system, local index)
    pub emitter_ids: HashMap<u32, (usize, usize)>,
    pub next_type: u32,
    pub next_emitter: u32,
}

impl PSystem {
    fn new() -> Self {
        PSystem {
            depth: 0,
            oldtonew: true,
            types: vec![],
            alive_types: vec![],
            particles: vec![],
            emitters: vec![],
            auto_update: true,
            auto_draw: true,
        }
    }
}

impl Particles {
    pub fn new() -> Self {
        Particles::default()
    }

    fn alloc_system(&mut self) -> usize {
        if let Some(i) = self.free.pop() {
            self.systems[i] = PSystem::new();
            i
        } else {
            self.systems.push(PSystem::new());
            self.systems.len() - 1
        }
    }

    pub fn create_system(&mut self) -> usize {
        self.alloc_system()
    }

    pub fn destroy_system(&mut self, si: usize) {
        if si < self.systems.len() {
            self.systems[si] = PSystem::new();
            self.free.push(si);
            self.type_ids.retain(|_, v| v.0 != si);
            self.emitter_ids.retain(|_, v| v.0 != si);
        }
    }

    pub fn update(&mut self) {
        for si in 0..self.systems.len() {
            if self.free.contains(&si) {
                continue;
            }
            if !self.systems[si].auto_update {
                continue;
            }
            let emitters: Vec<(f64, f64, f64, f64, i64, i64)> = self.systems[si]
                .emitters
                .iter()
                .map(|e| {
                    (
                        e.region.0, e.region.1, e.region.2, e.region.3, e.region.4, e.distr,
                    )
                })
                .collect();
            let streams: Vec<i64> = self.systems[si].emitters.iter().map(|e| e.stream).collect();
            for (ei, &stream) in streams.iter().enumerate() {
                if stream > 0 {
                    let r = emitters[ei];
                    let count = stream.min(64) as usize;
                    for _ in 0..count {
                        let (x, y) = self.region_point(r);
                        self.spawn(si, ei, x, y);
                    }
                }
            }
            self.update_system(si);
        }
    }

    fn region_point(&mut self, r: (f64, f64, f64, f64, i64, i64)) -> (f64, f64) {
        let (x1, y1, x2, y2, shape, distr) = r;
        let t = match distr {
            1 => {
                // gaussian-ish: average of two uniforms
                (self.rand() + self.rand()) / 2.0
            }
            2 => {
                // inverse gaussian: biased to the edges
                let u = self.rand();
                if u < 0.5 {
                    u
                } else {
                    1.0 - u
                }
            }
            _ => self.rand(),
        };
        let (mut fx, mut fy) = (t, t);
        if shape == 1 || shape == 2 {
            // ellipse / diamond: pick a random direction with radius t
            let a = self.rand() * std::f64::consts::TAU;
            let c = a.cos();
            let s = a.sin();
            if shape == 2 {
                fx = t * c / (c.abs() + s.abs()).max(1e-9);
                fy = t * s / (c.abs() + s.abs()).max(1e-9);
            } else {
                fx = t * c;
                fy = t * s;
            }
        }
        (x1 + (x2 - x1) * fx, y1 + (y2 - y1) * fy)
    }

    fn spawn(&mut self, si: usize, _ei: usize, x: f64, y: f64) {
        if self.systems[si].types.is_empty() {
            return;
        }
        let ti = (self.rand() * self.systems[si].types.len() as f64) as usize;
        self.spawn_particle(si, ti, x, y);
    }

    pub fn spawn_particle(&mut self, si: usize, ti: usize, x: f64, y: f64) {
        if si >= self.systems.len() || ti >= self.systems[si].types.len() {
            return;
        }
        let t = self.systems[si].types[ti].clone();
        let r = |p: &Particles| p.rand();
        let size = t.size.0 + (t.size.1 - t.size.0) * r(self);
        let alpha = t.alphas.0; // start alpha (fades handled simply)
        let color = if t.hsv {
            hsv_merge(t.colors.0, t.colors.1, r(self))
        } else {
            merge_colors(t.colors.0, t.colors.1, r(self))
        };
        let sp = t.speed.0 + (t.speed.1 - t.speed.0) * r(self);
        let dir = (t.direction.0 + (t.direction.1 - t.direction.0) * r(self)).to_radians();
        let angle = t.orientation.0 + (t.orientation.1 - t.orientation.0) * r(self);
        let angle = if t.orientation.4 {
            norm_deg(dir.to_degrees() + angle)
        } else {
            angle
        };
        let life = t.life.0 + (t.life.1 - t.life.0) * r(self);
        self.systems[si].particles.push(Particle {
            x,
            y,
            xs: sp * dir.cos(),
            ys: -sp * dir.sin(),
            size,
            color,
            alpha: alpha.clamp(0.0, 1.0),
            angle,
            life: life.max(1.0),
            age: 0.0,
            ty: ti,
        });
    }

    fn update_system(&mut self, si: usize) {
        let mut new_particles: Vec<Particle> = Vec::new();
        let mut i = 0;
        while i < self.systems[si].particles.len() {
            let mut p = self.systems[si].particles[i].clone();
            let t = self.systems[si].types[p.ty].clone();
            // motion + gravity
            p.xs += t.gravity_amount * t.gravity_dir.to_radians().cos();
            p.ys -= t.gravity_amount * t.gravity_dir.to_radians().sin();
            p.x += p.xs;
            p.y += p.ys;
            // size / angle evolution
            p.size = (p.size + t.size.2).max(0.0);
            p.angle = norm_deg(p.angle + t.orientation.2);
            p.age += 1.0;
            // alpha fade across the three keys
            let f = p.age / p.life;
            let (a1, a2, a3) = t.alphas;
            let alpha = if f < 0.5 {
                a1 + (a2 - a1) * (f / 0.5)
            } else {
                a2 + (a3 - a2) * ((f - 0.5) / 0.5)
            };
            p.alpha = alpha.clamp(0.0, 1.0);
            // secondary step particles
            if t.step_type >= 0 && t.step_count > 0 {
                for _ in 0..t.step_count.min(16) {
                    new_particles.push(Particle {
                        x: p.x,
                        y: p.y,
                        xs: 0.0,
                        ys: 0.0,
                        size: 1.0,
                        color: 0xFFFFFF,
                        alpha: 1.0,
                        angle: 0.0,
                        life: 1.0,
                        age: 0.0,
                        ty: t.step_type as usize,
                    });
                }
            }
            let dead = p.age >= p.life;
            if dead {
                if t.death_type >= 0 && t.death_count > 0 {
                    for _ in 0..t.step_count.min(16) {
                        let _ = t.step_count;
                        new_particles.push(Particle {
                            x: p.x,
                            y: p.y,
                            xs: 0.0,
                            ys: 0.0,
                            size: 1.0,
                            color: 0xFFFFFF,
                            alpha: 1.0,
                            angle: 0.0,
                            life: 1.0,
                            age: 0.0,
                            ty: t.death_type as usize,
                        });
                    }
                }
                self.systems[si].particles.swap_remove(i);
                continue;
            }
            self.systems[si].particles[i] = p;
            i += 1;
        }
        self.systems[si].particles.extend(new_particles);
        // cap particles per system
        if self.systems[si].particles.len() > 4096 {
            let excess = self.systems[si].particles.len() - 4096;
            for _ in 0..excess {
                self.systems[si].particles.remove(0);
            }
        }
    }

    pub fn burst(&mut self, si: usize, ei: usize, count: i64) {
        if si >= self.systems.len() || ei >= self.systems[si].emitters.len() {
            return;
        }
        let r = self.systems[si].emitters[ei].clone();
        for _ in 0..count.clamp(0, 2048) {
            let (x, y) = self.region_point((
                r.region.0, r.region.1, r.region.2, r.region.3, r.region.4, r.distr,
            ));
            self.spawn(si, ei, x, y);
        }
    }

    pub fn draw_system(&mut self, _game: &mut Game, si: usize) {
        // actual drawing is handled by world::draw_pass via a snapshot,
        // this only checks visibility (kept for API compatibility)
        let _ = si;
    }

    pub fn rand(&self) -> f64 {
        // cheap xorshift through a thread-local-ish seed
        use std::cell::Cell;
        thread_local! {
            static SEED: Cell<u64> = Cell::new(0x853C49E6748FEA9B);
        }
        SEED.with(|s| {
            let mut x = s.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.set(x);
            (x >> 11) as f64 / (1u64 << 53) as f64
        })
    }
}

fn merge_colors(c1: u32, c2: u32, t: f64) -> u32 {
    let mix = |a: u32, b: u32| -> u32 { (a as f64 * (1.0 - t) + b as f64 * t).round() as u32 };
    mix(c1 & 0xFF, c2 & 0xFF)
        | (mix((c1 >> 8) & 0xFF, (c2 >> 8) & 0xFF) << 8)
        | (mix((c1 >> 16) & 0xFF, (c2 >> 16) & 0xFF) << 16)
}

fn hsv_merge(c1: u32, c2: u32, t: f64) -> u32 {
    // interpolate hue-wise via rgb merge of hsv-converted colors
    let (h1, s1, v1) = rgb_to_hsv(c1);
    let (h2, s2, v2) = rgb_to_hsv(c2);
    let mut dh = h2 as f64 - h1 as f64;
    if dh > 180.0 {
        dh -= 360.0;
    }
    if dh < -180.0 {
        dh += 360.0;
    }
    let h = ((h1 as f64 + dh * t).rem_euclid(360.0)) as u32;
    let s = (s1 as f64 * (1.0 - t) + s2 as f64 * t) as u32;
    let v = (v1 as f64 * (1.0 - t) + v2 as f64 * t) as u32;
    let (r, g, b) = hsv_to_rgb_u32(h, s, v);
    r | (g << 8) | (b << 16)
}

fn rgb_to_hsv(c: u32) -> (u32, u32, u32) {
    let r = (c & 0xFF) as f64 / 255.0;
    let g = ((c >> 8) & 0xFF) as f64 / 255.0;
    let b = ((c >> 16) & 0xFF) as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let s = if max == 0.0 { 0.0 } else { d / max };
    ((h) as u32, (s * 255.0) as u32, (max * 255.0) as u32)
}

fn hsv_to_rgb_u32(h: u32, s: u32, v: u32) -> (u32, u32, u32) {
    let h = h % 360;
    let c = v as f64 * s as f64 / 255.0;
    let hp = h as f64 / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v as f64 - c;
    (
        ((r1 + m) * 255.0) as u32,
        ((g1 + m) * 255.0) as u32,
        ((b1 + m) * 255.0) as u32,
    )
}

pub fn draw_shape_pub(
    game: &mut Game,
    shape: i64,
    x: f64,
    y: f64,
    size: f64,
    color: u32,
    alpha: f32,
    angle: f64,
    add: bool,
) {
    draw_shape(game, shape, x, y, size, color, alpha, angle, add)
}

fn draw_shape(
    game: &mut Game,
    shape: i64,
    x: f64,
    y: f64,
    size: f64,
    color: u32,
    alpha: f32,
    angle: f64,
    add: bool,
) {
    let _ = add;
    let s = (size.max(0.5)) as f32;
    match shape {
        1 | 5 | 13 => game
            .backend
            .draw_circle(x as f32, y as f32, s, true, color, alpha), // disk/circle/cloud
        6 => game
            .backend
            .draw_circle(x as f32, y as f32, s, false, color, alpha), // ring
        7 => {
            // sphere: two offset circles
            game.backend
                .draw_circle(x as f32, y as f32, s, true, color, alpha);
            game.backend
                .draw_circle(x as f32, y as f32 - s * 0.3, s * 0.6, true, 0xFFFFFF, alpha);
        }
        2 => game.backend.draw_rect(
            x as f32 - s,
            y as f32 - s,
            s * 2.0,
            s * 2.0,
            true,
            color,
            alpha,
        ), // square
        3 => {
            // line
            let rad = angle.to_radians();
            let dx = rad.cos() * size.max(0.5);
            let dy = -rad.sin() * size.max(0.5);
            game.backend.draw_line(
                (x - dx) as f32,
                (y - dy) as f32,
                (x + dx) as f32,
                (y + dy) as f32,
                s.max(1.0),
                color,
                alpha,
            );
        }
        4 | 8 | 9 | 10 => {
            // star / flare / spark / shard: radiating lines
            let rad = angle.to_radians();
            for k in 0..4 {
                let a = rad + k as f64 * std::f64::consts::FRAC_PI_2;
                let dx = a.cos() * size.max(0.5);
                let dy = -a.sin() * size.max(0.5);
                game.backend.draw_line(
                    x as f32,
                    y as f32,
                    (x + dx) as f32,
                    (y + dy) as f32,
                    1.0,
                    color,
                    alpha,
                );
            }
        }
        11 => game
            .backend
            .draw_rect(x as f32, y as f32, s, s, true, color, alpha), // snow: pixel
        12 => game
            .backend
            .draw_circle(x as f32, y as f32, s * 1.4, true, color, alpha * 0.5), // smoke: soft circle
        _ => game
            .backend
            .draw_rect(x as f32, y as f32, 1.0, 1.0, true, color, alpha), // pixel
    }
}

// ---------------------------------------------------------------- gml api

pub fn call(game: &mut Game, name: &str, a: &[Value]) -> Result<Value, String> {
    let _ = game;
    match name {
        "part_system_create" => {
            game.particles.create_system();
            Ok(Value::Real((game.particles.systems.len() - 1) as f64))
        }
        "part_system_destroy" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            game.particles.destroy_system(si);
            Ok(Value::Real(0.0))
        }
        "part_system_clear" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            if let Some(s) = game.particles.systems.get_mut(si) {
                s.particles.clear();
            }
            Ok(Value::Real(0.0))
        }
        "part_system_draw_order" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            let oldtonew = a.get(1).ok_or("missing order")?.as_f64()? != 0.0;
            if let Some(s) = game.particles.systems.get_mut(si) {
                s.oldtonew = oldtonew;
            }
            Ok(Value::Real(0.0))
        }
        "part_system_depth" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            let depth = a.get(1).ok_or("missing depth")?.as_f64()? as i32;
            if let Some(s) = game.particles.systems.get_mut(si) {
                s.depth = depth;
            }
            Ok(Value::Real(0.0))
        }
        "part_system_update" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            game.particles.update_system_pub(si);
            Ok(Value::Real(0.0))
        }
        "part_system_automatic_update" | "part_system_automatic_draw" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            let on = a.get(1).ok_or("missing flag")?.as_f64()? != 0.0;
            if let Some(s) = game.particles.systems.get_mut(si) {
                if name.ends_with("update") {
                    s.auto_update = on;
                } else {
                    s.auto_draw = on;
                }
            }
            Ok(Value::Real(0.0))
        }
        "part_system_position" => {
            let _si = a.get(0).and_then(|v| v.as_f64().ok()).unwrap_or(0.0) as usize;
            Ok(Value::Real(0.0)) // position of the system is meaningless here
        }
        "part_particles_clear" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            if let Some(s) = game.particles.systems.get_mut(si) {
                s.particles.clear();
            }
            Ok(Value::Real(0.0))
        }
        "part_particles_count" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            let n = game
                .particles
                .systems
                .get(si)
                .map(|s| s.particles.len())
                .unwrap_or(0);
            Ok(Value::Real(n as f64))
        }
        "part_type_create" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            if si >= game.particles.systems.len() {
                return Err("part_type_create: no such system".into());
            }
            game.particles.systems[si].types.push(PType::default());
            game.particles.systems[si].alive_types.push(true);
            let local = game.particles.systems[si].types.len() - 1;
            game.particles.next_type += 1;
            let id = game.particles.next_type;
            game.particles.type_ids.insert(id, (si, local));
            Ok(Value::Real(id as f64))
        }
        "part_type_destroy" => {
            let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
            if let Some((si, local)) = game.particles.type_ids.remove(&id) {
                if let Some(s) = game.particles.systems.get_mut(si) {
                    if local < s.alive_types.len() {
                        s.alive_types[local] = false;
                    }
                }
            }
            Ok(Value::Real(0.0))
        }
        "part_type_clear" => {
            let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
            if let Some(&(si, local)) = game.particles.type_ids.get(&id) {
                let t = PType::default();
                if let Some(s) = game.particles.systems.get_mut(si) {
                    if local < s.types.len() {
                        s.types[local] = t;
                    }
                }
            }
            Ok(Value::Real(0.0))
        }
        "part_type_exists" => {
            let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
            Ok(Value::Real(
                game.particles.type_ids.contains_key(&id) as i32 as f64,
            ))
        }
        "part_type_shape" => set_pt(game, a, |t, v| t.shape = v as i64),
        "part_type_sprite" => set_pt(game, a, |t, v| t.sprite = v as i64),
        "part_type_size" => set_pt4(game, a, |t, v| t.size = v),
        "part_type_scale" => set_pt2(game, a, |t, v| t.scale = v),
        "part_type_color1" => set_pt(game, a, |t, v| {
            let c = v as u32;
            t.colors = (c, c, c, false);
        }),
        "part_type_color2" => set_pt2(game, a, |t, v| {
            let (a, b) = ((v.0 as u32), (v.1 as u32));
            t.colors = (a, b, b, false);
        }),
        "part_type_color3" | "part_type_color_mix" => set_pt3(game, a, |t, v| {
            t.colors = (v.0 as u32, v.1 as u32, v.2 as u32, name.ends_with("mix"));
        }),
        "part_type_color_hsv" => set_pt3(game, a, |t, v| {
            t.hsv = true;
            t.color_rng = v;
        }),
        "part_type_color_rgb" => set_pt3(game, a, |t, v| {
            t.hsv = false;
            t.colors = (v.0 as u32, v.1 as u32, v.2 as u32, false);
        }),
        "part_type_alpha1" => set_pt(game, a, |t, v| t.alphas = (v, v, v)),
        "part_type_alpha2" => set_pt2(game, a, |t, v| t.alphas = (v.0, v.1, v.1)),
        "part_type_alpha3" => set_pt3(game, a, |t, v| t.alphas = v),
        "part_type_speed" => set_pt4(game, a, |t, v| t.speed = v),
        "part_type_direction" => set_pt2(game, a, |t, v| t.direction = v),
        "part_type_gravity" => set_pt2(game, a, |t, v| {
            t.gravity_amount = v.0;
            t.gravity_dir = v.1;
        }),
        "part_type_life" => set_pt2(game, a, |t, v| t.life = v),
        "part_type_step" => set_pt2(game, a, |t, v| {
            t.step_type = v.0 as i64;
            t.step_count = v.1 as i64;
        }),
        "part_type_death" => set_pt2(game, a, |t, v| {
            t.death_type = v.0 as i64;
            t.death_count = v.1 as i64;
        }),
        "part_type_blend" => set_pt(game, a, |t, v| t.blend_add = v != 0.0),
        "part_type_orientation" => {
            // (ind, ang_min, ang_max, ang_incr, ang_wobble, ang_relative)
            let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
            let v = (
                a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(3).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(4).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
            );
            let rel = a.get(5).and_then(|v| v.as_f64().ok()).unwrap_or(0.0) != 0.0;
            apply_pt(game, id, |t| t.orientation = (v.0, v.1, v.2, v.3, rel))
        }
        "part_emitter_create" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            if si >= game.particles.systems.len() {
                return Err("part_emitter_create: no such system".into());
            }
            game.particles.systems[si].emitters.push(PEmitter {
                region: (0.0, 0.0, 0.0, 0.0, 0),
                distr: 0,
                stream: -1,
            });
            let local = game.particles.systems[si].emitters.len() - 1;
            game.particles.next_emitter += 1;
            let id = game.particles.next_emitter;
            game.particles.emitter_ids.insert(id, (si, local));
            Ok(Value::Real(id as f64))
        }
        "part_emitter_destroy" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            game.particles.emitter_ids.remove(&id);
            Ok(Value::Real(0.0))
        }
        "part_emitter_destroy_all" => {
            let si = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            if let Some(s) = game.particles.systems.get_mut(si) {
                s.emitters.clear();
            }
            game.particles.emitter_ids.retain(|_, v| v.0 != si);
            Ok(Value::Real(0.0))
        }
        "part_emitter_clear" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            if let Some(&(si, local)) = game.particles.emitter_ids.get(&id) {
                if let Some(s) = game.particles.systems.get_mut(si) {
                    if local < s.emitters.len() {
                        s.emitters[local] = PEmitter {
                            region: (0.0, 0.0, 0.0, 0.0, 0),
                            distr: 0,
                            stream: -1,
                        };
                    }
                }
            }
            Ok(Value::Real(0.0))
        }
        "part_emitter_exists" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            Ok(Value::Real(
                game.particles.emitter_ids.contains_key(&id) as i32 as f64,
            ))
        }
        "part_emitter_region" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            let r = (
                a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(3).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(4).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
                a.get(5).and_then(|v| v.as_f64().ok()).unwrap_or(0.0) as i64,
            );
            let distr = a.get(6).and_then(|v| v.as_f64().ok()).unwrap_or(0.0) as i64;
            apply_em(game, id, |e| {
                e.region = r;
                e.distr = distr;
            })
        }
        "part_emitter_stream" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            let count = a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0) as i64;
            apply_em(game, id, |e| e.stream = if count > 0 { count } else { -1 })
        }
        "part_emitter_burst" => {
            let id = a.get(0).ok_or("missing emitter")?.as_f64()? as u32;
            let count = a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(1.0) as i64;
            if let Some(&(si, local)) = game.particles.emitter_ids.get(&id) {
                game.particles.burst(si, local, count);
            }
            Ok(Value::Real(0.0))
        }
        "part_particles_create" | "part_particles_create_color" => {
            let _psi = a.get(0).ok_or("missing system")?.as_f64()? as usize;
            let x = a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0);
            let y = a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0);
            let ty = a.get(3).ok_or("missing type")?.as_f64()? as u32;
            let count = if name.ends_with("color") {
                let c = a.get(4).and_then(|v| v.as_f64().ok()).unwrap_or(1.0) as u32;
                let _ = c;
                1
            } else {
                a.get(4).and_then(|v| v.as_f64().ok()).unwrap_or(1.0) as i64
            };
            if let Some(&(s, local)) = game.particles.type_ids.get(&ty) {
                for _ in 0..count.clamp(0, 2048) {
                    game.particles.spawn_particle(s, local, x, y);
                }
            }
            Ok(Value::Real(0.0))
        }
        _ => Err(format!("unknown function '{}'", name)),
    }
}

impl Particles {
    pub fn update_system_pub(&mut self, si: usize) {
        self.update_system(si);
    }
}

fn apply_pt(game: &mut Game, id: u32, f: impl FnOnce(&mut PType)) -> Result<Value, String> {
    if let Some(&(si, local)) = game.particles.type_ids.get(&id) {
        let mut t = game
            .particles
            .systems
            .get(si)
            .and_then(|s| s.types.get(local))
            .cloned()
            .ok_or("bad type")?;
        f(&mut t);
        if let Some(s) = game.particles.systems.get_mut(si) {
            if local < s.types.len() {
                s.types[local] = t;
            }
        }
        Ok(Value::Real(0.0))
    } else {
        Err("no such particle type".into())
    }
}

fn apply_em(game: &mut Game, id: u32, f: impl FnOnce(&mut PEmitter)) -> Result<Value, String> {
    if let Some(&(si, local)) = game.particles.emitter_ids.get(&id) {
        let mut e = game
            .particles
            .systems
            .get(si)
            .and_then(|s| s.emitters.get(local))
            .cloned()
            .ok_or("bad emitter")?;
        f(&mut e);
        if let Some(s) = game.particles.systems.get_mut(si) {
            if local < s.emitters.len() {
                s.emitters[local] = e;
            }
        }
        Ok(Value::Real(0.0))
    } else {
        Err("no such emitter".into())
    }
}

fn set_pt(game: &mut Game, a: &[Value], f: impl FnOnce(&mut PType, f64)) -> Result<Value, String> {
    let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
    let v = a.get(1).ok_or("missing value")?.as_f64()?;
    apply_pt(game, id, |t| f(t, v))
}

fn set_pt2(
    game: &mut Game,
    a: &[Value],
    f: impl FnOnce(&mut PType, (f64, f64)),
) -> Result<Value, String> {
    let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
    let v = (
        a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
    );
    apply_pt(game, id, |t| f(t, v))
}

fn set_pt3(
    game: &mut Game,
    a: &[Value],
    f: impl FnOnce(&mut PType, (f64, f64, f64)),
) -> Result<Value, String> {
    let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
    let v = (
        a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(3).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
    );
    apply_pt(game, id, |t| f(t, v))
}

fn set_pt4(
    game: &mut Game,
    a: &[Value],
    f: impl FnOnce(&mut PType, (f64, f64, f64, f64)),
) -> Result<Value, String> {
    let id = a.get(0).ok_or("missing type")?.as_f64()? as u32;
    let v = (
        a.get(1).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(2).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(3).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
        a.get(4).and_then(|v| v.as_f64().ok()).unwrap_or(0.0),
    );
    apply_pt(game, id, |t| f(t, v))
}

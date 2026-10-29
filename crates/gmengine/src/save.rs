//! game_save / game_load: full runtime state serialized to JSON.

use crate::instance::Instance;
use crate::world::Game;
use gml::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
enum SVal {
    R(f64),
    S(String),
    A(Vec<SVal>, usize),
}

impl SVal {
    fn from_value(v: &Value) -> SVal {
        match v {
            Value::Real(x) => SVal::R(*x),
            Value::Str(s) => SVal::S(s.clone()),
            Value::Array { data, w } => SVal::A(data.iter().map(SVal::from_value).collect(), *w),
        }
    }
    fn to_value(&self) -> Value {
        match self {
            SVal::R(x) => Value::Real(*x),
            SVal::S(s) => Value::Str(s.clone()),
            SVal::A(d, w) => Value::Array {
                data: d.iter().map(SVal::to_value).collect(),
                w: *w,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SInstance {
    id: i64,
    obj: usize,
    x: f64,
    y: f64,
    xstart: f64,
    ystart: f64,
    hspeed: f64,
    vspeed: f64,
    direction: f64,
    friction: f64,
    gravity: f64,
    gravity_dir: f64,
    visible: bool,
    solid: bool,
    persistent: bool,
    depth: i32,
    sprite_index: i64,
    mask_index: i64,
    image_index: f64,
    image_speed: f64,
    image_xscale: f64,
    image_yscale: f64,
    image_angle: f64,
    image_alpha: f64,
    image_blend: u32,
    alarm: Vec<f64>,
    path_index: i64,
    path_pos: f64,
    path_speed: f64,
    timeline_index: i64,
    timeline_running: bool,
    timeline_pos: f64,
    vars: Vec<(String, SVal)>,
}

#[derive(Serialize, Deserialize)]
struct SaveGame {
    room_idx: usize,
    score: f64,
    lives: f64,
    health: f64,
    globals: Vec<(String, SVal)>,
    instances: Vec<SInstance>,
}

fn snap(i: &Instance) -> SInstance {
    SInstance {
        id: i.id,
        obj: i.obj,
        x: i.x,
        y: i.y,
        xstart: i.xstart,
        ystart: i.ystart,
        hspeed: i.hspeed,
        vspeed: i.vspeed,
        direction: i.direction,
        friction: i.friction,
        gravity: i.gravity,
        gravity_dir: i.gravity_dir,
        visible: i.visible,
        solid: i.solid,
        persistent: i.persistent,
        depth: i.depth,
        sprite_index: i.sprite_index,
        mask_index: i.mask_index,
        image_index: i.image_index,
        image_speed: i.image_speed,
        image_xscale: i.image_xscale,
        image_yscale: i.image_yscale,
        image_angle: i.image_angle,
        image_alpha: i.image_alpha,
        image_blend: i.image_blend,
        alarm: i.alarm.to_vec(),
        path_index: i.path_index,
        path_pos: i.path_pos,
        path_speed: i.path_speed,
        timeline_index: i.timeline_index,
        timeline_running: i.timeline_running,
        timeline_pos: i.timeline_pos,
        vars: i
            .vars
            .iter()
            .map(|(k, v)| (k.clone(), SVal::from_value(v)))
            .collect(),
    }
}

fn restore(i: &mut Instance, s: &SInstance) {
    i.x = s.x;
    i.y = s.y;
    i.xstart = s.xstart;
    i.ystart = s.ystart;
    i.hspeed = s.hspeed;
    i.vspeed = s.vspeed;
    i.direction = s.direction;
    i.friction = s.friction;
    i.gravity = s.gravity;
    i.gravity_dir = s.gravity_dir;
    i.visible = s.visible;
    i.solid = s.solid;
    i.persistent = s.persistent;
    i.depth = s.depth;
    i.sprite_index = s.sprite_index;
    i.mask_index = s.mask_index;
    i.image_index = s.image_index;
    i.image_speed = s.image_speed;
    i.image_xscale = s.image_xscale;
    i.image_yscale = s.image_yscale;
    i.image_angle = s.image_angle;
    i.image_alpha = s.image_alpha;
    i.image_blend = s.image_blend;
    for (k, v) in s.alarm.iter().enumerate().take(12) {
        i.alarm[k] = *v;
    }
    i.path_index = s.path_index;
    i.path_pos = s.path_pos;
    i.path_speed = s.path_speed;
    i.timeline_index = s.timeline_index;
    i.timeline_running = s.timeline_running;
    i.timeline_pos = s.timeline_pos;
    i.vars.clear();
    for (k, v) in &s.vars {
        i.vars.insert(k.clone(), v.to_value());
    }
}

pub fn save_game(game: &mut Game, fname: &str) -> Result<(), String> {
    let sg = SaveGame {
        room_idx: game.room_idx,
        score: game.score,
        lives: game.lives,
        health: game.health,
        globals: game
            .globals
            .iter()
            .map(|(k, v)| (k.clone(), SVal::from_value(v)))
            .collect(),
        instances: game
            .slots
            .iter()
            .flatten()
            .filter(|i| i.alive)
            .map(snap)
            .collect(),
    };
    let json = serde_json::to_string_pretty(&sg).map_err(|e| e.to_string())?;
    std::fs::write(fname, json).map_err(|e| format!("cannot write save file: {}", e))
}

pub fn load_game(game: &mut Game, fname: &str) -> Result<(), String> {
    let text =
        std::fs::read_to_string(fname).map_err(|e| format!("cannot read save file: {}", e))?;
    let sg: SaveGame =
        serde_json::from_str(&text).map_err(|e| format!("invalid save file: {}", e))?;
    // wipe everything
    for s in game.slots.iter_mut().flatten() {
        s.alive = false;
    }
    game.sweep();
    game.score = sg.score;
    game.lives = sg.lives;
    game.health = sg.health;
    game.globals.clear();
    for (k, v) in &sg.globals {
        game.globals.insert(k.clone(), v.to_value());
    }
    // enter the saved room WITHOUT running creation events (state comes from the save)
    if sg.room_idx < game.project.rooms.len() {
        game.room_idx = sg.room_idx;
        game.apply_room_settings();
    }
    for s in &sg.instances {
        if s.obj >= game.project.objects.len() {
            continue;
        }
        game.next_id = game.next_id.max(s.id + 1);
        let o = &game.project.objects[s.obj];
        let mut inst = Instance::new(s.id, s.obj, s.x, s.y, o);
        restore(&mut inst, s);
        game.alloc_slot_pub(inst);
    }
    Ok(())
}

//! The RGM project file format (`.gmproj`): a JSON document describing
//! sprites, sounds, backgrounds, scripts, objects and rooms — the same
//! resource model as Game Maker 8.1, in a modern, diff-friendly shape.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::Path as FsPath;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub sprites: Vec<Sprite>,
    #[serde(default)]
    pub sounds: Vec<Sound>,
    #[serde(default)]
    pub backgrounds: Vec<Background>,
    #[serde(default)]
    pub paths: Vec<Path>,
    #[serde(default)]
    pub scripts: Vec<Script>,
    #[serde(default)]
    pub timelines: Vec<Timeline>,
    #[serde(default)]
    pub fonts: Vec<Font>,
    #[serde(default)]
    pub objects: Vec<GmObject>,
    #[serde(default)]
    pub rooms: Vec<Room>,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: "new_game".into(),
            sprites: vec![],
            sounds: vec![],
            backgrounds: vec![],
            paths: vec![],
            scripts: vec![],
            timelines: vec![],
            fonts: vec![],
            objects: vec![],
            rooms: vec![Room::new("room0", 640, 480)],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sprite {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub xoffset: i32,
    pub yoffset: i32,
    pub bbox_left: i32,
    pub bbox_top: i32,
    pub bbox_right: i32,
    pub bbox_bottom: i32,
    /// Precise (per-pixel) collision checking.
    #[serde(default)]
    pub precise: bool,
    /// Base64-encoded raw RGBA frames (w*h*4 bytes each).
    #[serde(default)]
    pub frames: Vec<String>,
}

impl Sprite {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        let xoffset = (w as i32) / 2;
        let yoffset = (h as i32) / 2;
        Sprite {
            name: name.into(),
            width: w,
            height: h,
            xoffset,
            yoffset,
            bbox_left: -xoffset,
            bbox_top: -yoffset,
            bbox_right: w as i32 - 1 - xoffset,
            bbox_bottom: h as i32 - 1 - yoffset,
            precise: false,
            frames: vec![encode_rgba(&vec![0; (w * h * 4) as usize])],
        }
    }
    pub fn decode_frame(&self, idx: usize) -> Option<Vec<u8>> {
        self.frames.get(idx).and_then(|s| B64.decode(s).ok())
    }
    pub fn set_frame(&mut self, idx: usize, rgba: &[u8]) {
        if idx < self.frames.len() {
            self.frames[idx] = encode_rgba(rgba);
        }
    }
    pub fn add_frame(&mut self, rgba: &[u8]) {
        self.frames.push(encode_rgba(rgba));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sound {
    pub name: String,
    /// "wav" or "ogg"
    pub kind: String,
    /// Base64-encoded file bytes.
    pub data: String,
    #[serde(default = "default_vol")]
    pub volume: f64,
}

fn default_vol() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Background {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Base64-encoded raw RGBA.
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Script {
    pub name: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PathPoint {
    pub x: f64,
    pub y: f64,
    /// Relative speed factor for the segment starting at this point (GM: % of path speed).
    #[serde(default = "default_path_speed")]
    pub speed: f64,
}

fn default_path_speed() -> f64 {
    100.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Path {
    pub name: String,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub points: Vec<PathPoint>,
}

impl Path {
    pub fn new(name: &str) -> Self {
        Path {
            name: name.into(),
            closed: true,
            points: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimelineMoment {
    pub step: i64,
    pub code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timeline {
    pub name: String,
    #[serde(default)]
    pub moments: Vec<TimelineMoment>,
}

impl Timeline {
    pub fn new(name: &str) -> Self {
        Timeline {
            name: name.into(),
            moments: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Font {
    pub name: String,
    /// Font size in pixels (used when drawing with this font).
    pub size: f64,
    /// Base64-encoded TTF file; empty = built-in font at the given size.
    #[serde(default)]
    pub data: String,
}

impl Font {
    pub fn new(name: &str, size: f64) -> Self {
        Font {
            name: name.into(),
            size,
            data: String::new(),
        }
    }
}

/// One parameter of a D&D action. GM stores both a real and a string value;
/// which one is used depends on the field type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DndParam {
    Real(f64),
    Str(String),
    Both { real: f64, str: String },
}

/// A drag-and-drop action inside an event (GM8-style).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DndAction {
    pub lib: u32,
    pub id: u32,
    /// 0 normal, 1 begin block, 2 end block, 3 else, 4 exit, 5 repeat, 6 while, 7 until
    #[serde(default)]
    pub kind: u8,
    /// -1 = self, -2 = other, otherwise an object index
    #[serde(default = "default_neg")]
    pub applies_to: i64,
    #[serde(default)]
    pub relative: bool,
    #[serde(default)]
    pub not: bool,
    #[serde(default)]
    pub params: Vec<DndParam>,
}

impl DndAction {
    pub fn new(lib: u32, id: u32) -> Self {
        DndAction {
            lib,
            id,
            kind: 0,
            applies_to: -1,
            relative: false,
            not: false,
            params: vec![],
        }
    }
}

/// Which event a D&D action list belongs to (mirrors the code-event kinds).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EventTarget {
    Create,
    Destroy,
    Alarm { n: u8 },
    Step,
    BeginStep,
    EndStep,
    Collision { object: usize },
    Keyboard { key: i64 },
    KeyPress { key: i64 },
    KeyRelease { key: i64 },
    Mouse { kind: String },
    Draw,
    Other { kind: String },
}

impl EventTarget {
    pub fn label(&self, objects: &[GmObject]) -> String {
        match self {
            EventTarget::Create => "Create".into(),
            EventTarget::Destroy => "Destroy".into(),
            EventTarget::Alarm { n } => format!("Alarm {}", n),
            EventTarget::Step => "Step".into(),
            EventTarget::BeginStep => "Begin Step".into(),
            EventTarget::EndStep => "End Step".into(),
            EventTarget::Collision { object } => {
                let name = objects
                    .get(*object)
                    .map(|o| o.name.clone())
                    .unwrap_or_else(|| "?".into());
                format!("Collision: {}", name)
            }
            EventTarget::Keyboard { key } => format!("Keyboard: {}", key_name(*key)),
            EventTarget::KeyPress { key } => format!("Key Press: {}", key_name(*key)),
            EventTarget::KeyRelease { key } => format!("Key Release: {}", key_name(*key)),
            EventTarget::Mouse { kind } => format!("Mouse: {}", kind),
            EventTarget::Draw => "Draw".into(),
            EventTarget::Other { kind } => format!("Other: {}", kind),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GmObject {
    pub name: String,
    pub sprite_index: i64,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub solid: bool,
    #[serde(default)]
    pub persistent: bool,
    #[serde(default)]
    pub depth: i32,
    /// -1 = no parent.
    #[serde(default)]
    pub mask_index: i64,
    #[serde(default = "default_neg")]
    pub parent: i64,
    #[serde(default)]
    pub events: Vec<EventDef>,
}

fn default_true() -> bool {
    true
}
fn default_neg() -> i64 {
    -1
}

impl GmObject {
    pub fn new(name: &str) -> Self {
        GmObject {
            name: name.into(),
            sprite_index: -1,
            visible: true,
            solid: false,
            persistent: false,
            depth: 0,
            mask_index: -1,
            parent: -1,
            events: vec![],
        }
    }
}

/// One event handler on an object, tagged by type (serde internally tagged).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventDef {
    Create {
        code: String,
    },
    Destroy {
        code: String,
    },
    Alarm {
        n: u8,
        code: String,
    },
    Step {
        code: String,
    },
    BeginStep {
        code: String,
    },
    EndStep {
        code: String,
    },
    Collision {
        object: usize,
        code: String,
    },
    Keyboard {
        key: i64,
        code: String,
    },
    KeyPress {
        key: i64,
        code: String,
    },
    KeyRelease {
        key: i64,
        code: String,
    },
    /// kind: "lmb_down" | "lmb_pressed" | "lmb_released" | "rmb_down" |
    /// "rmb_pressed" | "rmb_released" | "mmb_down" | "mmb_pressed" |
    /// "mmb_released" | "glmb_down" | "glmb_pressed" | "glmb_released" |
    /// "grmb_down" | "grmb_pressed" | "grmb_released" | "gmmb_down" |
    /// "gmmb_pressed" | "gmmb_released" | "wheel_up" | "wheel_down" |
    /// "enter" | "leave" | "no_button"
    Mouse {
        kind: String,
        code: String,
    },
    Draw {
        code: String,
    },
    /// A GM8-style drag-and-drop action list.
    Actions {
        target: Box<EventTarget>,
        actions: Vec<DndAction>,
    },
    /// kind: "animation_end" | "outside_room" | "intersect_boundary" |
    /// "game_start" | "game_end" | "room_start" | "room_end" |
    /// "no_more_lives" | "no_more_health"
    Other {
        kind: String,
        code: String,
    },
}

impl EventDef {
    pub fn code(&self) -> &str {
        match self {
            EventDef::Create { code }
            | EventDef::Destroy { code }
            | EventDef::Alarm { code, .. }
            | EventDef::Step { code }
            | EventDef::BeginStep { code }
            | EventDef::EndStep { code }
            | EventDef::Collision { code, .. }
            | EventDef::Keyboard { code, .. }
            | EventDef::KeyPress { code, .. }
            | EventDef::KeyRelease { code, .. }
            | EventDef::Mouse { code, .. }
            | EventDef::Draw { code }
            | EventDef::Other { code, .. } => code,
            EventDef::Actions { .. } => "",
        }
    }
    pub fn code_mut(&mut self) -> &mut String {
        match self {
            EventDef::Create { code }
            | EventDef::Destroy { code }
            | EventDef::Alarm { code, .. }
            | EventDef::Step { code }
            | EventDef::BeginStep { code }
            | EventDef::EndStep { code }
            | EventDef::Collision { code, .. }
            | EventDef::Keyboard { code, .. }
            | EventDef::KeyPress { code, .. }
            | EventDef::KeyRelease { code, .. }
            | EventDef::Mouse { code, .. }
            | EventDef::Draw { code }
            | EventDef::Other { code, .. } => code,
            EventDef::Actions { .. } => panic!("Actions event has no code string"),
        }
    }
    pub fn label(&self, objects: &[GmObject]) -> String {
        match self {
            EventDef::Create { .. } => "Create".into(),
            EventDef::Destroy { .. } => "Destroy".into(),
            EventDef::Alarm { n, .. } => format!("Alarm {}", n),
            EventDef::Step { .. } => "Step".into(),
            EventDef::BeginStep { .. } => "Begin Step".into(),
            EventDef::EndStep { .. } => "End Step".into(),
            EventDef::Collision { object, .. } => {
                let name = objects
                    .get(*object)
                    .map(|o| o.name.clone())
                    .unwrap_or_else(|| "?".into());
                format!("Collision: {}", name)
            }
            EventDef::Keyboard { key, .. } => format!("Keyboard: {}", key_name(*key)),
            EventDef::KeyPress { key, .. } => format!("Key Press: {}", key_name(*key)),
            EventDef::KeyRelease { key, .. } => format!("Key Release: {}", key_name(*key)),
            EventDef::Mouse { kind, .. } => format!("Mouse: {}", kind),
            EventDef::Draw { .. } => "Draw".into(),
            EventDef::Actions { target, actions } => {
                format!("{} (D&D, {} actions)", target.label(objects), actions.len())
            }
            EventDef::Other { kind, .. } => format!("Other: {}", kind),
        }
    }
}

pub fn key_name(vk: i64) -> String {
    match vk {
        0 => "<no key>".into(),
        8 => "Backspace".into(),
        9 => "Tab".into(),
        13 => "Enter".into(),
        16 => "Shift".into(),
        17 => "Control".into(),
        18 => "Alt".into(),
        27 => "Escape".into(),
        32 => "Space".into(),
        33 => "Page Up".into(),
        34 => "Page Down".into(),
        35 => "End".into(),
        36 => "Home".into(),
        37 => "Left".into(),
        38 => "Up".into(),
        39 => "Right".into(),
        40 => "Down".into(),
        45 => "Insert".into(),
        46 => "Delete".into(),
        48..=57 => format!("{}", (b'0' + (vk - 48) as u8) as char),
        65..=90 => format!("{}", (b'A' + (vk - 65) as u8) as char),
        96..=105 => format!("Numpad {}", vk - 96),
        106 => "Numpad *".into(),
        107 => "Numpad +".into(),
        109 => "Numpad -".into(),
        110 => "Numpad .".into(),
        111 => "Numpad /".into(),
        112..=123 => format!("F{}", vk - 111),
        _ => format!("Key {}", vk),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInstance {
    pub x: f64,
    pub y: f64,
    pub object: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomBackground {
    #[serde(default)]
    pub visible: bool,
    #[serde(default = "default_neg")]
    pub index: i64,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub tiled_h: bool,
    #[serde(default)]
    pub tiled_v: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomView {
    #[serde(default)]
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// -1 = no following object.
    #[serde(default = "default_neg")]
    pub object: i64,
    #[serde(default = "default_border")]
    pub hborder: f64,
    #[serde(default = "default_border")]
    pub vborder: f64,
    #[serde(default)]
    pub hspeed: f64,
    #[serde(default)]
    pub vspeed: f64,
}

fn default_border() -> f64 {
    32.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomTile {
    /// Background resource used as the tileset.
    pub bg: i64,
    /// Source rect inside the background (pixels).
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    /// Position in the room (top-left).
    pub x: f64,
    pub y: f64,
    pub depth: i32,
    /// Runtime tile id (assigned on load).
    #[serde(default)]
    pub id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub name: String,
    #[serde(default)]
    pub caption: String,
    pub width: f64,
    pub height: f64,
    #[serde(default = "default_speed")]
    pub speed: f64,
    /// GM color (0xBBGGRR).
    #[serde(default)]
    pub back_color: u32,
    #[serde(default = "default_true")]
    pub show_color: bool,
    #[serde(default)]
    pub creation_code: String,
    #[serde(default)]
    pub instances: Vec<RoomInstance>,
    #[serde(default)]
    pub backgrounds: Vec<RoomBackground>,
    #[serde(default)]
    pub tiles: Vec<RoomTile>,
    /// Up to 8 views. Empty = views disabled.
    #[serde(default)]
    pub views: Vec<RoomView>,
    /// Legacy single-view field (migrated to views[0] on load).
    #[serde(default, skip_serializing)]
    pub view: Option<RoomView>,
}

fn default_speed() -> f64 {
    60.0
}

impl Room {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        Room {
            name: name.into(),
            caption: String::new(),
            width: w as f64,
            height: h as f64,
            speed: 60.0,
            back_color: 0,
            show_color: true,
            creation_code: String::new(),
            instances: vec![],
            backgrounds: vec![],
            tiles: vec![],
            views: vec![],
            view: None,
        }
    }
}

pub fn encode_rgba(rgba: &[u8]) -> String {
    B64.encode(rgba)
}

pub fn decode_rgba(s: &str) -> Vec<u8> {
    B64.decode(s).unwrap_or_default()
}

pub fn save(project: &Project, path: &FsPath) -> Result<(), String> {
    let mut p = project.clone();
    for room in p.rooms.iter_mut() {
        room.view = None; // legacy field is never saved
    }
    let json = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("cannot write {}: {}", path.display(), e))
}

/// Parse a project from JSON text (also migrates legacy single-view rooms).
pub fn parse(text: &str) -> Result<Project, String> {
    let mut project: Project =
        serde_json::from_str(text).map_err(|e| format!("invalid project file: {}", e))?;
    for room in project.rooms.iter_mut() {
        if let Some(v) = room.view.take() {
            if room.views.is_empty() {
                room.views.push(v);
            }
        }
    }
    Ok(project)
}

pub fn load(path: &FsPath) -> Result<Project, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    parse(&text)
}

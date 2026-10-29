use gmformat::EventDef;
use gml::Value;
use std::collections::HashMap;

pub const ID_BASE: i64 = 100001;
pub const SELF: i64 = -1;
pub const OTHER: i64 = -2;
pub const ALL: i64 = -3;
pub const NOONE: i64 = -4;

/// Mouse event kinds (mirrors GM8's mouse event list).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseEv {
    Button {
        button: u8,
        down: bool,
        pressed: bool,
        global: bool,
    }, // button: 0 left, 1 right, 2 middle
    WheelUp,
    WheelDown,
    Enter,
    Leave,
    NoButton,
}

/// Identifies one event handler slot on an object.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventKey {
    Create,
    Destroy,
    Alarm(u8),
    Step,
    BeginStep,
    EndStep,
    Collision(usize),
    Keyboard(i64),
    KeyPress(i64),
    KeyRelease(i64),
    Mouse(MouseEv),
    Draw,
    AnimationEnd,
    OutsideRoom,
    IntersectBoundary,
    GameStart,
    GameEnd,
    RoomStart,
    RoomEnd,
    NoMoreLives,
    NoMoreHealth,
}

impl EventKey {
    pub fn from_def(def: &EventDef) -> EventKey {
        match def {
            EventDef::Create { .. } => EventKey::Create,
            EventDef::Destroy { .. } => EventKey::Destroy,
            EventDef::Alarm { n, .. } => EventKey::Alarm(*n),
            EventDef::Step { .. } => EventKey::Step,
            EventDef::BeginStep { .. } => EventKey::BeginStep,
            EventDef::EndStep { .. } => EventKey::EndStep,
            EventDef::Collision { object, .. } => EventKey::Collision(*object),
            EventDef::Keyboard { key, .. } => EventKey::Keyboard(*key),
            EventDef::KeyPress { key, .. } => EventKey::KeyPress(*key),
            EventDef::KeyRelease { key, .. } => EventKey::KeyRelease(*key),
            EventDef::Mouse { kind, .. } => EventKey::Mouse(parse_mouse_kind(kind)),
            EventDef::Draw { .. } => EventKey::Draw,
            EventDef::Actions { target, .. } => target_key(target),
            EventDef::Other { kind, .. } => match kind.as_str() {
                "animation_end" => EventKey::AnimationEnd,
                "outside_room" => EventKey::OutsideRoom,
                "intersect_boundary" => EventKey::IntersectBoundary,
                "game_start" => EventKey::GameStart,
                "game_end" => EventKey::GameEnd,
                "room_start" => EventKey::RoomStart,
                "room_end" => EventKey::RoomEnd,
                "no_more_lives" => EventKey::NoMoreLives,
                "no_more_health" => EventKey::NoMoreHealth,
                _ => EventKey::Create,
            },
        }
    }
}

pub fn parse_mouse_kind(kind: &str) -> MouseEv {
    match kind {
        "lmb_down" => MouseEv::Button {
            button: 0,
            down: true,
            pressed: false,
            global: false,
        },
        "lmb_pressed" => MouseEv::Button {
            button: 0,
            down: false,
            pressed: true,
            global: false,
        },
        "lmb_released" => MouseEv::Button {
            button: 0,
            down: false,
            pressed: false,
            global: false,
        },
        "rmb_down" => MouseEv::Button {
            button: 1,
            down: true,
            pressed: false,
            global: false,
        },
        "rmb_pressed" => MouseEv::Button {
            button: 1,
            down: false,
            pressed: true,
            global: false,
        },
        "rmb_released" => MouseEv::Button {
            button: 1,
            down: false,
            pressed: false,
            global: false,
        },
        "mmb_down" => MouseEv::Button {
            button: 2,
            down: true,
            pressed: false,
            global: false,
        },
        "mmb_pressed" => MouseEv::Button {
            button: 2,
            down: false,
            pressed: true,
            global: false,
        },
        "mmb_released" => MouseEv::Button {
            button: 2,
            down: false,
            pressed: false,
            global: false,
        },
        "glmb_down" => MouseEv::Button {
            button: 0,
            down: true,
            pressed: false,
            global: true,
        },
        "glmb_pressed" => MouseEv::Button {
            button: 0,
            down: false,
            pressed: true,
            global: true,
        },
        "glmb_released" => MouseEv::Button {
            button: 0,
            down: false,
            pressed: false,
            global: true,
        },
        "grmb_down" => MouseEv::Button {
            button: 1,
            down: true,
            pressed: false,
            global: true,
        },
        "grmb_pressed" => MouseEv::Button {
            button: 1,
            down: false,
            pressed: true,
            global: true,
        },
        "grmb_released" => MouseEv::Button {
            button: 1,
            down: false,
            pressed: false,
            global: true,
        },
        "gmmb_down" => MouseEv::Button {
            button: 2,
            down: true,
            pressed: false,
            global: true,
        },
        "gmmb_pressed" => MouseEv::Button {
            button: 2,
            down: false,
            pressed: true,
            global: true,
        },
        "gmmb_released" => MouseEv::Button {
            button: 2,
            down: false,
            pressed: false,
            global: true,
        },
        "wheel_up" => MouseEv::WheelUp,
        "wheel_down" => MouseEv::WheelDown,
        "enter" => MouseEv::Enter,
        "leave" => MouseEv::Leave,
        "no_button" => MouseEv::NoButton,
        _ => MouseEv::NoButton,
    }
}

pub fn mouse_kind_string(ev: &MouseEv) -> String {
    match ev {
        MouseEv::Button {
            button,
            down,
            pressed,
            global,
        } => {
            let b = ["lmb", "rmb", "mmb"][*button as usize % 3];
            let state = if *pressed {
                "pressed"
            } else if *down {
                "down"
            } else {
                "released"
            };
            if *global {
                format!("g{}_{}", b, state)
            } else {
                format!("{}_{}", b, state)
            }
        }
        MouseEv::WheelUp => "wheel_up".into(),
        MouseEv::WheelDown => "wheel_down".into(),
        MouseEv::Enter => "enter".into(),
        MouseEv::Leave => "leave".into(),
        MouseEv::NoButton => "no_button".into(),
    }
}

/// A live instance in a room.
pub struct Instance {
    pub id: i64,
    pub obj: usize,
    pub x: f64,
    pub y: f64,
    pub xstart: f64,
    pub ystart: f64,
    pub xprev: f64,
    pub yprev: f64,
    pub hspeed: f64,
    pub vspeed: f64,
    /// Stored direction (GM8 keeps it even when speed is 0).
    pub direction: f64,
    pub friction: f64,
    pub gravity: f64,
    pub gravity_dir: f64,
    pub visible: bool,
    pub solid: bool,
    pub persistent: bool,
    pub depth: i32,
    pub sprite_index: i64,
    pub mask_index: i64,
    pub image_index: f64,
    pub image_speed: f64,
    pub image_xscale: f64,
    pub image_yscale: f64,
    pub image_angle: f64,
    pub image_alpha: f64,
    pub image_blend: u32,
    pub alarm: [f64; 12],
    // path following
    pub path_index: i64,
    pub path_pos: f64,
    pub path_posprev: f64,
    pub path_speed: f64,
    pub path_endaction: i64,
    pub path_absolute: bool,
    pub path_xstart: f64,
    pub path_ystart: f64,
    // timeline
    pub timeline_index: i64,
    pub timeline_running: bool,
    pub timeline_speed: f64,
    pub timeline_pos: f64,
    pub timeline_loop: bool,
    pub vars: HashMap<String, Value>,
    pub alive: bool,
    pub active: bool,
    pub hovered: bool,
}

impl Instance {
    pub fn new(id: i64, obj: usize, x: f64, y: f64, o: &gmformat::GmObject) -> Instance {
        Instance {
            id,
            obj,
            x,
            y,
            xstart: x,
            ystart: y,
            xprev: x,
            yprev: y,
            hspeed: 0.0,
            vspeed: 0.0,
            direction: 0.0,
            friction: 0.0,
            gravity: 0.0,
            gravity_dir: 270.0,
            visible: o.visible,
            solid: o.solid,
            persistent: o.persistent,
            depth: o.depth,
            sprite_index: o.sprite_index,
            mask_index: o.mask_index,
            image_index: 0.0,
            image_speed: 1.0,
            image_xscale: 1.0,
            image_yscale: 1.0,
            image_angle: 0.0,
            image_alpha: 1.0,
            image_blend: 0xFFFFFF,
            alarm: [-1.0; 12],
            path_index: -1,
            path_pos: 0.0,
            path_posprev: 0.0,
            path_speed: 0.0,
            path_endaction: 0,
            path_absolute: true,
            path_xstart: x,
            path_ystart: y,
            timeline_index: -1,
            timeline_running: false,
            timeline_speed: 1.0,
            timeline_pos: 0.0,
            timeline_loop: false,
            vars: HashMap::new(),
            alive: true,
            active: true,
            hovered: false,
        }
    }

    pub fn speed(&self) -> f64 {
        (self.hspeed * self.hspeed + self.vspeed * self.vspeed).sqrt()
    }
    pub fn direction(&self) -> f64 {
        self.direction
    }
    pub fn set_speed_dir(&mut self, speed: f64, dir: f64) {
        self.direction = dir;
        self.hspeed = speed * dir.to_radians().cos();
        self.vspeed = -speed * dir.to_radians().sin();
    }
    /// Recompute stored direction from h/v (after direct h/v writes).
    pub fn recalc_dir(&mut self) {
        self.direction = norm_deg(-self.vspeed.atan2(self.hspeed).to_degrees());
    }
}

pub fn norm_deg(d: f64) -> f64 {
    let mut d = d % 360.0;
    if d < 0.0 {
        d += 360.0;
    }
    d
}

/// Decoded sprite data for drawing and collision checking.
pub struct SpriteData {
    pub w: u32,
    pub h: u32,
    pub xoff: i32,
    pub yoff: i32,
    pub bbox: (i32, i32, i32, i32),
    pub precise: bool,
    /// Per-frame precise alpha masks (bit per pixel, row-major).
    pub masks: Vec<Vec<u64>>,
}

impl SpriteData {
    pub fn from_def(s: &gmformat::Sprite) -> SpriteData {
        let mut masks = Vec::new();
        let bits = (s.width * s.height) as usize;
        let words = bits.div_ceil(64);
        for f in 0..s.frames.len() {
            let mut m = vec![0u64; words];
            if let Some(rgba) = s.decode_frame(f) {
                for i in 0..bits {
                    if rgba[i * 4 + 3] > 127 {
                        m[i / 64] |= 1u64 << (i % 64);
                    }
                }
            }
            masks.push(m);
        }
        SpriteData {
            w: s.width,
            h: s.height,
            xoff: s.xoffset,
            yoff: s.yoffset,
            bbox: (s.bbox_left, s.bbox_top, s.bbox_right, s.bbox_bottom),
            precise: s.precise,
            masks,
        }
    }

    pub fn mask_bit(&self, frame: usize, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return false;
        }
        let i = (y as usize) * (self.w as usize) + x as usize;
        self.masks[frame][i / 64] & (1u64 << (i % 64)) != 0
    }
}

/// Precomputed path geometry for fast position lookups.
pub struct PathData {
    pub points: Vec<(f64, f64, f64)>, // (x, y, speed factor)
    pub closed: bool,
    /// Cumulative traversal cost at each point (seg length / speed factor).
    pub cost: Vec<f64>,
    pub total_cost: f64,
}

impl PathData {
    pub fn from_def(p: &gmformat::Path) -> PathData {
        let pts: Vec<(f64, f64, f64)> = p
            .points
            .iter()
            .map(|q| (q.x, q.y, q.speed.max(1.0)))
            .collect();
        let n = pts.len();
        let mut cost = vec![0.0f64; n.max(1)];
        let mut total = 0.0;
        if n > 1 {
            for i in 0..n {
                let j = if i + 1 < n {
                    i + 1
                } else if p.closed {
                    0
                } else {
                    break;
                };
                let (x1, y1, s1) = pts[i];
                let (x2, y2, _) = pts[j];
                total += ((x2 - x1).hypot(y2 - y1)) / s1;
                if j != 0 {
                    cost[j] = total;
                } else {
                    // closed loop: wrap cost bookkeeping on final segment
                    cost[0] = 0.0;
                }
            }
            if !p.closed {
                // last point cost already set
            }
        }
        PathData {
            points: pts,
            closed: p.closed,
            cost,
            total_cost: total,
        }
    }

    /// Position at fraction t in [0,1) of total cost. Returns (x, y, direction).
    pub fn point_at(&self, t: f64) -> (f64, f64, f64) {
        let n = self.points.len();
        if n == 0 {
            return (0.0, 0.0, 0.0);
        }
        if n == 1 {
            return (self.points[0].0, self.points[0].1, 0.0);
        }
        let mut t = t % 1.0;
        if t < 0.0 {
            t += 1.0;
        }
        let target = t * self.total_cost;
        // find segment
        let mut acc = 0.0;
        let segs = if self.closed { n } else { n - 1 };
        for i in 0..segs {
            let j = if i + 1 < n { i + 1 } else { 0 };
            let (x1, y1, _) = self.points[i];
            let (x2, y2, _) = self.points[j];
            let seg_cost = ((x2 - x1).hypot(y2 - y1)) / self.points[i].2;
            if acc + seg_cost >= target || i == segs - 1 {
                let f = if seg_cost > 0.0 {
                    ((target - acc) / seg_cost).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let x = x1 + (x2 - x1) * f;
                let y = y1 + (y2 - y1) * f;
                let dir = norm_deg(-(y2 - y1).atan2(x2 - x1).to_degrees());
                return (x, y, dir);
            }
            acc += seg_cost;
        }
        let (x, y, _) = self.points[n - 1];
        (x, y, 0.0)
    }

    pub fn clone_points(&self) -> Vec<(f64, f64, f64)> {
        self.points.clone()
    }

    /// Total geometric length in pixels.
    pub fn length(&self) -> f64 {
        let n = self.points.len();
        let mut len = 0.0;
        if n > 1 {
            for i in 0..n {
                let j = if i + 1 < n {
                    i + 1
                } else if self.closed {
                    0
                } else {
                    break;
                };
                len += (self.points[j].0 - self.points[i].0)
                    .hypot(self.points[j].1 - self.points[i].1);
            }
        }
        len
    }
}

/// A runtime room tile.
#[derive(Clone)]
pub struct RTile {
    pub id: i64,
    pub bg: i64,
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub x: f64,
    pub y: f64,
    pub depth: i32,
    pub visible: bool,
}

/// Point lookup on a cloned point list (avoids borrow conflicts).
pub fn path_point_at(points: &[(f64, f64, f64)], total_cost: f64, t: f64) -> (f64, f64, f64) {
    let n = points.len();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    if n == 1 {
        return (points[0].0, points[0].1, 0.0);
    }
    let mut t = t % 1.0;
    if t < 0.0 {
        t += 1.0;
    }
    let target = t * total_cost;
    let mut acc = 0.0;
    // treat as open polyline unless the first point equals the last
    let closed = points[0].0 == points[n - 1].0 && points[0].1 == points[n - 1].1;
    let segs = if closed { n - 1 } else { n - 1 };
    for i in 0..segs {
        let (x1, y1, s1) = points[i];
        let (x2, y2, _) = points[i + 1];
        let seg_cost = ((x2 - x1).hypot(y2 - y1)) / s1;
        if acc + seg_cost >= target || i == segs - 1 {
            let f = if seg_cost > 0.0 {
                ((target - acc) / seg_cost).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let x = x1 + (x2 - x1) * f;
            let y = y1 + (y2 - y1) * f;
            let dir = norm_deg(-(y2 - y1).atan2(x2 - x1).to_degrees());
            return (x, y, dir);
        }
        acc += seg_cost;
    }
    (points[n - 1].0, points[n - 1].1, 0.0)
}

/// Map a D&D event target to its engine key.
pub fn target_key(t: &gmformat::EventTarget) -> EventKey {
    use gmformat::EventTarget;
    match t {
        EventTarget::Create => EventKey::Create,
        EventTarget::Destroy => EventKey::Destroy,
        EventTarget::Alarm { n } => EventKey::Alarm(*n),
        EventTarget::Step => EventKey::Step,
        EventTarget::BeginStep => EventKey::BeginStep,
        EventTarget::EndStep => EventKey::EndStep,
        EventTarget::Collision { object } => EventKey::Collision(*object),
        EventTarget::Keyboard { key } => EventKey::Keyboard(*key),
        EventTarget::KeyPress { key } => EventKey::KeyPress(*key),
        EventTarget::KeyRelease { key } => EventKey::KeyRelease(*key),
        EventTarget::Mouse { kind } => EventKey::Mouse(parse_mouse_kind(kind)),
        EventTarget::Draw => EventKey::Draw,
        EventTarget::Other { kind } => match kind.as_str() {
            "animation_end" => EventKey::AnimationEnd,
            "outside_room" => EventKey::OutsideRoom,
            "intersect_boundary" => EventKey::IntersectBoundary,
            "game_start" => EventKey::GameStart,
            "game_end" => EventKey::GameEnd,
            "room_start" => EventKey::RoomStart,
            "room_end" => EventKey::RoomEnd,
            "no_more_lives" => EventKey::NoMoreLives,
            "no_more_health" => EventKey::NoMoreHealth,
            _ => EventKey::Create,
        },
    }
}

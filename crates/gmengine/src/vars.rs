use crate::instance::*;
use crate::world::Game;
use crate::world::ViewState;
use gml::{Value, NOONE};

fn real(x: f64) -> Value {
    Value::Real(x)
}

impl gml::Runtime for Game {
    fn read_instance_var(&mut self, inst: i64, name: &str) -> Option<Value> {
        let i = self.inst(inst)?;
        Some(match name {
            "x" => real(i.x),
            "y" => real(i.y),
            "xstart" => real(i.xstart),
            "ystart" => real(i.ystart),
            "xprevious" => real(i.xprev),
            "yprevious" => real(i.yprev),
            "hspeed" => real(i.hspeed),
            "vspeed" => real(i.vspeed),
            "speed" => real(i.speed()),
            "direction" => real(i.direction()),
            "friction" => real(i.friction),
            "gravity" => real(i.gravity),
            "gravity_direction" => real(i.gravity_dir),
            "visible" => real(i.visible as i32 as f64),
            "solid" => real(i.solid as i32 as f64),
            "persistent" => real(i.persistent as i32 as f64),
            "depth" => real(i.depth as f64),
            "sprite_index" => real(i.sprite_index as f64),
            "mask_index" => real(i.mask_index as f64),
            "image_index" => real(i.image_index),
            "image_speed" => real(i.image_speed),
            "image_xscale" => real(i.image_xscale),
            "image_yscale" => real(i.image_yscale),
            "image_angle" => real(i.image_angle),
            "image_alpha" => real(i.image_alpha),
            "image_blend" => real(i.image_blend as f64),
            "id" => real(i.id as f64),
            "object_index" => real(i.obj as f64),
            "alarm" => Value::Array {
                data: i.alarm.iter().map(|&a| real(a)).collect(),
                w: 1,
            },
            "path_index" => real(i.path_index as f64),
            "path_position" => real(i.path_pos),
            "path_position_previous" => real(i.path_posprev),
            "path_speed" => real(i.path_speed),
            "path_endaction" => real(i.path_endaction as f64),
            "path_absolute" => real(i.path_absolute as i32 as f64),
            "timeline_index" => real(i.timeline_index as f64),
            "timeline_running" => real(i.timeline_running as i32 as f64),
            "timeline_speed" => real(i.timeline_speed),
            "timeline_position" => real(i.timeline_pos),
            "timeline_loop" => real(i.timeline_loop as i32 as f64),
            "sprite_width" => {
                let spr = i.sprite_index;
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    real(self.sprites[spr as usize].w as f64 * i.image_xscale.abs())
                } else {
                    real(0.0)
                }
            }
            "sprite_height" => {
                let spr = i.sprite_index;
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    real(self.sprites[spr as usize].h as f64 * i.image_yscale.abs())
                } else {
                    real(0.0)
                }
            }
            "sprite_xoffset" => {
                let spr = i.sprite_index;
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    real(self.sprites[spr as usize].xoff as f64)
                } else {
                    real(0.0)
                }
            }
            "sprite_yoffset" => {
                let spr = i.sprite_index;
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    real(self.sprites[spr as usize].yoff as f64)
                } else {
                    real(0.0)
                }
            }
            "bbox_left" | "bbox_top" | "bbox_right" | "bbox_bottom" => {
                let spr = if i.mask_index >= 0 {
                    i.mask_index
                } else {
                    i.sprite_index
                };
                if spr >= 0 && (spr as usize) < self.sprites.len() {
                    let b = self.sprites[spr as usize].bbox;
                    real(match name {
                        "bbox_left" => b.0,
                        "bbox_top" => b.1,
                        "bbox_right" => b.2,
                        _ => b.3,
                    } as f64)
                } else {
                    real(0.0)
                }
            }
            _ => i.vars.get(name).cloned()?,
        })
    }

    fn write_instance_var(&mut self, inst: i64, name: &str, v: Value) -> Result<(), String> {
        let n_paths = self.paths.len();
        let n_timelines = self.timelines.len();
        let i = match self.inst_mut(inst) {
            Some(i) => i,
            None => {
                return Err(format!(
                    "cannot set variable '{}' on a nonexistent instance",
                    name
                ))
            }
        };
        match name {
            "id" | "object_index" | "bbox_left" | "bbox_top" | "bbox_right" | "bbox_bottom" => {
                return Err(format!("variable '{}' is read-only", name));
            }
            _ => {}
        }
        // alarm array can be written wholesale
        if name == "alarm" {
            if let Value::Array { data, .. } = &v {
                for (k, a) in i.alarm.iter_mut().enumerate() {
                    if let Some(x) = data.get(k) {
                        *a = x.as_f64().unwrap_or(-1.0);
                    }
                }
                return Ok(());
            }
            return Err("alarm must be an array".into());
        }
        macro_rules! num {
            () => {
                v.as_f64()?
            };
        }
        match name {
            "x" => i.x = num!(),
            "y" => i.y = num!(),
            "xstart" => i.xstart = num!(),
            "ystart" => i.ystart = num!(),
            "xprevious" => i.xprev = num!(),
            "yprevious" => i.yprev = num!(),
            "hspeed" => {
                i.hspeed = num!();
                i.recalc_dir();
            }
            "vspeed" => {
                i.vspeed = num!();
                i.recalc_dir();
            }
            "speed" => {
                let sp = num!();
                let d = i.direction;
                i.set_speed_dir(sp, d);
            }
            "direction" => {
                let d = num!();
                let sp = i.speed();
                i.set_speed_dir(sp, d);
            }
            "friction" => i.friction = num!(),
            "gravity" => i.gravity = num!(),
            "gravity_direction" => i.gravity_dir = num!(),
            "visible" => i.visible = num!() != 0.0,
            "solid" => i.solid = num!() != 0.0,
            "persistent" => i.persistent = num!() != 0.0,
            "depth" => i.depth = num!() as i32,
            "sprite_index" => i.sprite_index = num!() as i64,
            "mask_index" => i.mask_index = num!() as i64,
            "image_index" => i.image_index = num!(),
            "image_speed" => i.image_speed = num!(),
            "image_xscale" => i.image_xscale = num!(),
            "image_yscale" => i.image_yscale = num!(),
            "image_angle" => i.image_angle = num!(),
            "image_alpha" => i.image_alpha = num!().clamp(0.0, 1.0),
            "image_blend" => i.image_blend = num!() as u32,
            "path_index" => {
                let pi = num!() as i64;
                if pi < 0 || (pi as usize) < n_paths {
                    i.path_index = pi;
                }
            }
            "path_position" => {
                i.path_pos = num!().clamp(0.0, 1.0);
            }
            "path_position_previous" => i.path_posprev = num!(),
            "path_speed" => i.path_speed = num!(),
            "path_endaction" => i.path_endaction = num!() as i64,
            "path_absolute" => i.path_absolute = num!() != 0.0,
            "timeline_index" => {
                let ti = num!() as i64;
                if ti < 0 || (ti as usize) < n_timelines {
                    i.timeline_index = ti;
                }
            }
            "timeline_running" => i.timeline_running = num!() != 0.0,
            "timeline_speed" => i.timeline_speed = num!(),
            "timeline_position" => i.timeline_pos = num!(),
            "timeline_loop" => i.timeline_loop = num!() != 0.0,
            _ => {
                i.vars.insert(name.to_string(), v);
            }
        }
        Ok(())
    }

    fn read_user_global(&mut self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    fn write_user_global(&mut self, name: &str, v: Value) {
        self.globals.insert(name.to_string(), v);
    }

    fn read_global_builtin(&mut self, name: &str) -> Option<Value> {
        Some(match name {
            "score" => real(self.score),
            "lives" => real(self.lives),
            "health" => real(self.health),
            "room" => real(self.room_idx as f64),
            "room_first" => real(0.0),
            "room_last" => real((self.project.rooms.len().max(1) - 1) as f64),
            "room_width" => real(self.project.rooms[self.room_idx].width),
            "room_height" => real(self.project.rooms[self.room_idx].height),
            "room_speed" => real(self.room_speed_val),
            "room_caption" => Value::Str(self.project.rooms[self.room_idx].caption.clone()),
            "room_persistent" => real(0.0),
            "fps" => real(self.fps),
            "current_time" => real(self.backend.now_ms()),
            "current_second" | "current_minute" | "current_hour" | "current_day"
            | "current_weekday" | "current_month" | "current_year" => {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let (y, mo, d, h, mi, s, wd) = civil_from_unix(secs);
                real(match name {
                    "current_second" => s as f64,
                    "current_minute" => mi as f64,
                    "current_hour" => h as f64,
                    "current_day" => d as f64,
                    "current_weekday" => wd as f64,
                    "current_month" => mo as f64,
                    _ => y as f64,
                })
            }
            "mouse_x" => real(self.mouse_world().0),
            "mouse_y" => real(self.mouse_world().1),
            "mouse_button" => real(self.mouse_bits as f64),
            "keyboard_key" => real(self.last_vk as f64),
            "keyboard_lastkey" => real(self.last_vk as f64),
            "keyboard_lastchar" => Value::Str(self.last_char.clone()),
            "keyboard_string" => Value::Str(self.keyboard_string.clone()),
            "debug_mode" => real(0.0),
            "working_directory" => Value::Str(
                std::env::current_dir()
                    .map(|p| format!("{}/", p.display()))
                    .unwrap_or_default(),
            ),
            "instance_count" => real(self.alive_ids().len() as f64),
            "view_enabled" => real(self.view_enabled as i32 as f64),
            "view_current" => real(self.view_current as f64),
            "view_visible" => self.view_array(|v| v.visible as i32 as f64),
            "view_xview" => self.view_array(|v| v.x),
            "view_yview" => self.view_array(|v| v.y),
            "view_wview" => self.view_array(|v| v.w),
            "view_hview" => self.view_array(|v| v.h),
            "view_hborder" => self.view_array(|v| v.hborder),
            "view_vborder" => self.view_array(|v| v.vborder),
            "view_hspeed" => self.view_array(|v| v.hspeed),
            "view_vspeed" => self.view_array(|v| v.vspeed),
            "view_object" => self.view_array(|v| v.object as f64),
            "cursor_sprite" => real(-1.0),
            "show_score" | "show_lives" | "show_health" => real(0.0),
            _ => return None,
        })
    }

    fn write_global_builtin(&mut self, name: &str, v: Value) -> Result<(), String> {
        // view arrays arrive as whole arrays from element-wise writes
        if matches!(
            name,
            "view_visible"
                | "view_xview"
                | "view_yview"
                | "view_wview"
                | "view_hview"
                | "view_hborder"
                | "view_vborder"
                | "view_hspeed"
                | "view_vspeed"
                | "view_object"
        ) {
            if let Value::Array { data, .. } = &v {
                self.write_view_array(name, data);
                return Ok(());
            }
        }
        let n = v.as_f64()?;
        match name {
            "score" => self.score = n,
            "lives" => self.lives = n,
            "health" => self.health = n,
            "room_speed" => {
                if n >= 1.0 {
                    self.room_speed_val = n;
                }
            }
            "room_caption" => self.project.rooms[self.room_idx].caption = v.as_display_string(),
            "keyboard_string" => self.keyboard_string = v.as_display_string(),
            "keyboard_lastkey" => self.last_vk = n as i64,
            "keyboard_lastchar" => self.last_char = v.as_display_string(),
            "view_enabled" => self.view_enabled = n != 0.0,
            "view_visible" | "view_xview" | "view_yview" | "view_wview" | "view_hview"
            | "view_hborder" | "view_vborder" | "view_hspeed" | "view_vspeed" | "view_object" => {
                // element-wise writes arrive here as whole arrays
                if let Value::Array { data, .. } = &v {
                    self.write_view_array(name, data);
                    return Ok(());
                }
                let _ = n;
                return Err(format!("variable '{}' must be set per element", name));
            }
            "show_score" | "show_lives" | "show_health" | "cursor_sprite" => { /* stored dummies */
            }
            _ => return Err(format!("variable '{}' cannot be set", name)),
        }
        Ok(())
    }

    fn read_constant(&mut self, name: &str) -> Option<Value> {
        // resource names
        for (i, s) in self.project.sprites.iter().enumerate() {
            if s.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, s) in self.project.backgrounds.iter().enumerate() {
            if s.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, s) in self.project.sounds.iter().enumerate() {
            if s.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, o) in self.project.objects.iter().enumerate() {
            if o.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, p) in self.project.paths.iter().enumerate() {
            if p.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, t) in self.project.timelines.iter().enumerate() {
            if t.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, f) in self.project.fonts.iter().enumerate() {
            if f.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, r) in self.project.rooms.iter().enumerate() {
            if r.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        for (i, s) in self.project.scripts.iter().enumerate() {
            if s.name.to_ascii_lowercase() == name {
                return Some(real(i as f64));
            }
        }
        constant_table(name)
    }

    fn write_object_var(&mut self, obj: i64, name: &str, v: Value) {
        let ids = if obj == ALL {
            self.alive_ids()
        } else {
            self.ids_for_target(obj, NOONE, NOONE)
        };
        for id in ids {
            let _ = gml::Runtime::write_instance_var(self, id, name, v.clone());
        }
    }

    fn read_first_instance_var(&mut self, obj: i64, name: &str) -> Result<Value, String> {
        let ids = self.ids_for_target(obj, NOONE, NOONE);
        for id in ids {
            if let Some(v) = gml::Runtime::read_instance_var(self, id, name) {
                return Ok(v);
            }
        }
        Err(match self.project.objects.get(obj as usize) {
            Some(o) => format!("no instance of '{}' has a variable '{}'", o.name, name),
            None => format!("unknown variable '{}'", name),
        })
    }

    fn instances_matching(&mut self, target: &Value) -> Result<Vec<i64>, String> {
        match target {
            Value::Real(t) => Ok(self.ids_for_target(*t as i64, NOONE, NOONE)),
            _ => Err("invalid with() target".into()),
        }
    }

    fn is_alive(&mut self, id: i64) -> bool {
        self.inst(id).is_some()
    }

    fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        self_id: i64,
        other_id: i64,
    ) -> Result<Value, String> {
        // scripts take priority
        if self
            .scripts
            .iter()
            .any(|(k, _)| k == &name.to_ascii_lowercase())
        {
            return self.run_script_by_name(name, args, self_id, other_id);
        }
        crate::funcs::call_builtin(self, name, args, self_id, other_id)
    }
}

/// Static constant table: colors, virtual keys, mouse buttons, event ids.
pub fn constant_table(name: &str) -> Option<Value> {
    let v = match name {
        // colors (0xBBGGRR)
        "c_aqua" => 32768.0,
        "c_black" => 0.0,
        "c_blue" => 16711680.0,
        "c_dkgray" => 4210752.0,
        "c_fuchsia" => 16711935.0,
        "c_gray" => 8421504.0,
        "c_green" => 32768.0,
        "c_lime" => 65280.0,
        "c_ltgray" => 12632256.0,
        "c_maroon" => 128.0,
        "c_navy" => 8388608.0,
        "c_olive" => 32896.0,
        "c_orange" => 4235519.0,
        "c_purple" => 8388736.0,
        "c_red" => 255.0,
        "c_silver" => 12632256.0,
        "c_teal" => 8421376.0,
        "c_white" => 16777215.0,
        "c_yellow" => 65535.0,
        // mouse buttons
        "mb_none" => 0.0,
        "mb_any" => -1.0,
        "mb_left" => 1.0,
        "mb_right" => 2.0,
        "mb_middle" => 4.0,
        // virtual keys
        "vk_nokey" => 0.0,
        "vk_anykey" => 1.0,
        "vk_enter" | "vk_return" => 13.0,
        "vk_escape" => 27.0,
        "vk_space" => 32.0,
        "vk_shift" => 16.0,
        "vk_control" => 17.0,
        "vk_alt" => 18.0,
        "vk_backspace" => 8.0,
        "vk_tab" => 9.0,
        "vk_home" => 36.0,
        "vk_end" => 35.0,
        "vk_delete" => 46.0,
        "vk_insert" => 45.0,
        "vk_pageup" => 33.0,
        "vk_pagedown" => 34.0,
        "vk_pause" => 19.0,
        "vk_left" => 37.0,
        "vk_right" => 39.0,
        "vk_up" => 38.0,
        "vk_down" => 40.0,
        "vk_f1" => 112.0,
        "vk_f2" => 113.0,
        "vk_f3" => 114.0,
        "vk_f4" => 115.0,
        "vk_f5" => 116.0,
        "vk_f6" => 117.0,
        "vk_f7" => 118.0,
        "vk_f8" => 119.0,
        "vk_f9" => 120.0,
        "vk_f10" => 121.0,
        "vk_f11" => 122.0,
        "vk_f12" => 123.0,
        "vk_numpad0" => 96.0,
        "vk_numpad1" => 97.0,
        "vk_numpad2" => 98.0,
        "vk_numpad3" => 99.0,
        "vk_numpad4" => 100.0,
        "vk_numpad5" => 101.0,
        "vk_numpad6" => 102.0,
        "vk_numpad7" => 103.0,
        "vk_numpad8" => 104.0,
        "vk_numpad9" => 105.0,
        "vk_multiply" => 106.0,
        "vk_add" => 107.0,
        "vk_subtract" => 109.0,
        "vk_divide" => 111.0,
        "vk_decimal" => 110.0,
        "vk_lshift" => 160.0,
        "vk_rshift" => 161.0,
        "vk_lcontrol" => 162.0,
        "vk_rcontrol" => 163.0,
        // event type ids (for event_perform)
        "ev_create" => 0.0,
        "ev_destroy" => 1.0,
        "ev_alarm" => 2.0,
        "ev_step" => 3.0,
        "ev_collision" => 4.0,
        "ev_keyboard" => 5.0,
        "ev_mouse" => 6.0,
        "ev_other" => 7.0,
        "ev_draw" => 8.0,
        "ev_keypress" => 9.0,
        "ev_keyrelease" => 10.0,
        "ev_left_button" => 0.0,
        "ev_right_button" => 1.0,
        "ev_middle_button" => 2.0,
        "ev_no_button" => 3.0,
        "ev_left_press" => 4.0,
        "ev_right_press" => 5.0,
        "ev_middle_press" => 6.0,
        "ev_left_release" => 7.0,
        "ev_right_release" => 8.0,
        "ev_middle_release" => 9.0,
        "ev_mouse_enter" => 10.0,
        "ev_mouse_leave" => 11.0,
        "ev_mouse_wheel_up" => 12.0,
        "ev_mouse_wheel_down" => 13.0,
        "ev_global_left_button" => 50.0,
        "ev_global_right_button" => 51.0,
        "ev_global_middle_button" => 52.0,
        "ev_global_left_press" => 54.0,
        "ev_global_right_press" => 55.0,
        "ev_global_middle_press" => 56.0,
        "ev_global_left_release" => 57.0,
        "ev_global_right_release" => 58.0,
        "ev_global_middle_release" => 59.0,
        "ev_step_normal" => 0.0,
        "ev_step_begin" => 1.0,
        "ev_step_end" => 2.0,
        "ev_outside" => 0.0,
        "ev_boundary" => 1.0,
        "ev_game_start" => 2.0,
        "ev_game_end" => 3.0,
        "ev_room_start" => 4.0,
        "ev_room_end" => 5.0,
        "ev_no_more_lives" => 6.0,
        "ev_no_more_health" => 7.0,
        "ev_animation_end" => 8.0,
        "ev_user0" => 10.0,
        "ev_user1" => 11.0,
        "ev_user2" => 12.0,
        "ev_user3" => 13.0,
        // alignment
        "fa_left" | "fa_top" => 0.0,
        "fa_center" | "fa_middle" => 1.0,
        "fa_right" | "fa_bottom" => 2.0,
        // blend modes
        "bm_normal" => 0.0,
        "bm_add" => 1.0,
        "bm_max" => 2.0,
        "bm_subtract" => 3.0,
        // particle shapes
        "pt_shape_pixel" => 0.0,
        "pt_shape_disk" => 1.0,
        "pt_shape_square" => 2.0,
        "pt_shape_line" => 3.0,
        "pt_shape_star" => 4.0,
        "pt_shape_circle" => 5.0,
        "pt_shape_ring" => 6.0,
        "pt_shape_sphere" => 7.0,
        "pt_shape_flare" => 8.0,
        "pt_shape_spark" => 9.0,
        "pt_shape_shard" => 10.0,
        "pt_shape_snow" => 11.0,
        "pt_shape_smoke" => 12.0,
        "pt_shape_cloud" => 13.0,
        // emitter shapes
        "ps_shape_rectangle" => 0.0,
        "ps_shape_ellipse" => 1.0,
        "ps_shape_diamond" => 2.0,
        "ps_shape_line" => 3.0,
        // emitter distributions
        "ps_distr_linear" => 0.0,
        "ps_distr_gaussian" => 1.0,
        "ps_distr_inversegaussian" => 2.0,
        _ => return None,
    };
    Some(Value::Real(v))
}

/// (year, month, day, hour, minute, second, weekday 0=Sunday)
pub fn civil_from_unix(secs: i64) -> (i32, u32, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let s = (rem % 60) as u32;
    let wd = ((days + 4).rem_euclid(7)) as u32; // 1970-01-01 = Thursday
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d, h, mi, s, wd)
}

impl Game {
    fn view_array(&self, f: impl Fn(&ViewState) -> f64) -> Value {
        Value::Array {
            data: self.views.iter().map(|v| Value::Real(f(v))).collect(),
            w: 1,
        }
    }

    fn write_view_array(&mut self, name: &str, data: &[Value]) {
        for (k, v) in data.iter().enumerate().take(8) {
            let x = match v.as_f64() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let view = &mut self.views[k];
            match name {
                "view_visible" => view.visible = x != 0.0,
                "view_xview" => view.x = x,
                "view_yview" => view.y = x,
                "view_wview" => view.w = x,
                "view_hview" => view.h = x,
                "view_hborder" => view.hborder = x,
                "view_vborder" => view.vborder = x,
                "view_hspeed" => view.hspeed = x,
                "view_vspeed" => view.vspeed = x,
                "view_object" => view.object = x as i64,
                _ => {}
            }
        }
    }
}

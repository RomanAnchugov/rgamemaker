use crate::instance::*;
use crate::world::Game;
use gml::{Value, ALL, NOONE, SELF};

fn real(x: f64) -> Value {
    Value::Real(x)
}

fn rf(a: &[Value], i: usize) -> Result<f64, String> {
    a.get(i)
        .ok_or_else(|| format!("missing argument {}", i + 1))?
        .as_f64()
}
fn ri(a: &[Value], i: usize) -> Result<i64, String> {
    Ok(rf(a, i)? as i64)
}
fn rs(a: &[Value], i: usize) -> Result<String, String> {
    Ok(a.get(i)
        .ok_or_else(|| format!("missing argument {}", i + 1))?
        .as_display_string())
}
fn has(a: &[Value], i: usize) -> bool {
    a.get(i).is_some()
}
fn optf(a: &[Value], i: usize, def: f64) -> Result<f64, String> {
    if has(a, i) {
        rf(a, i)
    } else {
        Ok(def)
    }
}

fn ids_for(game: &Game, t: f64, self_id: i64) -> Vec<i64> {
    if t == -1.0 {
        game.ids_for_target(SELF, self_id, NOONE)
    } else {
        game.ids_for_target(t as i64, NOONE, NOONE)
    }
}

#[allow(dead_code)]
fn place_solid_at(game: &Game, id: i64, x: f64, y: f64) -> bool {
    game.place_meeting_target(id, x, y, i64::MIN, id, NOONE, true)
}

pub fn call_builtin(
    game: &mut Game,
    name: &str,
    a: Vec<Value>,
    self_id: i64,
    other_id: i64,
) -> Result<Value, String> {
    match name {
        // ================= D&D support extras =================
        "position_destroy" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let all = game.alive_ids();
            for id in all {
                if point_hits_instance(game, id, x, y) {
                    game.destroy_instance(id);
                }
            }
            Ok(real(0.0))
        }
        "position_change" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let obj = ri(&a, 2)?;
            let perform = optf(&a, 3, 1.0)? != 0.0;
            let hits: Vec<i64> = game
                .alive_ids()
                .into_iter()
                .filter(|&id| point_hits_instance(game, id, x, y))
                .collect();
            for id in hits {
                if obj >= 0 && (obj as usize) < game.project.objects.len() {
                    if perform {
                        game.run_event(id, &EventKey::Destroy, NOONE);
                    }
                    let (sprite_index, visible, solid, persistent, depth, mask_index) = {
                        let o = &game.project.objects[obj as usize];
                        (
                            o.sprite_index,
                            o.visible,
                            o.solid,
                            o.persistent,
                            o.depth,
                            o.mask_index,
                        )
                    };
                    if let Some(i) = game.inst_mut(id) {
                        i.obj = obj as usize;
                        i.sprite_index = sprite_index;
                        i.visible = visible;
                        i.solid = solid;
                        i.persistent = persistent;
                        i.depth = depth;
                        i.mask_index = mask_index;
                    }
                    if perform {
                        game.run_event(id, &EventKey::Create, NOONE);
                    }
                }
            }
            Ok(real(0.0))
        }
        "show_question" => {
            game.errors
                .push(format!("QUESTION: {} (yes assumed)", rs(&a, 0)?));
            Ok(real(1.0))
        }
        "url_open" => {
            let url = rs(&a, 0)?;
            game.errors.push(format!("OPEN URL: {}", url));
            let _ = open_url_sys(&url);
            Ok(real(0.0))
        }
        "highscore_clear" => {
            game.highscore.clear();
            Ok(real(0.0))
        }
        "highscore_add" => {
            let name = rs(&a, 0)?;
            let sc = rf(&a, 1)?;
            game.highscore.push((name, sc));
            game.highscore
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            game.highscore.truncate(10);
            Ok(real(0.0))
        }
        "highscore_add_current" => {
            let sc = game.score;
            game.highscore.push(("Player".into(), sc));
            game.highscore
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            game.highscore.truncate(10);
            Ok(real(0.0))
        }
        "highscore_position" => {
            let sc = rf(&a, 0)?;
            let pos = game
                .highscore
                .iter()
                .position(|(_, s)| *s > sc)
                .map(|p| p + 1)
                .unwrap_or(11);
            Ok(real(pos as f64))
        }
        "draw_highscore" => {
            let (x1, y1) = (rf(&a, 0)?, rf(&a, 1)?);
            let (x2, _y2) = (rf(&a, 2)?, rf(&a, 3)?);
            let rows = game.highscore.clone();
            let (c, al) = (game.draw_color, game.draw_alpha as f32);
            let mut yy = y1;
            for (i, (name, sc)) in rows.iter().enumerate() {
                let line = format!("{:2}. {:<10} {}", i + 1, name, *sc as i64);
                let (sx, sy) = game.view_screen(x1, yy);
                game.backend.draw_text(
                    game.draw_font as u32,
                    sx as f32,
                    sy as f32,
                    &line,
                    crate::world::FONT_SIZE,
                    c,
                    al,
                );
                yy += 18.0;
                if yy > _y2 {
                    break;
                }
            }
            let _ = x2;
            Ok(real(0.0))
        }
        "draw_rectangle_color" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            let c1 = optf(&a, 4, 0.0)? as u32;
            let c2 = optf(&a, 5, 0xFFFFFF as f64)? as u32;
            let outline = optf(&a, 6, 0.0)? != 0.0;
            let (sx, sy) = game.view_screen(x1.min(x2), y1.min(y2));
            // approximate the gradient with two halves
            let w = (x2 - x1).abs() / 2.0;
            game.backend.draw_rect(
                sx as f32,
                sy as f32,
                w as f32,
                (y2 - y1).abs() as f32,
                !outline,
                c1,
                game.draw_alpha as f32,
            );
            game.backend.draw_rect(
                (sx + w) as f32,
                sy as f32,
                w as f32,
                (y2 - y1).abs() as f32,
                !outline,
                c2,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }

        // ================= dynamic code =================
        "execute_string" => {
            let code = rs(&a, 0)?;
            game.run_code(&code, self_id)
        }
        "execute_file" => {
            let path = rs(&a, 0)?;
            let code = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            game.run_code(&code, self_id)
        }
        "game_save" => {
            let f = rs(&a, 0)?;
            crate::save::save_game(game, &f)?;
            Ok(real(0.0))
        }
        "game_load" => {
            let f = rs(&a, 0)?;
            crate::save::load_game(game, &f)?;
            Ok(real(0.0))
        }

        // ================= drawing extras =================
        "draw_sprite_part" => {
            let spr = ri(&a, 0)?;
            let sub = optf(&a, 1, 0.0)?;
            let left = rf(&a, 2)?;
            let top = rf(&a, 3)?;
            let w = rf(&a, 4)?;
            let h = rf(&a, 5)?;
            let x = rf(&a, 6)?;
            let y = rf(&a, 7)?;
            game.draw_sprite_part_impl(spr, sub, left, top, w, h, x, y);
            Ok(real(0.0))
        }
        "draw_sprite_part_ext" => {
            let spr = ri(&a, 0)?;
            let sub = optf(&a, 1, 0.0)?;
            let left = rf(&a, 2)?;
            let top = rf(&a, 3)?;
            let w = rf(&a, 4)?;
            let h = rf(&a, 5)?;
            let x = rf(&a, 6)?;
            let y = rf(&a, 7)?;
            let xs = optf(&a, 8, 1.0)?;
            let ys = optf(&a, 9, 1.0)?;
            game.draw_sprite_part_impl(spr, sub, left, top, w * xs, h * ys, x, y);
            Ok(real(0.0))
        }
        "draw_text_color" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let s = rs(&a, 2)?;
            let c1 = optf(&a, 3, 0xFFFFFF as f64)? as u32;
            let alpha = optf(&a, 7, 1.0)?;
            let save = game.draw_color;
            game.draw_color = c1;
            game.draw_text_impl(x, y, &s, 1.0, 1.0, 0.0);
            game.draw_color = save;
            let _ = alpha;
            Ok(real(0.0))
        }
        "draw_set_halign" => {
            game.draw_halign = ri(&a, 0)?;
            Ok(real(0.0))
        }
        "draw_set_valign" => {
            game.draw_valign = ri(&a, 0)?;
            Ok(real(0.0))
        }
        "draw_get_color" => Ok(real(game.draw_color as f64)),
        "draw_get_alpha" => Ok(real(game.draw_alpha)),
        "draw_set_font" => {
            let f = ri(&a, 0)?;
            if f < 0 || (f as usize) < game.fonts.len() {
                game.draw_font = (f + 1) as usize; // 0 = builtin, ids are 1-based here
            }
            Ok(real(0.0))
        }
        "draw_roundrect" => {
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let x2 = rf(&a, 2)?;
            let y2 = rf(&a, 3)?;
            let outline = optf(&a, 4, 0.0)? != 0.0;
            let (sx, sy) = game.view_screen(x1.min(x2), y1.min(y2));
            game.backend.draw_rect(
                sx as f32,
                sy as f32,
                (x2 - x1).abs() as f32,
                (y2 - y1).abs() as f32,
                !outline,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_triangle" => {
            let (x1, y1, x2, y2, x3, y3) = (
                rf(&a, 0)?,
                rf(&a, 1)?,
                rf(&a, 2)?,
                rf(&a, 3)?,
                rf(&a, 4)?,
                rf(&a, 5)?,
            );
            let outline = optf(&a, 6, 0.0)? != 0.0;
            let (s1x, s1y) = game.view_screen(x1, y1);
            let (s2x, s2y) = game.view_screen(x2, y2);
            let (s3x, s3y) = game.view_screen(x3, y3);
            let (c, al) = (game.draw_color, game.draw_alpha as f32);
            game.backend
                .draw_line(s1x as f32, s1y as f32, s2x as f32, s2y as f32, 1.0, c, al);
            game.backend
                .draw_line(s2x as f32, s2y as f32, s3x as f32, s3y as f32, 1.0, c, al);
            game.backend
                .draw_line(s3x as f32, s3y as f32, s1x as f32, s1y as f32, 1.0, c, al);
            if !outline {
                // crude fill: centroid lines
                let cx = (s1x + s2x + s3x) / 3.0;
                let cy = (s1y + s2y + s3y) / 3.0;
                game.backend
                    .draw_line(cx as f32, cy as f32, cx as f32, cy as f32, 1.0, c, al);
            }
            Ok(real(0.0))
        }
        "draw_arrow" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            let size = optf(&a, 4, 10.0)?;
            let (s1x, s1y) = game.view_screen(x1, y1);
            let (s2x, s2y) = game.view_screen(x2, y2);
            let (c, al) = (game.draw_color, game.draw_alpha as f32);
            game.backend
                .draw_line(s1x as f32, s1y as f32, s2x as f32, s2y as f32, 2.0, c, al);
            let ang = (s2y - s1y).atan2(s2x - s1x);
            for da in [2.6, -2.6] {
                let ax = s2x + (ang + da).cos() * size;
                let ay = s2y + (ang + da).sin() * size;
                game.backend
                    .draw_line(s2x as f32, s2y as f32, ax as f32, ay as f32, 2.0, c, al);
            }
            Ok(real(0.0))
        }
        "draw_healthbar" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            let amount = rf(&a, 4)?.clamp(0.0, 100.0);
            let back = optf(&a, 5, 0x808080 as u32 as f64)? as u32;
            let mincol = optf(&a, 6, 255.0)? as u32;
            let maxcol = optf(&a, 7, 65280.0)? as u32;
            let showback = optf(&a, 9, 1.0)? != 0.0;
            let (sx, sy) = game.view_screen(x1.min(x2), y1.min(y2));
            let w = (x2 - x1).abs();
            let h = (y2 - y1).abs().max(4.0);
            if showback {
                game.backend
                    .draw_rect(sx as f32, sy as f32, w as f32, h as f32, true, back, 1.0);
            }
            let col = if amount < 50.0 {
                merge(mincol, maxcol, amount / 50.0)
            } else {
                merge(mincol, maxcol, 1.0)
            };
            game.backend.draw_rect(
                sx as f32,
                sy as f32,
                (w * amount / 100.0) as f32,
                h as f32,
                true,
                col,
                1.0,
            );
            Ok(real(0.0))
        }

        // ================= string format / misc =================
        "string_format" => {
            let fmt_str = rs(&a, 0)?;
            let tot = optf(&a, 1, 0.0)? as usize;
            let dec = optf(&a, 2, 2.0)? as usize;
            let val = optf(&a, 3, 0.0)?;
            let mut num = format!("{:.*}", dec, val);
            if num.contains('.') {
                while num.ends_with('0') {
                    num.pop();
                }
                if num.ends_with('.') {
                    num.pop();
                }
            }
            let out = fmt_str.replace("%", &num);
            if out.len() < tot {
                Ok(Value::Str(format!(
                    "{}{}",
                    " ".repeat(tot - out.len()),
                    out
                )))
            } else {
                Ok(Value::Str(out))
            }
        }
        "move_snap" => {
            let hs = rf(&a, 0)?;
            let vs = rf(&a, 1)?;
            if let Some(i) = game.inst_mut(self_id) {
                if hs > 0.0 {
                    i.x = (i.x / hs).round() * hs;
                }
                if vs > 0.0 {
                    i.y = (i.y / vs).round() * vs;
                }
            }
            Ok(real(0.0))
        }
        "place_snapped" => {
            let hs = rf(&a, 0)?;
            let vs = rf(&a, 1)?;
            let (x, y) = game.inst(self_id).map(|i| (i.x, i.y)).unwrap_or((0.0, 0.0));
            let okx = hs <= 0.0 || (x % hs).abs() < 0.001;
            let oky = vs <= 0.0 || (y % vs).abs() < 0.001;
            Ok(real((okx && oky) as i32 as f64))
        }
        "mp_linear_step" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let sp = rf(&a, 2)?;
            let checkall = optf(&a, 3, 0.0)? != 0.0;
            let solid_only = !checkall;
            let (cx, cy) = game.inst(self_id).map(|i| (i.x, i.y)).unwrap_or((0.0, 0.0));
            let dist = (x - cx).hypot(y - cy);
            if dist <= sp {
                if game.place_meeting_target(self_id, x, y, ALL, self_id, NOONE, solid_only) {
                    return Ok(real(0.0));
                }
                if let Some(i) = game.inst_mut(self_id) {
                    i.x = x;
                    i.y = y;
                }
                return Ok(real(1.0));
            }
            let nx = cx + (x - cx) / dist * sp;
            let ny = cy + (y - cy) / dist * sp;
            if !game.place_meeting_target(self_id, nx, ny, ALL, self_id, NOONE, solid_only) {
                if let Some(i) = game.inst_mut(self_id) {
                    i.x = nx;
                    i.y = ny;
                }
                Ok(real(1.0))
            } else {
                Ok(real(0.0))
            }
        }
        "mp_potential_step" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let sp = rf(&a, 2)?;
            let checkall = optf(&a, 3, 0.0)? != 0.0;
            let solid_only = !checkall;
            let (cx, cy) = game.inst(self_id).map(|i| (i.x, i.y)).unwrap_or((0.0, 0.0));
            let base_ang = -(y - cy).atan2(x - cx).to_degrees();
            // try straight, then rotated offsets
            for da in [0.0, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0] {
                let ang = (base_ang + da).to_radians();
                let nx = cx + ang.cos() * sp;
                let ny = cy - ang.sin() * sp;
                if !game.place_meeting_target(self_id, nx, ny, ALL, self_id, NOONE, solid_only) {
                    if let Some(i) = game.inst_mut(self_id) {
                        i.x = nx;
                        i.y = ny;
                    }
                    return Ok(real(1.0));
                }
            }
            Ok(real(0.0))
        }

        // ================= dates =================
        "date_current_datetime" | "date_current_date" | "date_current_time" => {
            let secs = now_secs();
            let (y, mo, d, h, mi, s, _) = crate::vars::civil_from_unix(secs);
            Ok(real(create_datetime(y, mo, d, h, mi, s)))
        }
        "date_create_datetime" => {
            let y = ri(&a, 0)?;
            let mo = ri(&a, 1)?;
            let d = ri(&a, 2)?;
            let h = ri(&a, 3)?;
            let mi = ri(&a, 4)?;
            let s = ri(&a, 5)?;
            Ok(real(create_datetime(
                y as i32, mo as u32, d as u32, h as u32, mi as u32, s as u32,
            )))
        }
        "date_valid_datetime" => Ok(real(1.0)),
        "date_leap_year" => {
            let y = ri(&a, 0)?;
            Ok(real(
                (y % 4 == 0 && y % 100 != 0 || y % 400 == 0) as i32 as f64,
            ))
        }
        "date_days_in_month" => {
            let mo = ri(&a, 1)?;
            let y = ri(&a, 0)?;
            let days = match mo {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if y % 4 == 0 && y % 100 != 0 || y % 400 == 0 {
                        29
                    } else {
                        28
                    }
                }
                _ => 30,
            };
            Ok(real(days as f64))
        }
        "date_get_year" => Ok(real(dt_part(rf(&a, 0)?, 0) as f64)),
        "date_get_month" => Ok(real(dt_part(rf(&a, 0)?, 1) as f64)),
        "date_get_day" => Ok(real(dt_part(rf(&a, 0)?, 2) as f64)),
        "date_get_hour" => Ok(real(dt_part(rf(&a, 0)?, 3) as f64)),
        "date_get_minute" => Ok(real(dt_part(rf(&a, 0)?, 4) as f64)),
        "date_get_second" => Ok(real(dt_part(rf(&a, 0)?, 5) as f64)),
        "date_get_weekday" => Ok(real(dt_part(rf(&a, 0)?, 6) as f64)),
        "date_date_string" => {
            let dt = rf(&a, 0)?;
            Ok(Value::Str(format!(
                "{:02}.{:02}.{}",
                dt_part(dt, 2),
                dt_part(dt, 1),
                dt_part(dt, 0)
            )))
        }
        "date_time_string" => {
            let dt = rf(&a, 0)?;
            Ok(Value::Str(format!(
                "{:02}:{:02}:{:02}",
                dt_part(dt, 3),
                dt_part(dt, 4),
                dt_part(dt, 5)
            )))
        }
        "date_datetime_string" => {
            let dt = rf(&a, 0)?;
            Ok(Value::Str(format!(
                "{:02}.{:02}.{} {:02}:{:02}:{:02}",
                dt_part(dt, 2),
                dt_part(dt, 1),
                dt_part(dt, 0),
                dt_part(dt, 3),
                dt_part(dt, 4),
                dt_part(dt, 5)
            )))
        }

        // ================= fonts =================
        "font_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.project.fonts.len()) as i32 as f64,
        )),
        "font_get_name" => Ok(Value::Str(
            game.project
                .fonts
                .get(ri(&a, 0)? as usize)
                .map(|f| f.name.clone())
                .unwrap_or_default(),
        )),

        // ================= paths =================
        "path_start" => {
            let path = ri(&a, 0)?;
            let speed = rf(&a, 1)?;
            let endaction = optf(&a, 2, 0.0)? as i64;
            let absolute = optf(&a, 3, 1.0)? != 0.0;
            if path < 0 || (path as usize) >= game.paths.len() {
                return Err("path_start: no such path".into());
            }
            if let Some(i) = game.inst_mut(self_id) {
                i.path_index = path;
                i.path_speed = speed;
                i.path_endaction = endaction;
                i.path_absolute = absolute;
                i.path_pos = 0.0;
                i.path_posprev = 0.0;
                i.path_xstart = i.x;
                i.path_ystart = i.y;
            }
            Ok(real(0.0))
        }
        "path_end" => {
            if let Some(i) = game.inst_mut(self_id) {
                i.path_index = -1;
                i.path_speed = 0.0;
            }
            Ok(real(0.0))
        }
        "path_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.paths.len()) as i32 as f64,
        )),
        "path_get_name" => Ok(Value::Str(
            game.project
                .paths
                .get(ri(&a, 0)? as usize)
                .map(|p| p.name.clone())
                .unwrap_or_default(),
        )),
        "path_get_length" => Ok(real(
            game.paths
                .get(ri(&a, 0)? as usize)
                .map(|p| p.length())
                .unwrap_or(0.0),
        )),
        "path_get_number" => Ok(real(
            game.paths
                .get(ri(&a, 0)? as usize)
                .map(|p| p.points.len())
                .unwrap_or(0) as f64,
        )),
        "path_get_closed" => Ok(real(
            game.paths
                .get(ri(&a, 0)? as usize)
                .map(|p| p.closed)
                .unwrap_or(false) as i32 as f64,
        )),
        "path_get_point_x" | "path_get_point_y" | "path_get_point_speed" => {
            let pi = ri(&a, 0)? as usize;
            let n = ri(&a, 1)? as usize;
            let pt = game
                .project
                .paths
                .get(pi)
                .and_then(|p| p.points.get(n))
                .cloned();
            Ok(real(match (name, pt) {
                ("path_get_point_x", Some(p)) => p.x,
                ("path_get_point_y", Some(p)) => p.y,
                (_, Some(p)) => p.speed,
                (_, None) => 0.0,
            }))
        }
        "path_get_x" | "path_get_y" => {
            let path = ri(&a, 0)?;
            let t = rf(&a, 1)?;
            let (x, y, _) = game
                .paths
                .get(path as usize)
                .map(|p| path_point_at(&p.clone_points(), p.total_cost, t))
                .unwrap_or((0.0, 0.0, 0.0));
            Ok(real(if name == "path_get_x" { x } else { y }))
        }
        "path_draw" => {
            let path = ri(&a, 0)?;
            let pts: Vec<(f64, f64)> = game
                .project
                .paths
                .get(path as usize)
                .map(|p| p.points.iter().map(|q| (q.x, q.y)).collect())
                .unwrap_or_default();
            let (c, al) = (game.draw_color, game.draw_alpha as f32);
            for w in pts.windows(2) {
                let (s1x, s1y) = game.view_screen(w[0].0, w[0].1);
                let (s2x, s2y) = game.view_screen(w[1].0, w[1].1);
                game.backend
                    .draw_line(s1x as f32, s1y as f32, s2x as f32, s2y as f32, 2.0, c, al);
            }
            Ok(real(0.0))
        }

        // ================= timelines =================
        "timeline_start" => {
            let has_timelines = !game.timelines.is_empty();
            if let Some(i) = game.inst_mut(self_id) {
                if i.timeline_index < 0 && has_timelines {
                    i.timeline_index = 0;
                }
                i.timeline_pos = 0.0;
                i.timeline_running = true;
            }
            Ok(real(0.0))
        }
        "timeline_stop" => {
            if let Some(i) = game.inst_mut(self_id) {
                i.timeline_running = false;
            }
            Ok(real(0.0))
        }
        "timeline_pause" => {
            if let Some(i) = game.inst_mut(self_id) {
                i.timeline_running = false;
            }
            Ok(real(0.0))
        }
        "timeline_clear" => {
            if let Some(i) = game.inst_mut(self_id) {
                i.timeline_index = -1;
                i.timeline_running = false;
                i.timeline_pos = 0.0;
            }
            Ok(real(0.0))
        }
        "timeline_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.timelines.len()) as i32 as f64,
        )),
        "timeline_get_name" => Ok(Value::Str(
            game.project
                .timelines
                .get(ri(&a, 0)? as usize)
                .map(|t| t.name.clone())
                .unwrap_or_default(),
        )),
        "timeline_max_moment" => Ok(real(
            game.timelines
                .get(ri(&a, 0)? as usize)
                .and_then(|t| t.last())
                .map(|(s, _)| *s as f64)
                .unwrap_or(0.0),
        )),

        // ================= tiles =================
        "tile_add" => {
            let bg = ri(&a, 0)?;
            let left = ri(&a, 1)?;
            let top = ri(&a, 2)?;
            let width = ri(&a, 3)?;
            let height = ri(&a, 4)?;
            let x = rf(&a, 5)?;
            let y = rf(&a, 6)?;
            let depth = ri(&a, 7)?;
            let id = game.next_tile_id;
            game.next_tile_id += 1;
            game.tiles.push(crate::instance::RTile {
                id,
                bg,
                left: left as i32,
                top: top as i32,
                width: width as i32,
                height: height as i32,
                x,
                y,
                depth: depth as i32,
                visible: true,
            });
            Ok(real(id as f64))
        }
        "tile_delete" => {
            let id = ri(&a, 0)?;
            game.tiles.retain(|t| t.id != id);
            Ok(real(0.0))
        }
        "tile_exists" => {
            let id = ri(&a, 0)?;
            Ok(real(game.tiles.iter().any(|t| t.id == id) as i32 as f64))
        }
        "tile_find" => {
            let depth = ri(&a, 0)?;
            let id = ri(&a, 1)?;
            Ok(real(
                if game
                    .tiles
                    .iter()
                    .any(|t| t.id == id && t.depth == depth as i32)
                {
                    id as f64
                } else {
                    -1.0
                },
            ))
        }
        "tile_get_x"
        | "tile_get_y"
        | "tile_get_left"
        | "tile_get_top"
        | "tile_get_width"
        | "tile_get_height"
        | "tile_get_depth"
        | "tile_get_background" => {
            let id = ri(&a, 0)?;
            let t = game
                .tiles
                .iter()
                .find(|t| t.id == id)
                .ok_or("no such tile")?;
            Ok(real(match name {
                "tile_get_x" => t.x,
                "tile_get_y" => t.y,
                "tile_get_left" => t.left as f64,
                "tile_get_top" => t.top as f64,
                "tile_get_width" => t.width as f64,
                "tile_get_height" => t.height as f64,
                "tile_get_depth" => t.depth as f64,
                _ => t.bg as f64,
            }))
        }
        "tile_set_position" => {
            let id = ri(&a, 0)?;
            let x = rf(&a, 1)?;
            let y = rf(&a, 2)?;
            if let Some(t) = game.tiles.iter_mut().find(|t| t.id == id) {
                t.x = x;
                t.y = y;
            }
            Ok(real(0.0))
        }
        "tile_set_region" => {
            let id = ri(&a, 0)?;
            let left = ri(&a, 1)?;
            let top = ri(&a, 2)?;
            let width = ri(&a, 3)?;
            let height = ri(&a, 4)?;
            if let Some(t) = game.tiles.iter_mut().find(|t| t.id == id) {
                t.left = left as i32;
                t.top = top as i32;
                t.width = width as i32;
                t.height = height as i32;
            }
            Ok(real(0.0))
        }
        "tile_set_background" => {
            let id = ri(&a, 0)?;
            let bg = ri(&a, 1)?;
            if let Some(t) = game.tiles.iter_mut().find(|t| t.id == id) {
                t.bg = bg;
            }
            Ok(real(0.0))
        }
        "tile_set_depth" => {
            let id = ri(&a, 0)?;
            let depth = ri(&a, 1)?;
            if let Some(t) = game.tiles.iter_mut().find(|t| t.id == id) {
                t.depth = depth as i32;
            }
            Ok(real(0.0))
        }
        "tile_set_visible" => {
            let id = ri(&a, 0)?;
            let vis = optf(&a, 1, 1.0)? != 0.0;
            if let Some(t) = game.tiles.iter_mut().find(|t| t.id == id) {
                t.visible = vis;
            }
            Ok(real(0.0))
        }
        "tile_layer_show" | "tile_layer_hide" => {
            let depth = ri(&a, 0)?;
            let show = name.ends_with("show");
            for t in game.tiles.iter_mut() {
                if t.depth == depth as i32 {
                    t.visible = show;
                }
            }
            Ok(real(0.0))
        }
        "tile_layer_delete" => {
            let depth = ri(&a, 0)?;
            game.tiles.retain(|t| t.depth != depth as i32);
            Ok(real(0.0))
        }
        "tile_layer_shift" => {
            let depth = ri(&a, 0)?;
            let dx = rf(&a, 1)?;
            let dy = rf(&a, 2)?;
            for t in game.tiles.iter_mut() {
                if t.depth == depth as i32 {
                    t.x += dx;
                    t.y += dy;
                }
            }
            Ok(real(0.0))
        }
        "tile_layer_depth" => {
            let depth = ri(&a, 0)?;
            let newdepth = ri(&a, 1)?;
            for t in game.tiles.iter_mut() {
                if t.depth == depth as i32 {
                    t.depth = newdepth as i32;
                }
            }
            Ok(real(0.0))
        }
        "tile_layer_find" => {
            let depth = ri(&a, 0)?;
            let id = ri(&a, 1)?;
            Ok(real(
                if game
                    .tiles
                    .iter()
                    .any(|t| t.id == id && t.depth == depth as i32)
                {
                    id as f64
                } else {
                    -1.0
                },
            ))
        }
        "tile_layer_count" => {
            let depth = ri(&a, 0)?;
            Ok(real(
                game.tiles
                    .iter()
                    .filter(|t| t.depth == depth as i32)
                    .count() as f64,
            ))
        }

        // ================= math =================
        "abs" => Ok(real(rf(&a, 0)?.abs())),
        "sqrt" => Ok(real(rf(&a, 0)?.sqrt())),
        "sqr" => {
            let x = rf(&a, 0)?;
            Ok(real(x * x))
        }
        "sin" => Ok(real(rf(&a, 0)?.to_radians().sin())),
        "cos" => Ok(real(rf(&a, 0)?.to_radians().cos())),
        "tan" => Ok(real(rf(&a, 0)?.to_radians().tan())),
        "arcsin" => Ok(real(rf(&a, 0)?.asin().to_degrees())),
        "arccos" => Ok(real(rf(&a, 0)?.acos().to_degrees())),
        "arctan" => Ok(real(rf(&a, 0)?.atan().to_degrees())),
        "arctan2" => {
            // GM8: arctan2(x, y) — note the argument order!
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            Ok(real(y.atan2(x).to_degrees()))
        }
        "exp" => Ok(real(rf(&a, 0)?.exp())),
        "ln" => Ok(real(rf(&a, 0)?.ln())),
        "log10" => Ok(real(rf(&a, 0)?.log10())),
        "log2" => Ok(real(rf(&a, 0)?.log2())),
        "power" => {
            let x = rf(&a, 0)?;
            let p = rf(&a, 1)?;
            Ok(real(x.powf(p)))
        }
        "floor" => Ok(real(rf(&a, 0)?.floor())),
        "ceil" => Ok(real(rf(&a, 0)?.ceil())),
        "round" => Ok(real(rf(&a, 0)?.round())),
        "sign" => {
            let x = rf(&a, 0)?;
            Ok(real(if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                0.0
            }))
        }
        "frac" => {
            let x = rf(&a, 0)?;
            Ok(real(x - x.trunc()))
        }
        "random" => {
            let n = rf(&a, 0)?;
            Ok(real(game.rand() * n))
        }
        "random_range" => {
            let lo = rf(&a, 0)?;
            let hi = rf(&a, 1)?;
            Ok(real(lo + game.rand() * (hi - lo)))
        }
        "irandom" => Ok(real(game.rand() * (rf(&a, 0)? + 1.0) as i64 as f64)),
        "irandom_range" => {
            let lo = rf(&a, 0)?;
            let hi = rf(&a, 1)?;
            Ok(real(lo + (game.rand() * (hi - lo + 1.0)) as i64 as f64))
        }
        "choose" => {
            if a.is_empty() {
                return Err("choose needs at least one argument".into());
            }
            let idx = (game.rand() * a.len() as f64) as usize;
            Ok(a[idx.min(a.len() - 1)].clone())
        }
        "random_set_seed" => {
            let s = rf(&a, 0)?;
            game.rng = (s as u64) | 1;
            Ok(real(0.0))
        }
        "random_get_seed" => Ok(real(game.rng as f64)),
        "randomize" => {
            game.rng = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x1234)
                | 1;
            Ok(real(0.0))
        }
        "min" => {
            let mut m = f64::INFINITY;
            for v in &a {
                m = m.min(v.as_f64()?);
            }
            Ok(real(m))
        }
        "max" => {
            let mut m = f64::NEG_INFINITY;
            for v in &a {
                m = m.max(v.as_f64()?);
            }
            Ok(real(m))
        }
        "mean" => {
            if a.is_empty() {
                return Err("mean needs arguments".into());
            }
            let mut s = 0.0;
            for v in &a {
                s += v.as_f64()?;
            }
            Ok(real(s / a.len() as f64))
        }
        "median" => {
            let mut xs: Vec<f64> = Vec::new();
            for v in &a {
                xs.push(v.as_f64()?);
            }
            xs.sort_by(|p, q| p.partial_cmp(q).unwrap());
            Ok(real(xs[xs.len() / 2]))
        }
        "point_distance" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            Ok(real(((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()))
        }
        "point_direction" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            Ok(real(norm_deg(-(y2 - y1).atan2(x2 - x1).to_degrees())))
        }
        "lengthdir_x" => {
            let (len, dir) = (rf(&a, 0)?, rf(&a, 1)?);
            Ok(real(len * dir.to_radians().cos()))
        }
        "lengthdir_y" => {
            let (len, dir) = (rf(&a, 0)?, rf(&a, 1)?);
            Ok(real(-len * dir.to_radians().sin()))
        }
        "is_real" => Ok(real(matches!(a.get(0), Some(Value::Real(_))) as i32 as f64)),
        "is_string" => Ok(real(matches!(a.get(0), Some(Value::Str(_))) as i32 as f64)),

        // ================= strings =================
        "string" => Ok(Value::Str(rs(&a, 0)?)),
        "string_length" => Ok(real(rs(&a, 0)?.chars().count() as f64)),
        "string_copy" => {
            let s = rs(&a, 0)?;
            let idx = rf(&a, 1)?.max(1.0) as usize - 1;
            let count = rf(&a, 2)?.max(0.0) as usize;
            Ok(Value::Str(s.chars().skip(idx).take(count).collect()))
        }
        "string_pos" => {
            let sub = rs(&a, 0)?;
            let s = rs(&a, 1)?;
            let pos = s
                .find(&sub)
                .map(|b| s[..b].chars().count() + 1)
                .unwrap_or(0);
            Ok(real(pos as f64))
        }
        "string_char_at" => {
            let s = rs(&a, 0)?;
            let idx = rf(&a, 1)?.max(1.0) as usize - 1;
            Ok(Value::Str(
                s.chars()
                    .nth(idx)
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            ))
        }
        "string_delete" => {
            let s = rs(&a, 0)?;
            let idx = rf(&a, 1)?.max(1.0) as usize - 1;
            let count = rf(&a, 2)?.max(0.0) as usize;
            let mut out: String = s.chars().take(idx).collect();
            out.extend(s.chars().skip(idx + count));
            Ok(Value::Str(out))
        }
        "string_insert" => {
            let sub = rs(&a, 0)?;
            let s = rs(&a, 1)?;
            let idx = rf(&a, 2)?.max(1.0) as usize - 1;
            let mut out: String = s.chars().take(idx).collect();
            out.push_str(&sub);
            out.extend(s.chars().skip(idx));
            Ok(Value::Str(out))
        }
        "string_replace" => {
            let s = rs(&a, 0)?;
            let sub = rs(&a, 1)?;
            let new = rs(&a, 2)?;
            Ok(Value::Str(s.replacen(&sub, &new, 1)))
        }
        "string_replace_all" => {
            let s = rs(&a, 0)?;
            let sub = rs(&a, 1)?;
            let new = rs(&a, 2)?;
            Ok(Value::Str(s.replace(&sub, &new)))
        }
        "string_count" => {
            let sub = rs(&a, 0)?;
            let s = rs(&a, 1)?;
            if sub.is_empty() {
                return Ok(real(0.0));
            }
            Ok(real(s.matches(&sub).count() as f64))
        }
        "string_lower" => Ok(Value::Str(rs(&a, 0)?.to_lowercase())),
        "string_upper" => Ok(Value::Str(rs(&a, 0)?.to_uppercase())),
        "string_repeat" => {
            let s = rs(&a, 0)?;
            let n = rf(&a, 1)?.clamp(0.0, 100_000.0) as usize;
            Ok(Value::Str(s.repeat(n)))
        }
        "string_letters" => Ok(Value::Str(
            rs(&a, 0)?.chars().filter(|c| c.is_alphabetic()).collect(),
        )),
        "string_digits" => Ok(Value::Str(
            rs(&a, 0)?.chars().filter(|c| c.is_ascii_digit()).collect(),
        )),
        "string_lettersdigits" => Ok(Value::Str(
            rs(&a, 0)?.chars().filter(|c| c.is_alphanumeric()).collect(),
        )),
        "chr" => {
            let c = ri(&a, 0)?;
            Ok(Value::Str(
                char::from_u32(c as u32).unwrap_or('?').to_string(),
            ))
        }
        "ord" => {
            let s = rs(&a, 0)?;
            Ok(real(
                s.chars().next().map(|c| c as u32 as f64).unwrap_or(0.0),
            ))
        }
        "real" => {
            let s = rs(&a, 0)?;
            s.trim()
                .parse::<f64>()
                .map(real)
                .map_err(|_| format!("cannot convert '{}' to a number", s))
        }
        "string_width" => {
            let s = rs(&a, 0)?;
            Ok(real(
                game.backend
                    .text_width(game.draw_font as u32, &s, crate::world::FONT_SIZE)
                    as f64,
            ))
        }
        "string_height" => Ok(real(
            game.backend
                .text_height(game.draw_font as u32, crate::world::FONT_SIZE) as f64,
        )),

        // ================= colors =================
        "make_color_rgb" => {
            let (r, g, b) = (rf(&a, 0)? as u32, rf(&a, 1)? as u32, rf(&a, 2)? as u32);
            Ok(real((r | (g << 8) | (b << 16)) as f64))
        }
        "make_color_hsv" => {
            let (h, s, v) = (rf(&a, 0)? as i32, rf(&a, 1)? as u32, rf(&a, 2)? as u32);
            let (r, g, b) = hsv_to_rgb((h.rem_euclid(360)) as u32, s.min(255), v.min(255));
            Ok(real(
                (r as u32 | ((g as u32) << 8) | ((b as u32) << 16)) as f64,
            ))
        }
        "color_get_red" => Ok(real((rf(&a, 0)? as u32 & 0xFF) as f64)),
        "color_get_green" => Ok(real(((rf(&a, 0)? as u32 >> 8) & 0xFF) as f64)),
        "color_get_blue" => Ok(real(((rf(&a, 0)? as u32 >> 16) & 0xFF) as f64)),
        "merge_color" => {
            let c1 = rf(&a, 0)? as u32;
            let c2 = rf(&a, 1)? as u32;
            let t = rf(&a, 2)?.clamp(0.0, 1.0);
            let mix = |s1: u32, s2: u32| -> u32 {
                (s1 as f64 * (1.0 - t) + s2 as f64 * t).round() as u32
            };
            Ok(real(
                (mix(c1 & 0xFF, c2 & 0xFF)
                    | (mix((c1 >> 8) & 0xFF, (c2 >> 8) & 0xFF) << 8)
                    | (mix((c1 >> 16) & 0xFF, (c2 >> 16) & 0xFF) << 16)) as f64,
            ))
        }

        // ================= rooms / game =================
        "room_goto" => {
            let idx = ri(&a, 0)?;
            if idx < 0 || idx as usize >= game.project.rooms.len() {
                return Err("room_goto: no such room".into());
            }
            game.room_goto(idx as usize);
            Ok(real(0.0))
        }
        "room_goto_next" => {
            game.room_goto_next();
            Ok(real(0.0))
        }
        "room_goto_previous" => {
            game.room_goto_previous();
            Ok(real(0.0))
        }
        "room_restart" => {
            game.room_restart_pending();
            Ok(real(0.0))
        }
        "room_next" => {
            let idx = ri(&a, 0)?;
            Ok(real(if idx as usize + 1 < game.project.rooms.len() {
                idx as f64 + 1.0
            } else {
                -1.0
            }))
        }
        "room_previous" => {
            let idx = ri(&a, 0)?;
            Ok(real(if idx > 0 { idx as f64 - 1.0 } else { -1.0 }))
        }
        "room_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.project.rooms.len()) as i32 as f64,
        )),
        "room_get_name" => {
            let idx = ri(&a, 0)?;
            Ok(Value::Str(
                game.project
                    .rooms
                    .get(idx as usize)
                    .map(|r| r.name.clone())
                    .unwrap_or_default(),
            ))
        }
        "game_end" => {
            game.game_end();
            Ok(real(0.0))
        }
        "game_restart" => {
            game.game_restart();
            Ok(real(0.0))
        }
        "sleep" => {
            let ms = rf(&a, 0)?;
            std::thread::sleep(std::time::Duration::from_millis(ms.max(0.0) as u64));
            Ok(real(0.0))
        }
        "event_perform" => {
            let ty = ri(&a, 0)?;
            let numb = ri(&a, 1)?;
            let key = perform_key(ty, numb)?;
            game.run_event(self_id, &key, other_id);
            Ok(real(0.0))
        }

        // ================= instances =================
        "instance_create" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let obj = ri(&a, 2)?;
            if obj < 0 || obj as usize >= game.project.objects.len() {
                return Err(format!("instance_create: no object with index {}", obj));
            }
            let id = game.create_instance(x, y, obj as usize)?;
            Ok(real(id as f64))
        }
        "instance_destroy" => {
            game.destroy_instance(self_id);
            Ok(real(0.0))
        }
        "instance_exists" => {
            let t = rf(&a, 0)?;
            Ok(real(!ids_for(game, t, self_id).is_empty() as i32 as f64))
        }
        "instance_number" => {
            let t = rf(&a, 0)?;
            Ok(real(ids_for(game, t, self_id).len() as f64))
        }
        "instance_find" => {
            let t = rf(&a, 0)?;
            let n = ri(&a, 1)?;
            let ids = ids_for(game, t, self_id);
            Ok(match ids.get(n.max(0) as usize) {
                Some(&id) => real(id as f64),
                None => real(NOONE as f64),
            })
        }
        "instance_nearest" | "instance_furthest" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            let nearest = name == "instance_nearest";
            let mut best: Option<(f64, i64)> = None;
            for id in ids_for(game, t, self_id) {
                let (ix, iy) = game.inst(id).map(|i| (i.x, i.y)).unwrap_or((0.0, 0.0));
                let d = (ix - x).hypot(iy - y);
                best = match best {
                    None => Some((d, id)),
                    Some((bd, _)) => Some(if (d < bd) == nearest {
                        (d, id)
                    } else {
                        (bd, best.unwrap().1)
                    }),
                };
            }
            Ok(real(best.map(|(_, id)| id as f64).unwrap_or(NOONE as f64)))
        }
        "instance_position" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            for id in ids_for(game, t, self_id) {
                if point_hits_instance(game, id, x, y) {
                    return Ok(real(id as f64));
                }
            }
            Ok(real(NOONE as f64))
        }
        "instance_place" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            for id in ids_for(game, t, self_id) {
                if id != self_id
                    && game.place_meeting_target(self_id, x, y, id as i64, self_id, NOONE, false)
                {
                    return Ok(real(id as f64));
                }
            }
            Ok(real(NOONE as f64))
        }
        "instance_copy" => {
            let perform = optf(&a, 0, 1.0)? != 0.0;
            let (x, y, obj, snapshot) = match game.inst(self_id) {
                Some(i) => (i.x, i.y, i.obj, instance_snapshot(i)),
                None => return Err("instance_copy: no self instance".into()),
            };
            let id = game.create_instance(x, y, obj)?;
            if let Some(i) = game.inst_mut(id) {
                restore_snapshot(i, &snapshot);
            }
            if perform {
                game.run_event(id, &EventKey::Create, NOONE);
            }
            Ok(real(id as f64))
        }
        "instance_change" => {
            let obj = ri(&a, 0)?;
            let perform = optf(&a, 1, 1.0)? != 0.0;
            if obj < 0 || obj as usize >= game.project.objects.len() {
                return Err("instance_change: no such object".into());
            }
            if perform {
                game.run_event(self_id, &EventKey::Destroy, NOONE);
            }
            let (sprite_index, visible, solid, persistent, depth, mask_index) = {
                let o = &game.project.objects[obj as usize];
                (
                    o.sprite_index,
                    o.visible,
                    o.solid,
                    o.persistent,
                    o.depth,
                    o.mask_index,
                )
            };
            if let Some(i) = game.inst_mut(self_id) {
                i.obj = obj as usize;
                i.sprite_index = sprite_index;
                i.visible = visible;
                i.solid = solid;
                i.persistent = persistent;
                i.depth = depth;
                i.mask_index = mask_index;
            }
            if perform {
                game.run_event(self_id, &EventKey::Create, NOONE);
            }
            Ok(real(0.0))
        }
        "instance_deactivate_object" => {
            let t = rf(&a, 0)?;
            for id in ids_for(game, t, self_id) {
                if let Some(i) = game.inst_mut(id) {
                    i.active = false;
                }
            }
            Ok(real(0.0))
        }
        "instance_activate_object" => {
            let t = rf(&a, 0)?;
            for id in ids_for(game, t, self_id) {
                if let Some(i) = game.inst_mut(id) {
                    i.active = true;
                }
            }
            Ok(real(0.0))
        }
        "position_meeting" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            for id in ids_for(game, t, self_id) {
                if id != self_id && point_hits_instance(game, id, x, y) {
                    return Ok(real(1.0));
                }
            }
            Ok(real(0.0))
        }
        "position_empty" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let all = game.alive_ids();
            for id in all {
                if id != self_id && point_hits_instance(game, id, x, y) {
                    return Ok(real(0.0));
                }
            }
            Ok(real(1.0))
        }
        "instance_deactivate_region" | "instance_activate_region" => {
            // minimal support: rectangle + inside only
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let w = rf(&a, 2)?;
            let h = rf(&a, 3)?;
            let deactivate = name.starts_with("instance_deact");
            for id in game.alive_ids() {
                let inside = game
                    .inst(id)
                    .map(|i| i.x >= x1 && i.x <= x1 + w && i.y >= y1 && i.y <= y1 + h)
                    .unwrap_or(false);
                if inside {
                    if let Some(i) = game.inst_mut(id) {
                        i.active = !deactivate;
                    }
                }
            }
            Ok(real(0.0))
        }
        "place_meeting" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            Ok(real(
                game.place_meeting_target(self_id, x, y, t as i64, self_id, NOONE, false) as i32
                    as f64,
            ))
        }
        "place_empty" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            for id in game.alive_ids() {
                if id != self_id
                    && game.place_meeting_target(self_id, x, y, id as i64, self_id, NOONE, false)
                {
                    return Ok(real(0.0));
                }
            }
            Ok(real(1.0))
        }
        "place_free" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            Ok(real(
                !game.place_meeting_target(self_id, x, y, ALL, self_id, NOONE, true) as i32 as f64,
            ))
        }
        "motion_set" => {
            let dir = rf(&a, 0)?;
            let sp = rf(&a, 1)?;
            if let Some(i) = game.inst_mut(self_id) {
                i.set_speed_dir(sp, dir);
            }
            Ok(real(0.0))
        }
        "motion_add" => {
            let dir = rf(&a, 0)?;
            let sp = rf(&a, 1)?;
            if let Some(i) = game.inst_mut(self_id) {
                i.hspeed += sp * dir.to_radians().cos();
                i.vspeed -= sp * dir.to_radians().sin();
                i.recalc_dir();
            }
            Ok(real(0.0))
        }
        "move_towards_point" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let sp = rf(&a, 2)?;
            if let Some(i) = game.inst_mut(self_id) {
                let dir = norm_deg(-(y - i.y).atan2(x - i.x).to_degrees());
                i.set_speed_dir(sp, dir);
            }
            Ok(real(0.0))
        }
        "move_contact_solid" | "move_contact_all" => {
            let dir = rf(&a, 0)?;
            let maxd = rf(&a, 1)?;
            let solid_only = name.ends_with("solid");
            let maxd = if maxd <= 0.0 { 1_000_000.0 } else { maxd };
            let dx = dir.to_radians().cos();
            let dy = -dir.to_radians().sin();
            let (mut cx, mut cy) = game.inst(self_id).map(|i| (i.x, i.y)).unwrap_or((0.0, 0.0));
            let mut moved = 0.0;
            while moved < maxd {
                let nx = cx + dx;
                let ny = cy + dy;
                let blocked = game.alive_ids().iter().any(|&id| {
                    id != self_id
                        && game.place_meeting_target(
                            self_id, nx, ny, id as i64, self_id, NOONE, solid_only,
                        )
                });
                if blocked {
                    break;
                }
                cx = nx;
                cy = ny;
                moved += 1.0;
            }
            if let Some(i) = game.inst_mut(self_id) {
                i.x = cx;
                i.y = cy;
            }
            Ok(real(0.0))
        }
        "move_out_of_solid" | "move_out_of_all" => {
            let dir = rf(&a, 0)?;
            let maxd = optf(&a, 1, -1.0)?;
            let solid_only = name.ends_with("solid");
            let maxd = if maxd <= 0.0 { 1_000_000.0 } else { maxd };
            let dx = dir.to_radians().cos();
            let dy = -dir.to_radians().sin();
            let mut moved = 0.0;
            while moved < maxd {
                let still_inside = game.alive_ids().iter().any(|&id| {
                    id != self_id
                        && game.place_meeting_target(
                            self_id,
                            game.inst(self_id).map(|i| i.x).unwrap_or(0.0),
                            game.inst(self_id).map(|i| i.y).unwrap_or(0.0),
                            id as i64,
                            self_id,
                            NOONE,
                            solid_only,
                        )
                });
                if !still_inside {
                    break;
                }
                if let Some(i) = game.inst_mut(self_id) {
                    i.x += dx;
                    i.y += dy;
                }
                moved += 1.0;
            }
            Ok(real(0.0))
        }
        "move_bounce_solid" | "move_bounce_all" => {
            let solid_only = name.ends_with("solid");
            let (hs, vs, x, y) = match game.inst(self_id) {
                Some(i) => (i.hspeed, i.vspeed, i.x, i.y),
                None => return Ok(real(0.0)),
            };
            let block = |x: f64, y: f64| -> bool {
                game.alive_ids().iter().any(|&id| {
                    id != self_id
                        && game.place_meeting_target(
                            self_id, x, y, id as i64, self_id, NOONE, solid_only,
                        )
                })
            };
            let hx = block(x + hs, y);
            let vy = block(x, y + vs);
            let both = block(x + hs, y + vs) || block(x, y);
            if let Some(i) = game.inst_mut(self_id) {
                if hx && !vy {
                    i.hspeed = -i.hspeed;
                } else if vy && !hx {
                    i.vspeed = -i.vspeed;
                } else if both {
                    i.hspeed = -i.hspeed;
                    i.vspeed = -i.vspeed;
                }
            }
            Ok(real(0.0))
        }
        "move_random" => {
            let hs = rf(&a, 0)?;
            let vs = rf(&a, 1)?;
            let rw = game.project.rooms[game.room_idx].width;
            let rh = game.project.rooms[game.room_idx].height;
            let nx = if hs > 0.0 {
                (game.rand() * (rw / hs).floor()).floor() * hs + hs / 2.0
            } else {
                f64::NAN
            };
            let ny = if vs > 0.0 {
                (game.rand() * (rh / vs).floor()).floor() * vs + vs / 2.0
            } else {
                f64::NAN
            };
            if let Some(i) = game.inst_mut(self_id) {
                if !nx.is_nan() {
                    i.x = nx;
                }
                if !ny.is_nan() {
                    i.y = ny;
                }
            }
            Ok(real(0.0))
        }
        "distance_to_point" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let (l, t, r, b) = game.bbox(self_id);
            let cx = x.clamp(l, r);
            let cy = y.clamp(t, b);
            Ok(real((cx - x).hypot(cy - y)))
        }
        "distance_to_object" => {
            let t = rf(&a, 0)?;
            let (l, t2, r, b) = game.bbox(self_id);
            let mut best = f64::INFINITY;
            for id in ids_for(game, t, self_id) {
                if id == self_id {
                    continue;
                }
                let (l2, t3, r2, b2) = game.bbox(id);
                let dx = (l2 - r).max(0.0).max(l - r2);
                let dy = (t3 - b).max(0.0).max(t2 - b2);
                best = best.min(dx.hypot(dy));
            }
            Ok(real(best))
        }
        "collision_point" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let t = rf(&a, 2)?;
            for id in ids_for(game, t, self_id) {
                if id != self_id && point_hits_instance(game, id, x, y) {
                    return Ok(real(id as f64));
                }
            }
            Ok(real(NOONE as f64))
        }
        "collision_rectangle" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            let t = rf(&a, 4)?;
            for id in ids_for(game, t, self_id) {
                if id == self_id {
                    continue;
                }
                let (l, tp, r, b) = game.bbox(id);
                if r >= x1.min(x2) && l <= x2.max(x1) && b >= y1.min(y2) && tp <= y2.max(y1) {
                    return Ok(real(id as f64));
                }
            }
            Ok(real(NOONE as f64))
        }
        "collision_circle" | "collision_ellipse" => {
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let x2 = rf(&a, 2)?;
            let y2 = rf(&a, 3)?;
            let t = rf(&a, 4)?;
            for id in ids_for(game, t, self_id) {
                if id == self_id {
                    continue;
                }
                let (l, tp, r, b) = game.bbox(id);
                let hit = if name == "collision_circle" {
                    let cx = (x1 + x2) / 2.0;
                    let cy = (y1 + y2) / 2.0;
                    let rad = ((x2 - x1).hypot(y2 - y1)) / 2.0;
                    (cx - l.max(cx).min(r)).hypot(cy - tp.max(cy).min(b)) <= rad
                } else {
                    r >= x1.min(x2) && l <= x2.max(x1) && b >= y1.min(y2) && tp <= y2.max(y1)
                };
                if hit {
                    return Ok(real(id as f64));
                }
            }
            Ok(real(NOONE as f64))
        }
        "collision_line" => {
            let (x1, y1, x2, y2) = (rf(&a, 0)?, rf(&a, 1)?, rf(&a, 2)?, rf(&a, 3)?);
            let t = rf(&a, 4)?;
            let steps = (((x2 - x1).hypot(y2 - y1)) as usize).max(1);
            for s in 0..=steps {
                let f = s as f64 / steps as f64;
                let px = x1 + (x2 - x1) * f;
                let py = y1 + (y2 - y1) * f;
                for id in ids_for(game, t, self_id) {
                    if id != self_id && point_hits_instance(game, id, px, py) {
                        return Ok(real(id as f64));
                    }
                }
            }
            Ok(real(NOONE as f64))
        }

        // ================= keyboard / mouse =================
        "keyboard_check" => Ok(real(game.key_down(ri(&a, 0)?) as i32 as f64)),
        "keyboard_check_pressed" => Ok(real(game.key_pressed(ri(&a, 0)?) as i32 as f64)),
        "keyboard_check_released" => Ok(real(game.key_released(ri(&a, 0)?) as i32 as f64)),
        "keyboard_clear" => {
            let vk = ri(&a, 0)?;
            game.clear_key(vk); // proxied input; next frame refreshes
            Ok(real(0.0))
        }
        "mouse_check_button" => Ok(real(
            (game.mouse_bits & rf(&a, 0)? as u8 != 0) as i32 as f64,
        )),
        "mouse_check_button_pressed" => Ok(real(
            (game.mouse_pressed_bits & rf(&a, 0)? as u8 != 0) as i32 as f64,
        )),
        "mouse_check_button_released" => Ok(real(
            (game.mouse_released_bits & rf(&a, 0)? as u8 != 0) as i32 as f64,
        )),
        "mouse_clear" => {
            game.mouse_pressed_bits = 0;
            Ok(real(0.0))
        }
        "mouse_wheel_up" => Ok(real((game.wheel > 0) as i32 as f64)),
        "mouse_wheel_down" => Ok(real((game.wheel < 0) as i32 as f64)),
        "io_clear" => {
            game.keys_pressed.clear();
            game.keys_released.clear();
            game.mouse_pressed_bits = 0;
            game.mouse_released_bits = 0;
            Ok(real(0.0))
        }

        // ================= sounds =================
        "sound_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.sounds.len()) as i32 as f64,
        )),
        "sound_play" | "sound_loop" => {
            let idx = ri(&a, 0)?;
            if idx < 0 || idx as usize >= game.sounds.len() {
                return Err(format!("sound_play: no sound {}", idx));
            }
            let s = &game.sounds[idx as usize];
            let (bid, vol) = (s.backend_id, s.volume);
            game.backend.play_sound(bid, name == "sound_loop", vol);
            Ok(real(idx as f64))
        }
        "sound_stop" => {
            let idx = ri(&a, 0)?;
            if idx >= 0 && (idx as usize) < game.sounds.len() {
                game.backend
                    .stop_sound(game.sounds[idx as usize].backend_id);
            }
            Ok(real(0.0))
        }
        "sound_stop_all" => {
            for s in game.sounds.clone() {
                game.backend.stop_sound(s.backend_id);
            }
            Ok(real(0.0))
        }
        "sound_isplaying" => {
            let idx = ri(&a, 0)?;
            if idx >= 0 && (idx as usize) < game.sounds.len() {
                let bid = game.sounds[idx as usize].backend_id;
                return Ok(real(game.backend.sound_playing(bid) as i32 as f64));
            }
            Ok(real(0.0))
        }
        "sound_volume" => {
            let idx = ri(&a, 0)?;
            let vol = rf(&a, 1)?.clamp(0.0, 1.0);
            if idx >= 0 && (idx as usize) < game.sounds.len() {
                game.sounds[idx as usize].volume = vol;
            }
            Ok(real(0.0))
        }

        // ================= resources info =================
        "sprite_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.sprites.len()) as i32 as f64,
        )),
        "sprite_get_name" => Ok(Value::Str(
            game.project
                .sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.name.clone())
                .unwrap_or_default(),
        )),
        "sprite_get_number" => Ok(real(
            game.sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.masks.len())
                .unwrap_or(0) as f64,
        )),
        "sprite_get_width" => Ok(real(
            game.sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.w)
                .unwrap_or(0) as f64,
        )),
        "sprite_get_height" => Ok(real(
            game.sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.h)
                .unwrap_or(0) as f64,
        )),
        "sprite_get_xoffset" => Ok(real(
            game.sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.xoff)
                .unwrap_or(0) as f64,
        )),
        "sprite_get_yoffset" => Ok(real(
            game.sprites
                .get(ri(&a, 0)? as usize)
                .map(|s| s.yoff)
                .unwrap_or(0) as f64,
        )),
        "background_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.bg_tex.len()) as i32 as f64,
        )),
        "background_get_name" => Ok(Value::Str(
            game.project
                .backgrounds
                .get(ri(&a, 0)? as usize)
                .map(|b| b.name.clone())
                .unwrap_or_default(),
        )),
        "background_get_width" => Ok(real(
            game.project
                .backgrounds
                .get(ri(&a, 0)? as usize)
                .map(|b| b.width)
                .unwrap_or(0) as f64,
        )),
        "background_get_height" => Ok(real(
            game.project
                .backgrounds
                .get(ri(&a, 0)? as usize)
                .map(|b| b.height)
                .unwrap_or(0) as f64,
        )),
        "object_exists" => Ok(real(
            ((ri(&a, 0)? as usize) < game.project.objects.len()) as i32 as f64,
        )),
        "object_get_name" => Ok(Value::Str(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.name.clone())
                .unwrap_or_default(),
        )),
        "object_get_sprite" => Ok(real(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.sprite_index)
                .unwrap_or(-1) as f64,
        )),
        "object_get_solid" => Ok(real(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.solid)
                .unwrap_or(false) as i32 as f64,
        )),
        "object_get_visible" => Ok(real(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.visible)
                .unwrap_or(false) as i32 as f64,
        )),
        "object_get_depth" => Ok(real(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.depth)
                .unwrap_or(0) as f64,
        )),
        "object_get_parent" => Ok(real(
            game.project
                .objects
                .get(ri(&a, 0)? as usize)
                .map(|o| o.parent)
                .unwrap_or(-1) as f64,
        )),

        // ================= drawing =================
        "draw_sprite" => {
            let spr = ri(&a, 0)?;
            let sub = optf(&a, 1, 0.0)?;
            let x = rf(&a, 2)?;
            let y = rf(&a, 3)?;
            game.draw_sprite_impl(spr, sub, x, y, 1.0, 1.0, 0.0, 0xFFFFFF, 1.0);
            Ok(real(0.0))
        }
        "draw_sprite_ext" => {
            let spr = ri(&a, 0)?;
            let sub = optf(&a, 1, 0.0)?;
            let x = rf(&a, 2)?;
            let y = rf(&a, 3)?;
            let xs = optf(&a, 4, 1.0)?;
            let ys = optf(&a, 5, 1.0)?;
            let rot = optf(&a, 6, 0.0)?;
            let col = optf(&a, 7, 0xFFFFFF as f64)? as u32;
            let alpha = optf(&a, 8, 1.0)?;
            game.draw_sprite_impl(spr, sub, x, y, xs, ys, rot, col, alpha);
            Ok(real(0.0))
        }
        "draw_sprite_stretched" => {
            let spr = ri(&a, 0)?;
            let sub = optf(&a, 1, 0.0)?;
            let x = rf(&a, 2)?;
            let y = rf(&a, 3)?;
            let w = rf(&a, 4)?;
            let h = rf(&a, 5)?;
            if spr >= 0 && (spr as usize) < game.sprites.len() {
                let s = &game.sprites[spr as usize];
                game.draw_sprite_impl(
                    spr,
                    sub,
                    x,
                    y,
                    w / s.w as f64,
                    h / s.h as f64,
                    0.0,
                    0xFFFFFF,
                    1.0,
                );
            }
            Ok(real(0.0))
        }
        "draw_background" => {
            let bg = ri(&a, 0)?;
            let x = rf(&a, 1)?;
            let y = rf(&a, 2)?;
            if bg >= 0 && (bg as usize) < game.bg_tex.len() {
                let (w, h) = {
                    let b = &game.project.backgrounds[bg as usize];
                    (b.width as f64, b.height as f64)
                };
                let (sx, sy) = game.view_screen(x, y);
                let tex = game.bg_tex[bg as usize];
                game.backend.draw_sprite(
                    tex, sx as f32, sy as f32, w as f32, h as f32, 0.0, 1.0, 0xFFFFFF, false,
                    false, None,
                );
            }
            Ok(real(0.0))
        }
        "draw_background_ext" => {
            let bg = ri(&a, 0)?;
            let x = rf(&a, 1)?;
            let y = rf(&a, 2)?;
            let xs = optf(&a, 3, 1.0)?;
            let ys = optf(&a, 4, 1.0)?;
            let rot = optf(&a, 5, 0.0)?;
            let col = optf(&a, 6, 0xFFFFFF as f64)? as u32;
            let alpha = optf(&a, 7, 1.0)?;
            if bg >= 0 && (bg as usize) < game.bg_tex.len() {
                let (w, h) = {
                    let b = &game.project.backgrounds[bg as usize];
                    (b.width as f64, b.height as f64)
                };
                let (sx, sy) = game.view_screen(x, y);
                let tex = game.bg_tex[bg as usize];
                game.backend.draw_sprite(
                    tex,
                    sx as f32,
                    sy as f32,
                    (w * xs) as f32,
                    (h * ys) as f32,
                    rot as f32,
                    alpha as f32,
                    col,
                    false,
                    false,
                    None,
                );
            }
            Ok(real(0.0))
        }
        "draw_text" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let s = rs(&a, 2)?;
            game.draw_text_impl(x, y, &s, 1.0, 1.0, 0.0);
            Ok(real(0.0))
        }
        "draw_text_ext" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let s = rs(&a, 2)?;
            let sep = optf(&a, 3, -1.0)?;
            let w = optf(&a, 4, -1.0)?;
            let lines = if w > 0.0 {
                wrap_text(game, &s, w)
            } else {
                s.split('#').map(|l| l.to_string()).collect()
            };
            let lh = if sep > 0.0 {
                sep
            } else {
                game.backend
                    .text_height(game.draw_font as u32, crate::world::FONT_SIZE)
                    as f64
            };
            for (i, line) in lines.iter().enumerate() {
                game.draw_text_impl(x, y + i as f64 * lh, line, 1.0, 1.0, 0.0);
            }
            Ok(real(0.0))
        }
        "draw_text_transformed" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let s = rs(&a, 2)?;
            let xs = optf(&a, 3, 1.0)?;
            let ys = optf(&a, 4, 1.0)?;
            let rot = optf(&a, 5, 0.0)?;
            game.draw_text_impl(x, y, &s, xs, ys, rot);
            Ok(real(0.0))
        }
        "draw_point" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let (sx, sy) = game.view_screen(x, y);
            game.backend.draw_rect(
                sx as f32,
                sy as f32,
                1.0,
                1.0,
                true,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_line" => {
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let x2 = rf(&a, 2)?;
            let y2 = rf(&a, 3)?;
            let (sx1, sy1) = game.view_screen(x1, y1);
            let (sx2, sy2) = game.view_screen(x2, y2);
            game.backend.draw_line(
                sx1 as f32,
                sy1 as f32,
                sx2 as f32,
                sy2 as f32,
                1.0,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_rectangle" => {
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let x2 = rf(&a, 2)?;
            let y2 = rf(&a, 3)?;
            let outline = optf(&a, 4, 0.0)? != 0.0;
            let (sx, sy) = game.view_screen(x1.min(x2), y1.min(y2));
            game.backend.draw_rect(
                sx as f32,
                sy as f32,
                (x2 - x1).abs() as f32,
                (y2 - y1).abs() as f32,
                !outline,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_circle" => {
            let x = rf(&a, 0)?;
            let y = rf(&a, 1)?;
            let r = rf(&a, 2)?;
            let outline = optf(&a, 3, 0.0)? != 0.0;
            let (sx, sy) = game.view_screen(x, y);
            game.backend.draw_circle(
                sx as f32,
                sy as f32,
                r as f32,
                !outline,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_ellipse" => {
            let x1 = rf(&a, 0)?;
            let y1 = rf(&a, 1)?;
            let x2 = rf(&a, 2)?;
            let y2 = rf(&a, 3)?;
            let outline = optf(&a, 4, 0.0)? != 0.0;
            let cx = (x1 + x2) / 2.0;
            let cy = (y1 + y2) / 2.0;
            let (sx, sy) = game.view_screen(cx, cy);
            game.backend.draw_ellipse(
                sx as f32,
                sy as f32,
                ((x2 - x1) / 2.0).abs() as f32,
                ((y2 - y1) / 2.0).abs() as f32,
                !outline,
                game.draw_color,
                game.draw_alpha as f32,
            );
            Ok(real(0.0))
        }
        "draw_set_color" => {
            game.draw_color = rf(&a, 0)? as u32;
            Ok(real(0.0))
        }
        "draw_set_alpha" => {
            game.draw_alpha = rf(&a, 0)?.clamp(0.0, 1.0);
            Ok(real(0.0))
        }
        "draw_clear" => {
            game.backend.clear(rf(&a, 0)? as u32);
            Ok(real(0.0))
        }

        // ================= data structures =================
        "ds_list_create" => {
            game.ds_counter += 1;
            game.ds_lists.insert(game.ds_counter, Vec::new());
            Ok(real(game.ds_counter as f64))
        }
        "ds_list_destroy" => {
            game.ds_lists.remove(&(ri(&a, 0)? as u64));
            Ok(real(0.0))
        }
        "ds_list_clear" => {
            if let Some(l) = game.ds_lists.get_mut(&(ri(&a, 0)? as u64)) {
                l.clear();
            }
            Ok(real(0.0))
        }
        "ds_list_add" => {
            let id = ri(&a, 0)? as u64;
            if let Some(l) = game.ds_lists.get_mut(&id) {
                for v in a.iter().skip(1) {
                    l.push(v.clone());
                }
            }
            Ok(real(0.0))
        }
        "ds_list_insert" => {
            let id = ri(&a, 0)? as u64;
            let pos = ri(&a, 1)? as usize;
            if let Some(l) = game.ds_lists.get_mut(&id) {
                if pos <= l.len() {
                    l.insert(pos, a.get(2).cloned().unwrap_or(Value::Real(0.0)));
                }
            }
            Ok(real(0.0))
        }
        "ds_list_replace" => {
            let id = ri(&a, 0)? as u64;
            let pos = ri(&a, 1)? as usize;
            if let Some(l) = game.ds_lists.get_mut(&id) {
                if pos < l.len() {
                    l[pos] = a.get(2).cloned().unwrap_or(Value::Real(0.0));
                }
            }
            Ok(real(0.0))
        }
        "ds_list_delete" => {
            let id = ri(&a, 0)? as u64;
            let pos = ri(&a, 1)? as usize;
            if let Some(l) = game.ds_lists.get_mut(&id) {
                if pos < l.len() {
                    l.remove(pos);
                }
            }
            Ok(real(0.0))
        }
        "ds_list_size" => Ok(real(
            game.ds_lists
                .get(&(ri(&a, 0)? as u64))
                .map(|l| l.len())
                .unwrap_or(0) as f64,
        )),
        "ds_list_find_value" => {
            let id = ri(&a, 0)? as u64;
            let pos = ri(&a, 1)? as usize;
            Ok(game
                .ds_lists
                .get(&id)
                .and_then(|l| l.get(pos))
                .cloned()
                .unwrap_or(Value::Real(0.0)))
        }
        "ds_list_find_index" => {
            let id = ri(&a, 0)? as u64;
            let v = a.get(1).cloned().unwrap_or(Value::Real(0.0));
            Ok(real(
                game.ds_lists
                    .get(&id)
                    .and_then(|l| l.iter().position(|x| *x == v))
                    .unwrap_or(0) as f64,
            ))
        }
        "ds_map_create" => {
            game.ds_counter += 1;
            game.ds_maps.insert(game.ds_counter, Vec::new());
            Ok(real(game.ds_counter as f64))
        }
        "ds_map_destroy" => {
            game.ds_maps.remove(&(ri(&a, 0)? as u64));
            Ok(real(0.0))
        }
        "ds_map_clear" => {
            if let Some(m) = game.ds_maps.get_mut(&(ri(&a, 0)? as u64)) {
                m.clear();
            }
            Ok(real(0.0))
        }
        "ds_map_add" => {
            let id = ri(&a, 0)? as u64;
            let k = a.get(1).cloned().ok_or("missing key")?;
            let v = a.get(2).cloned().ok_or("missing value")?;
            if let Some(m) = game.ds_maps.get_mut(&id) {
                if !m.iter().any(|(mk, _)| *mk == k) {
                    m.push((k, v));
                }
            }
            Ok(real(0.0))
        }
        "ds_map_replace" => {
            let id = ri(&a, 0)? as u64;
            let k = a.get(1).cloned().ok_or("missing key")?;
            let v = a.get(2).cloned().ok_or("missing value")?;
            if let Some(m) = game.ds_maps.get_mut(&id) {
                for (mk, mv) in m.iter_mut() {
                    if *mk == k {
                        *mv = v;
                        break;
                    }
                }
            }
            Ok(real(0.0))
        }
        "ds_map_delete" => {
            let id = ri(&a, 0)? as u64;
            let k = a.get(1).cloned();
            if let Some(m) = game.ds_maps.get_mut(&id) {
                if let Some(k) = k {
                    m.retain(|(mk, _)| *mk != k);
                }
            }
            Ok(real(0.0))
        }
        "ds_map_exists" => {
            let id = ri(&a, 0)? as u64;
            let k = a.get(1).cloned();
            Ok(real(
                game.ds_maps
                    .get(&id)
                    .map(|m| k.map(|k| m.iter().any(|(mk, _)| *mk == k)).unwrap_or(false))
                    .unwrap_or(false) as i32 as f64,
            ))
        }
        "ds_map_find_value" => {
            let id = ri(&a, 0)? as u64;
            let k = a.get(1).cloned();
            Ok(game
                .ds_maps
                .get(&id)
                .and_then(|m| {
                    m.iter()
                        .find(|(mk, _)| Some(mk.clone()) == k)
                        .map(|(_, v)| v.clone())
                })
                .unwrap_or(Value::Real(0.0)))
        }
        "ds_map_size" => Ok(real(
            game.ds_maps
                .get(&(ri(&a, 0)? as u64))
                .map(|m| m.len())
                .unwrap_or(0) as f64,
        )),
        "ds_stack_create" => {
            game.ds_counter += 1;
            game.ds_stacks.insert(game.ds_counter, Vec::new());
            Ok(real(game.ds_counter as f64))
        }
        "ds_stack_destroy" => {
            game.ds_stacks.remove(&(ri(&a, 0)? as u64));
            Ok(real(0.0))
        }
        "ds_stack_clear" => {
            if let Some(s) = game.ds_stacks.get_mut(&(ri(&a, 0)? as u64)) {
                s.clear();
            }
            Ok(real(0.0))
        }
        "ds_stack_push" => {
            let id = ri(&a, 0)? as u64;
            if let Some(s) = game.ds_stacks.get_mut(&id) {
                s.push(a.get(1).cloned().unwrap_or(Value::Real(0.0)));
            }
            Ok(real(0.0))
        }
        "ds_stack_pop" => {
            let id = ri(&a, 0)? as u64;
            Ok(game
                .ds_stacks
                .get_mut(&id)
                .and_then(|s| s.pop())
                .unwrap_or(Value::Real(0.0)))
        }
        "ds_stack_top" => {
            let id = ri(&a, 0)? as u64;
            Ok(game
                .ds_stacks
                .get(&id)
                .and_then(|s| s.last())
                .cloned()
                .unwrap_or(Value::Real(0.0)))
        }
        "ds_stack_size" => Ok(real(
            game.ds_stacks
                .get(&(ri(&a, 0)? as u64))
                .map(|s| s.len())
                .unwrap_or(0) as f64,
        )),

        // ================= variables by name =================
        "variable_global_exists" => {
            let n = rs(&a, 0)?;
            Ok(real(game.globals.contains_key(&n) as i32 as f64))
        }
        "variable_global_get" => {
            let n = rs(&a, 0)?;
            game.globals
                .get(&n)
                .cloned()
                .ok_or_else(|| format!("unknown global variable '{}'", n))
        }
        "variable_global_set" => {
            let n = rs(&a, 0)?;
            let v = a.get(1).cloned().ok_or("missing value")?;
            game.globals.insert(n, v);
            Ok(real(0.0))
        }
        "variable_local_exists" => {
            let n = rs(&a, 0)?;
            Ok(real(
                gml::Runtime::read_instance_var(game, self_id, &n).is_some() as i32 as f64,
            ))
        }
        "variable_local_get" => {
            let n = rs(&a, 0)?;
            gml::Runtime::read_instance_var(game, self_id, &n)
                .ok_or_else(|| format!("unknown variable '{}'", n))
        }
        "variable_local_set" => {
            let n = rs(&a, 0)?;
            let v = a.get(1).cloned().ok_or("missing value")?;
            gml::Runtime::write_instance_var(game, self_id, &n, v)?;
            Ok(real(0.0))
        }

        // ================= misc =================
        "show_debug_message" => {
            let msg = rs(&a, 0)?;
            game.debug_log.push(msg.clone());
            if game.debug_log.len() > 8 {
                game.debug_log.remove(0);
            }
            game.errors.push(format!("DEBUG: {}", msg));
            Ok(real(0.0))
        }
        "show_message" => {
            game.errors.push(format!("MESSAGE: {}", rs(&a, 0)?));
            Ok(real(0.0))
        }
        "window_set_caption" => {
            game.project.rooms[game.room_idx].caption = rs(&a, 0)?;
            Ok(real(0.0))
        }
        "display_get_width" => Ok(real(1924.0)),
        "display_get_height" => Ok(real(1080.0)),
        "window_get_width" => Ok(real(game.project.rooms[game.room_idx].width)),
        "window_get_height" => Ok(real(game.project.rooms[game.room_idx].height)),

        _ => {
            // particle system functions
            if name.starts_with("part_") {
                return crate::particles::call(game, name, &a);
            }
            Err(format!("unknown function '{}'", name))
        }
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn create_datetime(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> f64 {
    // encode as yyyymmddhhmmss (decimal, like GM's internal datetime number)
    (y as f64) * 10_000_000_000.0
        + (mo as f64) * 100_000_000.0
        + (d as f64) * 1_000_000.0
        + (h as f64) * 10_000.0
        + (mi as f64) * 100.0
        + (s as f64)
}

/// Extract part of an encoded datetime: 0=year 1=month 2=day 3=hour 4=min 5=sec 6=weekday
fn dt_part(dt: f64, part: usize) -> i64 {
    let dt = dt as i64;
    let (y, mo, d, h, mi, s) = (
        dt / 10_000_000_000,
        (dt / 100_000_000) % 100,
        (dt / 1_000_000) % 100,
        (dt / 10_000) % 100,
        (dt / 100) % 100,
        dt % 100,
    );
    match part {
        0 => y,
        1 => mo,
        2 => d,
        3 => h,
        4 => mi,
        5 => s,
        _ => 0,
    }
}

fn merge(c1: u32, c2: u32, t: f64) -> u32 {
    let mix = |a: u32, b: u32| -> u32 { (a as f64 * (1.0 - t) + b as f64 * t).round() as u32 };
    mix(c1 & 0xFF, c2 & 0xFF)
        | (mix((c1 >> 8) & 0xFF, (c2 >> 8) & 0xFF) << 8)
        | (mix((c1 >> 16) & 0xFF, (c2 >> 16) & 0xFF) << 16)
}

fn point_hits_instance(game: &Game, id: i64, x: f64, y: f64) -> bool {
    let i = match game.inst(id) {
        Some(i) => i,
        None => return false,
    };
    let spr = if i.mask_index >= 0 {
        i.mask_index
    } else {
        i.sprite_index
    };
    if spr < 0 || (spr as usize) >= game.sprites.len() {
        return false;
    }
    let s = &game.sprites[spr as usize];
    let l = i.x + s.bbox.0 as f64;
    let t = i.y + s.bbox.1 as f64;
    let r = i.x + s.bbox.2 as f64;
    let b = i.y + s.bbox.3 as f64;
    if x < l || x > r || y < t || y > b {
        return false;
    }
    if s.precise {
        let f = game.frame_index_pub(i);
        let px = (x - (i.x + s.xoff as f64)) as i32;
        let py = (y - (i.y + s.yoff as f64)) as i32;
        return s.mask_bit(f, px, py);
    }
    true
}

fn perform_key(ty: i64, numb: i64) -> Result<EventKey, String> {
    Ok(match ty {
        0 => EventKey::Create,
        1 => EventKey::Destroy,
        2 => EventKey::Alarm(numb.max(0).min(11) as u8),
        3 => match numb {
            1 => EventKey::BeginStep,
            2 => EventKey::EndStep,
            _ => EventKey::Step,
        },
        4 => EventKey::Collision(numb.max(0) as usize),
        5 => EventKey::Keyboard(numb),
        9 => EventKey::KeyPress(numb),
        10 => EventKey::KeyRelease(numb),
        6 => EventKey::Mouse(match numb {
            0 => MouseEv::Button {
                button: 0,
                down: true,
                pressed: false,
                global: false,
            },
            1 => MouseEv::Button {
                button: 1,
                down: true,
                pressed: false,
                global: false,
            },
            2 => MouseEv::Button {
                button: 2,
                down: true,
                pressed: false,
                global: false,
            },
            3 => MouseEv::NoButton,
            4 => MouseEv::Button {
                button: 0,
                down: false,
                pressed: true,
                global: false,
            },
            5 => MouseEv::Button {
                button: 1,
                down: false,
                pressed: true,
                global: false,
            },
            6 => MouseEv::Button {
                button: 2,
                down: false,
                pressed: true,
                global: false,
            },
            7 => MouseEv::Button {
                button: 0,
                down: false,
                pressed: false,
                global: false,
            },
            8 => MouseEv::Button {
                button: 1,
                down: false,
                pressed: false,
                global: false,
            },
            9 => MouseEv::Button {
                button: 2,
                down: false,
                pressed: false,
                global: false,
            },
            10 => MouseEv::Enter,
            11 => MouseEv::Leave,
            12 => MouseEv::WheelUp,
            13 => MouseEv::WheelDown,
            50 => MouseEv::Button {
                button: 0,
                down: true,
                pressed: false,
                global: true,
            },
            51 => MouseEv::Button {
                button: 1,
                down: true,
                pressed: false,
                global: true,
            },
            52 => MouseEv::Button {
                button: 2,
                down: true,
                pressed: false,
                global: true,
            },
            54 => MouseEv::Button {
                button: 0,
                down: false,
                pressed: true,
                global: true,
            },
            55 => MouseEv::Button {
                button: 1,
                down: false,
                pressed: true,
                global: true,
            },
            56 => MouseEv::Button {
                button: 2,
                down: false,
                pressed: true,
                global: true,
            },
            57 => MouseEv::Button {
                button: 0,
                down: false,
                pressed: false,
                global: true,
            },
            58 => MouseEv::Button {
                button: 1,
                down: false,
                pressed: false,
                global: true,
            },
            59 => MouseEv::Button {
                button: 2,
                down: false,
                pressed: false,
                global: true,
            },
            _ => return Err("bad mouse event number".into()),
        }),
        7 => match numb {
            0 => EventKey::OutsideRoom,
            1 => EventKey::IntersectBoundary,
            2 => EventKey::GameStart,
            3 => EventKey::GameEnd,
            4 => EventKey::RoomStart,
            5 => EventKey::RoomEnd,
            6 => EventKey::NoMoreLives,
            7 => EventKey::NoMoreHealth,
            8 => EventKey::AnimationEnd,
            _ => return Err("bad other event number".into()),
        },
        8 => EventKey::Draw,
        _ => return Err("bad event type".into()),
    })
}

impl MouseEv {
    pub fn into_key(self) -> EventKey {
        EventKey::Mouse(self)
    }
}

fn hsv_to_rgb(h: u32, s: u32, v: u32) -> (u32, u32, u32) {
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

fn wrap_text(game: &mut Game, s: &str, w: f64) -> Vec<String> {
    let mut out = Vec::new();
    for para in s.split('#') {
        let mut line = String::new();
        for word in para.split(' ') {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };
            if game.backend.text_width(
                game.draw_font as u32,
                &candidate,
                game.current_font_size() as f32,
            ) as f64
                > w
                && !line.is_empty()
            {
                out.push(line.clone());
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        out.push(line);
    }
    out
}

struct Snapshot {
    x: f64,
    y: f64,
    hspeed: f64,
    vspeed: f64,
    vars: std::collections::HashMap<String, Value>,
}

fn instance_snapshot(i: &Instance) -> Snapshot {
    Snapshot {
        x: i.x,
        y: i.y,
        hspeed: i.hspeed,
        vspeed: i.vspeed,
        vars: i.vars.clone(),
    }
}

fn restore_snapshot(i: &mut Instance, s: &Snapshot) {
    i.x = s.x;
    i.y = s.y;
    i.hspeed = s.hspeed;
    i.vspeed = s.vspeed;
    i.vars = s.vars.clone();
}

impl Game {
    /// world → screen coordinates (relative to the current draw view)
    pub fn view_screen(&self, x: f64, y: f64) -> (f64, f64) {
        if self.view_enabled {
            let v = self.main_view();
            (x - v.x, y - v.y)
        } else {
            (x, y)
        }
    }

    /// Draw text with GM semantics: '#' is a newline, transforms supported.
    pub fn draw_text_impl(
        &mut self,
        x: f64,
        y: f64,
        text: &str,
        xscale: f64,
        yscale: f64,
        rot_deg: f64,
    ) {
        let lines: Vec<&str> = text.split('#').collect();
        let size = self.current_font_size();
        let lh = self.backend.text_height(self.draw_font as u32, size as f32) as f64 * yscale;
        let (color, alpha) = (self.draw_color, self.draw_alpha as f32);
        let (ha, va) = (self.draw_halign, self.draw_valign);
        let rad = rot_deg.to_radians();
        let (sin, cos) = rad.sin_cos();
        let (bx, by) = self.view_screen(x, y);
        // valign shifts the block start
        let vshift = match va {
            1 => -lh * lines.len() as f64 / 2.0,
            2 => -lh * lines.len() as f64,
            _ => 0.0,
        };
        for (i, line) in lines.iter().enumerate() {
            let w = self
                .backend
                .text_width(self.draw_font as u32, line, (size * xscale) as f32)
                as f64;
            let hshift = match ha {
                1 => -w / 2.0,
                2 => -w,
                _ => 0.0,
            };
            let ox = hshift;
            let oy = vshift + i as f64 * lh;
            let rx = bx + (ox * cos - oy * sin);
            let ry = by + (ox * sin + oy * cos);
            self.backend.draw_text(
                self.draw_font as u32,
                rx as f32,
                ry as f32,
                line,
                (size * xscale) as f32,
                color,
                alpha,
            );
        }
    }
}

impl Game {
    /// Font size in effect (custom font or built-in default).
    pub fn current_font_size(&self) -> f64 {
        if self.draw_font > 0 && self.draw_font - 1 < self.fonts.len() {
            self.fonts[self.draw_font - 1].size
        } else {
            crate::world::FONT_SIZE as f64
        }
    }

    /// draw_sprite_part: a sub-rectangle of a sprite frame.
    pub fn draw_sprite_part_impl(
        &mut self,
        sprite: i64,
        subimg: f64,
        left: f64,
        top: f64,
        w: f64,
        h: f64,
        x: f64,
        y: f64,
    ) {
        if sprite < 0 || (sprite as usize) >= self.sprites.len() {
            return;
        }
        let sidx = sprite as usize;
        let n = self.sprites[sidx].masks.len().max(1);
        let f = (((subimg.floor() as i64) % (n as i64) + n as i64) % n as i64) as usize;
        let Some(&tex) = self.sprite_tex[sidx].get(f) else {
            return;
        };
        let (sx, sy) = self.view_screen(x, y);
        self.backend.draw_sprite(
            tex,
            sx as f32,
            sy as f32,
            w as f32,
            h as f32,
            0.0,
            1.0,
            0xFFFFFF,
            false,
            false,
            Some((left as f32, top as f32, w as f32, h as f32)),
        );
    }
}

fn open_url_sys(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "windows")]
    let cmd = "cmd";
    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new(cmd)
            .args(["/C", "start", url])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "windows"))]
    std::process::Command::new(cmd)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

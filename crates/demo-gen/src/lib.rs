//! Builds a small self-contained demo game project: Arkanoid (room 1) plus a
//! "feature gallery" room (2) with paths, timelines, tiles and particles.

use gmformat::*;

fn put(buf: &mut [u8], w: u32, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if x < 0 || y < 0 || x >= w as i32 {
        return;
    }
    let idx = ((y as u32 * w + x as u32) * 4) as usize;
    if idx + 3 < buf.len() {
        buf[idx] = r;
        buf[idx + 1] = g;
        buf[idx + 2] = b;
        buf[idx + 3] = a;
    }
}

/// Ball sprite with a highlight dot rotating by `phase` (spin animation).
fn make_ball(phase: f64) -> (u32, u32, Vec<u8>) {
    let (w, h) = (16u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = x as f64 - 7.5;
            let dy = y as f64 - 7.5;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 7.0 {
                let (r, g, b) = if d > 5.5 {
                    (40, 40, 60)
                } else {
                    (240, 240, 250)
                };
                put(&mut buf, w, x, y, r, g, b, 255);
            }
        }
    }
    // spinning highlight
    let hx = 4.0 + 3.0 * phase.to_radians().cos();
    let hy = 4.0 + 3.0 * phase.to_radians().sin();
    for y in -1..=1i32 {
        for x in -1..=1i32 {
            if x * x + y * y <= 1 {
                put(
                    &mut buf,
                    w,
                    hx as i32 + x,
                    hy as i32 + y,
                    255,
                    255,
                    255,
                    255,
                );
            }
        }
    }
    (w, h, buf)
}

fn make_paddle() -> (u32, u32, Vec<u8>) {
    let (w, h) = (96u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let border = x < 2 || x >= w as i32 - 2 || y < 2 || y >= h as i32 - 2;
            let (r, g, b) = if border { (30, 30, 90) } else { (80, 110, 255) };
            put(&mut buf, w, x, y, r, g, b, 255);
        }
    }
    for x in (6..w as i32 - 6).step_by(12) {
        for y in 5..10 {
            put(&mut buf, w, x, y, 200, 220, 255, 255);
        }
    }
    (w, h, buf)
}

fn make_brick(r: u8, g: u8, b: u8) -> (u32, u32, Vec<u8>) {
    let (w, h) = (48u32, 24u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let border = x < 2 || x >= w as i32 - 2 || y < 2 || y >= h as i32 - 2;
            let hi = y == 2 || x == 2;
            let (rr, gg, bb) = if border {
                (r / 3, g / 3, b / 3)
            } else if hi {
                (
                    ((r as u16 + 90).min(255)) as u8,
                    ((g as u16 + 90).min(255)) as u8,
                    ((b as u16 + 90).min(255)) as u8,
                )
            } else {
                (r, g, b)
            };
            put(&mut buf, w, x, y, rr, gg, bb, 255);
        }
    }
    (w, h, buf)
}

fn make_wall() -> (u32, u32, Vec<u8>) {
    let (w, h) = (32u32, 32u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let border = x < 2 || x >= w as i32 - 2 || y < 2 || y >= h as i32 - 2;
            let (r, g, b) = if border {
                (70, 70, 80)
            } else {
                (140, 140, 150)
            };
            put(&mut buf, w, x, y, r, g, b, 255);
        }
    }
    (w, h, buf)
}

/// A 32x32 patterned tileset frame (checker with studs).
fn make_tileset() -> (u32, u32, Vec<u8>) {
    let (w, h) = (64u32, 32u32); // two 32x32 tiles side by side
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let in_tile2 = x >= 32;
            let lx = if in_tile2 { x - 32 } else { x };
            let checker = ((lx / 8) + (y / 8)) % 2 == 0;
            let (mut r, mut g, mut b): (u8, u8, u8) = if in_tile2 {
                if checker {
                    (96, 76, 140)
                } else {
                    (80, 62, 120)
                }
            } else if checker {
                (120, 96, 64)
            } else {
                (100, 78, 50)
            };
            // stud highlight
            if (8..12).contains(&(lx - 8 * (lx / 8))) && (8..12).contains(&(y - 8 * (y / 8))) {
                r = r.saturating_add(60);
                g = g.saturating_add(60);
                b = b.saturating_add(60);
            }
            put(&mut buf, w, x, y, r, g, b, 255);
        }
    }
    (w, h, buf)
}

fn make_orb() -> (u32, u32, Vec<u8>) {
    let (w, h) = (24u32, 24u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = x as f64 - 11.5;
            let dy = y as f64 - 11.5;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 11.0 {
                let (r, g, b) = if d < 5.0 {
                    (255, 244, 214)
                } else if d < 9.0 {
                    (250, 200, 60)
                } else {
                    (180, 120, 20)
                };
                put(&mut buf, w, x, y, r, g, b, 255);
            }
        }
    }
    (w, h, buf)
}

pub fn build_demo() -> Project {
    let mut p = Project {
        name: "RGM Demo — Arkanoid + Gallery".into(),
        ..Default::default()
    };

    // sprites: 0 ball, 1 paddle, 2 brick red, 3 brick green, 4 brick yellow, 5 wall, 6 orb
    let defs: Vec<(&str, (u32, u32, Vec<u8>))> = vec![
        ("spr_ball", make_ball(0.0)),
        ("spr_paddle", make_paddle()),
        ("spr_brick_red", make_brick(230, 70, 70)),
        ("spr_brick_green", make_brick(70, 200, 90)),
        ("spr_brick_yellow", make_brick(235, 210, 70)),
        ("spr_wall", make_wall()),
        ("spr_orb", make_orb()),
    ];
    let mut si = std::collections::HashMap::new();
    for (i, (name, (w, h, buf))) in defs.iter().enumerate() {
        si.insert(name.to_string(), i);
        let mut s = Sprite::new(name, *w, *h);
        s.frames[0] = encode_rgba(buf);
        // the ball gets a 3-frame spin animation
        if *name == "spr_ball" {
            s.frames.push(encode_rgba(&make_ball(120.0).2));
            s.frames.push(encode_rgba(&make_ball(240.0).2));
        }
        p.sprites.push(s);
    }

    // background 0: tileset
    {
        let (w, h, buf) = make_tileset();
        p.backgrounds.push(Background {
            name: "bg_tiles".into(),
            width: w,
            height: h,
            data: encode_rgba(&buf),
        });
    }

    // path 0: closed loop around the gallery room
    {
        let mut path = Path::new("path_ring");
        path.closed = true;
        for (x, y) in [
            (96.0, 120.0),
            (320.0, 80.0),
            (544.0, 120.0),
            (560.0, 300.0),
            (320.0, 360.0),
            (80.0, 300.0),
        ] {
            path.points.push(PathPoint { x, y, speed: 100.0 });
        }
        p.paths.push(path);
    }

    // timeline 0: gallery narration + a sparkle burst every loop
    {
        let mut tl = Timeline::new("tl_gallery");
        tl.moments.push(TimelineMoment {
            step: 20,
            code: "global.msg = \"Paths, timelines, tiles and particles live here\";".into(),
        });
        tl.moments.push(TimelineMoment {
            step: 90,
            code: "global.msg = \"Press F1 to return to Arkanoid\";".into(),
        });
        tl.moments.push(TimelineMoment {
            step: 120,
            code: "timeline_position = 0;".into(),
        });
        p.timelines.push(tl);
    }

    // objects: 0 wall, 1..3 bricks, 4 paddle, 5 ball, 6 controller, 7 follower, 8 gallery controller
    let mut wall = GmObject::new("obj_wall");
    wall.sprite_index = si["spr_wall"] as i64;
    wall.solid = true;
    p.objects.push(wall);

    for (name, spr, pts) in [
        ("obj_brick_red", "spr_brick_red", "10"),
        ("obj_brick_green", "spr_brick_green", "20"),
        ("obj_brick_yellow", "spr_brick_yellow", "30"),
    ] {
        let mut o = GmObject::new(name);
        o.sprite_index = si[spr] as i64;
        o.solid = true;
        o.events.push(EventDef::Create {
            code: format!("points = {};", pts),
        });
        o.events.push(EventDef::Destroy {
            code: "\
// particle explosion on destruction
if (variable_global_exists(\"ps\") && global.ps >= 0) {
    part_particles_create(global.ps, x, y, global.pt_spark, 24);
}
"
            .into(),
        });
        p.objects.push(o);
    }

    let mut paddle = GmObject::new("obj_paddle");
    paddle.sprite_index = si["spr_paddle"] as i64;
    paddle.events.push(EventDef::Create {
        code: "xs = x;".into(),
    });
    paddle.events.push(EventDef::Step {
        code: "\
if (keyboard_check(vk_left)) x -= 8;
if (keyboard_check(vk_right)) x += 8;
if (x < 64) x = 64;
if (x > room_width - 64) x = room_width - 64;
if (mouse_check_button(mb_left)) x = mouse_x;
"
        .into(),
    });
    p.objects.push(paddle);

    let mut ball = GmObject::new("obj_ball");
    ball.sprite_index = si["spr_ball"] as i64;
    ball.events.push(EventDef::Create {
        code: "stuck = 1; image_speed = 0.25;".into(),
    });
    ball.events.push(EventDef::Step {
        code: "\
if (stuck) {
    x = obj_paddle.x;
    y = obj_paddle.y - 24;
    exit;
}
if (abs(vspeed) < 0.7 && speed > 0) vspeed += 1.2;
if (y > room_height + 32) {
    lives -= 1;
    stuck = 1;
    speed = 0;
}
"
        .into(),
    });
    ball.events.push(EventDef::KeyPress {
        key: 32,
        code: "\
if (stuck && lives > 0) {
    stuck = 0;
    direction = 75 + random(30);
    speed = 7;
}
"
        .into(),
    });
    ball.events.push(EventDef::Collision {
        object: 0,
        code: "move_bounce_solid(true);".into(),
    });
    for bi in [1usize, 2, 3] {
        ball.events.push(EventDef::Collision {
            object: bi,
            code: "\
move_bounce_solid(true);
with (other) {
    score += points;
    instance_destroy();
}
"
            .into(),
        });
    }
    ball.events.push(EventDef::Collision {
        object: 4,
        code: "\
y = other.y - 20;
vspeed = -abs(vspeed);
hspeed = (x - other.x) * 0.12;
if (speed < 7) speed = 7;
"
        .into(),
    });
    p.objects.push(ball);

    // main controller: score/lives, particle system setup, room switching
    let mut controller = GmObject::new("obj_controller");
    controller.depth = -100;
    controller.events.push(EventDef::Other {
        kind: "game_start".into(),
        code: "\
score = 0;
lives = 3;
global.msg = \"\";
// particle setup: one system, two types
global.ps = part_system_create();
part_system_depth(global.ps, -50);
global.pt_spark = part_type_create(global.ps);
part_type_shape(global.pt_spark, pt_shape_spark);
part_type_size(global.pt_spark, 0.8, 1.6, -0.01, 0);
part_type_color1(global.pt_spark, c_yellow);
part_type_alpha3(global.pt_spark, 1, 0.8, 0);
part_type_speed(global.pt_spark, 2, 6, 0, 0);
part_type_direction(global.pt_spark, 0, 360, 0, 0);
part_type_life(global.pt_spark, 20, 40);
part_type_gravity(global.pt_spark, 0.15, 270);
global.pt_smoke = part_type_create(global.ps);
part_type_shape(global.pt_smoke, pt_shape_smoke);
part_type_size(global.pt_smoke, 2, 4, 0.05, 0);
part_type_color1(global.pt_smoke, c_gray);
part_type_alpha3(global.pt_smoke, 0.5, 0.4, 0);
part_type_life(global.pt_smoke, 40, 70);
part_type_speed(global.pt_smoke, 0.3, 1, 0, 0);
"
        .into(),
    });
    controller.events.push(EventDef::Step {
        code: "\
if (lives <= 0 && keyboard_check_pressed(vk_f2)) game_restart();
if (lives > 0
    && !instance_exists(obj_brick_red)
    && !instance_exists(obj_brick_green)
    && !instance_exists(obj_brick_yellow)
    && keyboard_check_pressed(vk_f2)) game_restart();
"
        .into(),
    });
    controller.events.push(EventDef::KeyPress {
        key: 112, // F1 — feature gallery
        code: "room_goto(1);".into(),
    });
    controller.events.push(EventDef::Draw {
        code: "\
draw_set_color(c_white);
draw_text(16, 12, \"SCORE: \" + string(score));
draw_text(16, 34, \"LIVES: \" + string(lives));
draw_set_color(c_yellow);
draw_text(room_width / 2 - 170, room_height - 26, \"<-/-> or mouse + SPACE | F1 = gallery\");
if (lives <= 0) {
    draw_set_color(c_red);
    draw_text(room_width / 2 - 110, room_height / 2 - 20, \"GAME OVER — press F2\");
}
if (!instance_exists(obj_brick_red) && !instance_exists(obj_brick_green) && !instance_exists(obj_brick_yellow) && lives > 0) {
    draw_set_color(c_lime);
    draw_text(room_width / 2 - 100, room_height / 2 - 20, \"YOU WIN! — press F2\");
}
"
        .into(),
    });
    p.objects.push(controller);

    // follower: rides path_ring forever
    let mut follower = GmObject::new("obj_follower");
    follower.sprite_index = si["spr_orb"] as i64;
    follower.events.push(EventDef::Create {
        code: "\
path_start(path_ring, 3, 1, true);
image_speed = 0;
"
        .into(),
    });
    p.objects.push(follower);

    // gallery controller: timeline + message + return key
    let mut gallery = GmObject::new("obj_gallery");
    gallery.depth = -100;
    gallery.events.push(EventDef::Create {
        code: "\
timeline_index = tl_gallery;
timeline_position = 0;
timeline_running = true;
timeline_loop = true;
"
        .into(),
    });
    gallery.events.push(EventDef::KeyPress {
        key: 112,
        code: "room_goto(0);".into(),
    });
    gallery.events.push(EventDef::Draw {
        code: "\
draw_set_color(c_white);
draw_text(16, 12, \"FEATURE GALLERY\");
if (variable_global_exists(\"msg\")) {
    draw_set_color(c_lime);
    draw_text(16, 36, global.msg);
}
draw_set_color(c_white);
draw_text(16, room_height - 24, \"path_ring follower + timeline tl_gallery + tiles | F1 = back\");
"
        .into(),
    });
    p.objects.push(gallery);

    // ---------------- room 1: arkanoid ----------------
    let mut room = Room::new("room_arkanoid", 640, 480);
    room.caption = "RGM Demo".into();
    room.back_color = 0x4A301E; // dark blue (GM 0xBBGGRR)
    room.show_color = true;
    for cx in 0..20 {
        room.instances.push(RoomInstance {
            x: 16.0 + cx as f64 * 32.0,
            y: 16.0,
            object: 0,
        });
    }
    for cy in 1..14 {
        room.instances.push(RoomInstance {
            x: 16.0,
            y: 16.0 + cy as f64 * 32.0,
            object: 0,
        });
        room.instances.push(RoomInstance {
            x: 624.0,
            y: 16.0 + cy as f64 * 32.0,
            object: 0,
        });
    }
    for row in 0..3usize {
        for col in 0..10usize {
            room.instances.push(RoomInstance {
                x: 112.0 + col as f64 * 48.0,
                y: 88.0 + row as f64 * 26.0,
                object: 1 + row,
            });
        }
    }
    room.instances.push(RoomInstance {
        x: 320.0,
        y: 432.0,
        object: 4,
    });
    room.instances.push(RoomInstance {
        x: 320.0,
        y: 400.0,
        object: 5,
    });
    room.instances.push(RoomInstance {
        x: 0.0,
        y: 0.0,
        object: 6,
    });

    // ---------------- room 2: feature gallery ----------------
    let mut gallery_room = Room::new("room_gallery", 640, 480);
    gallery_room.caption = "RGM Gallery".into();
    gallery_room.back_color = 0x2A1A30; // dark plum (GM 0xBBGGRR)
    gallery_room.creation_code = "global.msg = \"\";".into();
    // tiled floor made of tiles from bg_tiles
    for ty in (416..480).step_by(32) {
        for tx in (32..608).step_by(32) {
            let left = if ((tx / 32) + (ty / 32)) % 2 == 0 {
                0
            } else {
                32
            };
            gallery_room.tiles.push(RoomTile {
                bg: 0,
                left,
                top: 0,
                width: 32,
                height: 32,
                x: tx as f64,
                y: ty as f64,
                depth: 10,
                id: 0,
            });
        }
    }
    // ceiling tiles
    for tx in (32..608).step_by(32) {
        gallery_room.tiles.push(RoomTile {
            bg: 0,
            left: 32,
            top: 0,
            width: 32,
            height: 32,
            x: tx as f64,
            y: 32.0,
            depth: 10,
            id: 0,
        });
    }
    gallery_room.instances.push(RoomInstance {
        x: 96.0,
        y: 120.0,
        object: 7,
    });
    gallery_room.instances.push(RoomInstance {
        x: 0.0,
        y: 0.0,
        object: 8,
    });

    p.rooms = vec![room, gallery_room];
    p
}

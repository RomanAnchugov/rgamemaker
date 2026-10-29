use gmengine::{Game, NullBackend};

fn game() -> Game {
    let project = demo_gen::build_demo();
    Game::new(project, Box::new(NullBackend::default())).expect("demo must compile")
}

fn idle(g: &mut Game) {
    g.push_input(
        Default::default(),
        Default::default(),
        Default::default(),
        320.0,
        240.0,
        0,
        0,
        0,
        0,
    );
}

#[test]
fn execute_string_works() {
    let mut g = game();
    let v = g.run_code("execute_string(\"2 + 3 * 4\")", -4).unwrap();
    assert_eq!(v.as_f64().unwrap(), 14.0);
    let v = g
        .run_code("execute_string(\"global.zz = 41; global.zz + 1\")", -4)
        .unwrap();
    assert_eq!(v.as_f64().unwrap(), 42.0);
    assert!(g.take_errors().is_empty());
}

#[test]
fn paths_move_instances() {
    let mut g = game();
    g.run_code("room_goto(1)", -4).unwrap();
    g.step();
    // follower rides path_ring (object 7)
    let fid = g
        .run_code("instance_find(obj_follower, 0)", -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    let x0 = g.inst(fid).unwrap().x;
    let pos0 = g
        .run_code("with (obj_follower) path_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(pos0, 0.0);
    for _ in 0..30 {
        g.step();
    }
    let x1 = g.inst(fid).unwrap().x;
    let pos1 = g
        .run_code("with (obj_follower) path_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(pos1 > 0.0, "path position must advance, got {}", pos1);
    assert!(x1 != x0, "follower must move along the path");
    // path_end stops it
    g.run_code("with (obj_follower) path_end();", -4).unwrap();
    let pos_a = g
        .run_code("with (obj_follower) path_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    for _ in 0..10 {
        g.step();
    }
    let pos_b = g
        .run_code("with (obj_follower) path_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(pos_a, pos_b, "path_end must freeze the position");
    assert!(g.take_errors().is_empty());
}

#[test]
fn path_lookup_functions() {
    let mut g = game();
    assert_eq!(
        g.run_code("path_exists(path_ring)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        1.0
    );
    assert_eq!(
        g.run_code("path_get_number(path_ring)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        6.0
    );
    assert_eq!(
        g.run_code("path_get_point_x(path_ring, 0)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        96.0
    );
    assert!(
        g.run_code("path_get_length(path_ring)", -4)
            .unwrap()
            .as_f64()
            .unwrap()
            > 1000.0
    );
    assert_eq!(
        g.run_code("path_get_closed(path_ring)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        1.0
    );
}

#[test]
fn timelines_fire_moments() {
    let mut g = game();
    g.run_code("room_goto(1)", -4).unwrap();
    g.step(); // switch to the gallery room
              // obj_gallery has tl_gallery (moments at 20 and 90), looping via moment 120
    let mut fired = false;
    for i in 0..120 {
        g.step();
        let msg = g
            .run_code(
                "if (variable_global_exists(\"msg\")) global.msg else \"\"",
                -4,
            )
            .unwrap()
            .as_display_string();
        if msg.contains("Paths, timelines") {
            fired = true;
            break;
        }
        let _ = i;
    }
    assert!(fired, "timeline moment 20 must set global.msg");
    // timeline_position advances
    let pos = g
        .run_code("with (obj_gallery) timeline_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(pos >= 0.0);
    assert!(g.take_errors().is_empty());
}

#[test]
fn timeline_variables_control_it() {
    let mut g = game();
    g.run_code("room_goto(1)", -4).unwrap();
    g.step();
    g.run_code(
        "with (obj_gallery) { timeline_running = false; timeline_position = 0; }",
        -4,
    )
    .unwrap();
    for _ in 0..30 {
        g.step();
    }
    let pos = g
        .run_code("with (obj_gallery) timeline_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(pos, 0.0, "paused timeline must not advance");
    g.run_code(
        "with (obj_gallery) { timeline_running = true; timeline_speed = 5; }",
        -4,
    )
    .unwrap();
    for _ in 0..10 {
        g.step();
    }
    let pos = g
        .run_code("with (obj_gallery) timeline_position", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        pos >= 50.0,
        "timeline_speed=5 must advance ~50 steps, got {}",
        pos
    );
}

#[test]
fn tiles_are_loaded_and_editable() {
    let mut g = game();
    g.run_code("room_goto(1)", -4).unwrap();
    g.step();
    let count = g
        .run_code("tile_layer_count(10)", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        count > 30.0,
        "gallery floor tiles must exist, got {}",
        count
    );
    // add / query / move / delete
    let id = g
        .run_code("tile_add(bg_tiles, 0, 0, 32, 32, 100, 100, 5)", -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    assert!(id > 0);
    assert_eq!(
        g.run_code(&format!("tile_get_x(tile_find(5, {}))", id), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        100.0
    );
    g.run_code(&format!("tile_set_position({}, 200, 200)", id), -4)
        .unwrap();
    assert_eq!(
        g.run_code(&format!("tile_get_x({})", id), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        200.0
    );
    g.run_code(&format!("tile_delete({})", id), -4).unwrap();
    assert_eq!(
        g.run_code(&format!("tile_exists({})", id), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        0.0
    );
    g.draw(); // tiles render
    assert!(g.take_errors().is_empty());
}

#[test]
fn particles_spawn_update_and_die() {
    let mut g = game();
    g.step(); // game start sets up global.ps and types
    let ps = g.run_code("global.ps", -4).unwrap().as_f64().unwrap();
    assert!(ps >= 0.0);
    // burst sparks
    g.run_code(
        &format!(
            "part_particles_create({}, 320, 240, global.pt_spark, 50)",
            ps as i64
        ),
        -4,
    )
    .unwrap();
    let n = g
        .run_code(&format!("part_particles_count({})", ps as i64), -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(n >= 50.0, "burst must spawn particles, got {}", n);
    g.draw();
    // they die out (life 20..40 + smoke test later)
    for _ in 0..45 {
        g.step();
    }
    let n2 = g
        .run_code(&format!("part_particles_count({})", ps as i64), -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(n2 < 50.0, "particles must die, still alive: {}", n2);
    assert!(g.take_errors().is_empty());
}

#[test]
fn particle_emitters_stream() {
    let mut g = game();
    g.step();
    let ps = g.run_code("global.ps", -4).unwrap().as_f64().unwrap() as i64;
    let em = g
        .run_code(&format!("part_emitter_create({})", ps), -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    g.run_code(
        &format!(
            "part_emitter_region({}, 100, 100, 200, 200, ps_shape_rectangle, ps_distr_linear)",
            em
        ),
        -4,
    )
    .unwrap();
    g.run_code(
        &format!("part_emitter_stream({}, global.pt_smoke, 3)", em),
        -4,
    )
    .unwrap();
    for _ in 0..5 {
        g.step();
    }
    let n = g
        .run_code(&format!("part_particles_count({})", ps), -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(n >= 15.0, "stream emitter must produce ~3/step, got {}", n);
    g.run_code(&format!("part_emitter_stream({}, -1, 0)", em), -4)
        .unwrap();
    g.run_code(&format!("part_emitter_destroy({})", em), -4)
        .unwrap();
    assert!(g.take_errors().is_empty());
}

#[test]
fn brick_destroy_bursts_particles() {
    let mut g = game();
    g.step();
    g.run_code(
        "with (obj_ball) { stuck = 0; x = 112; y = 400; direction = 90; speed = 8; }",
        -4,
    )
    .unwrap();
    let mut hit = false;
    for _ in 0..120 {
        g.step();
        let n = g
            .run_code("part_particles_count(global.ps)", -4)
            .unwrap()
            .as_f64()
            .unwrap();
        if n > 0.0 {
            hit = true;
            break;
        }
    }
    assert!(hit, "brick destruction must spawn particles");
    g.draw();
    assert!(g.take_errors().is_empty());
}

#[test]
fn views_support_eight_slots() {
    let mut g = game();
    assert_eq!(
        g.run_code("view_enabled", -4).unwrap().as_f64().unwrap(),
        0.0
    );
    // enable + configure view 3 via arrays
    g.run_code("view_enabled = true; view_visible[3] = true; view_xview[3] = 100; view_wview[3] = 320; view_hview[3] = 240;", -4)
        .unwrap();
    assert_eq!(
        g.run_code("view_xview[3]", -4).unwrap().as_f64().unwrap(),
        100.0
    );
    assert_eq!(
        g.run_code("view_wview[3]", -4).unwrap().as_f64().unwrap(),
        320.0
    );
    assert_eq!(
        g.run_code("view_visible[1]", -4).unwrap().as_f64().unwrap(),
        0.0
    );
    assert_eq!(
        g.run_code("view_visible[3]", -4).unwrap().as_f64().unwrap(),
        1.0
    );
    assert_eq!(
        g.run_code("view_current", -4).unwrap().as_f64().unwrap(),
        0.0
    );
    g.draw();
    assert!(g.take_errors().is_empty());
}

#[test]
fn game_save_and_load_restores_state() {
    let mut g = game();
    g.step();
    g.run_code("with (obj_paddle) x = 500;", -4).unwrap();
    g.run_code("score = 77; lives = 2;", -4).unwrap();
    let save_path = std::env::temp_dir().join("rgm_test_save.sav");
    g.run_code(&format!("game_save(\"{}\")", save_path.display()), -4)
        .unwrap();
    assert!(save_path.exists());
    // change state
    g.run_code("with (obj_paddle) x = 100;", -4).unwrap();
    g.run_code("score = 0; lives = 0;", -4).unwrap();
    // restore
    g.run_code(&format!("game_load(\"{}\")", save_path.display()), -4)
        .unwrap();
    assert_eq!(g.run_code("score", -4).unwrap().as_f64().unwrap(), 77.0);
    assert_eq!(g.run_code("lives", -4).unwrap().as_f64().unwrap(), 2.0);
    let px = g
        .run_code("with (obj_paddle) x", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(px, 500.0);
    let _ = std::fs::remove_file(&save_path);
}

#[test]
fn misc_new_functions() {
    let mut g = game();
    g.step();
    // string_format
    let s = g
        .run_code("string_format(\"pi: %\", 8, 2, 3.14159)", -4)
        .unwrap()
        .as_display_string();
    assert_eq!(s, "pi: 3.14");
    let s2 = g
        .run_code("string_format(\"%\", 0, 2, 3.14159)", -4)
        .unwrap()
        .as_display_string();
    assert_eq!(s2, "3.14");
    // move_snap
    g.run_code("with (obj_paddle) { x = 123; move_snap(16, 16); }", -4)
        .unwrap();
    assert_eq!(
        g.run_code("with (obj_paddle) x", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        128.0
    );
    assert_eq!(
        g.run_code("with (obj_paddle) place_snapped(16, 16)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        1.0
    );
    // mp_linear_step toward a free point
    g.run_code("with (obj_ball) { stuck = 0; speed = 0; x = 320; y = 300; mp_linear_step(340, 300, 10, true); }", -4)
        .unwrap();
    assert_eq!(
        g.run_code("with (obj_ball) x", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        330.0
    );
    // dates
    let dt = g
        .run_code("date_create_datetime(2020, 5, 17, 13, 45, 21)", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(
        g.run_code(&format!("date_get_year({})", dt), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        2020.0
    );
    assert_eq!(
        g.run_code(&format!("date_get_month({})", dt), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        5.0
    );
    assert_eq!(
        g.run_code(&format!("date_get_hour({})", dt), -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        13.0
    );
    assert_eq!(
        g.run_code("date_leap_year(2020)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        1.0
    );
    assert_eq!(
        g.run_code("date_days_in_month(2021, 2)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        28.0
    );
    // draw extras run without errors
    g.run_code(
        "draw_set_color(c_red); draw_set_halign(fa_center); draw_set_valign(fa_middle); draw_healthbar(10, 10, 200, 24, health, c_black, c_red, c_lime, 0, true); draw_arrow(0, 0, 50, 50, 8); draw_triangle(0, 0, 20, 0, 10, 20, false); draw_roundrect(0, 0, 40, 40, true); draw_sprite_part(spr_ball, 0, 0, 0, 8, 8, 100, 100);",
        -4,
    )
    .unwrap();
    assert_eq!(
        g.run_code("draw_get_color()", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        255.0
    );
    // keyboard_string accumulation
    g.push_chars("hi");
    g.push_chars("!");
    assert_eq!(
        g.run_code("keyboard_string", -4)
            .unwrap()
            .as_display_string(),
        "hi!"
    );
    assert_eq!(
        g.run_code("keyboard_lastchar", -4)
            .unwrap()
            .as_display_string(),
        "!"
    );
    assert!(g.take_errors().is_empty());
}

#[test]
fn room_switch_keeps_tiles_per_room() {
    let mut g = game();
    g.run_code("room_goto(1)", -4).unwrap();
    g.step();
    let gallery_tiles = g
        .run_code("tile_layer_count(10)", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(gallery_tiles > 30.0);
    g.run_code("room_goto(0)", -4).unwrap();
    g.step();
    let arkanoid_tiles = g
        .run_code("tile_layer_count(10)", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(arkanoid_tiles, 0.0, "room 1 has no tiles");
    g.draw();
    assert!(g.take_errors().is_empty());
}

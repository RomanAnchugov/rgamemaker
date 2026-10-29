use gmengine::{Game, NullBackend};
use gmformat::decode_rgba;
use std::collections::HashSet;

fn game() -> Game {
    let project = demo_gen::build_demo();
    Game::new(project, Box::new(NullBackend::default())).expect("demo must compile")
}

fn press(game: &mut Game, vk: i64) {
    let mut down = HashSet::new();
    let mut pressed = HashSet::new();
    down.insert(vk);
    pressed.insert(vk);
    game.push_input(down, pressed, HashSet::new(), 320.0, 240.0, 0, 0, 0, 0);
}

fn idle(game: &mut Game) {
    game.push_input(
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        320.0,
        240.0,
        0,
        0,
        0,
        0,
    );
}

#[test]
fn demo_boots_and_instances_exist() {
    let mut g = game();
    for _ in 0..5 {
        g.step();
    }
    assert_eq!(
        g.run_code("instance_number(obj_wall)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        46.0
    );
    assert_eq!(
        g.run_code("instance_number(obj_brick_red)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        10.0
    );
    assert_eq!(
        g.run_code("instance_number(all)", -4)
            .unwrap()
            .as_f64()
            .unwrap(),
        79.0
    );
    assert!(g.take_errors().is_empty(), "no runtime errors expected");
}

#[test]
fn ball_has_three_spin_frames() {
    let project = demo_gen::build_demo();
    let ball = project
        .sprites
        .iter()
        .find(|s| s.name == "spr_ball")
        .unwrap();
    assert_eq!(
        ball.frames.len(),
        3,
        "ball must have a 3-frame spin animation"
    );
    assert!(ball.frames.iter().all(|f| !decode_rgba(f).is_empty()));
}

#[test]
fn game_start_event_sets_score_and_lives() {
    let mut g = game();
    assert_eq!(g.run_code("score", -4).unwrap().as_f64().unwrap(), 0.0);
    assert_eq!(g.run_code("lives", -4).unwrap().as_f64().unwrap(), 3.0);
}

#[test]
fn ball_launches_with_space() {
    let mut g = game();
    g.step();
    let ball = g
        .run_code("instance_find(obj_ball, 0)", -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    assert_eq!(g.inst(ball).unwrap().speed(), 0.0);
    press(&mut g, 32); // space
    g.step();
    idle(&mut g);
    let sp = g.inst(ball).unwrap().speed();
    assert!((sp - 7.0).abs() < 0.001, "ball speed must be 7, got {}", sp);
    // ball moves upward now
    let y0 = g.inst(ball).unwrap().y;
    for _ in 0..10 {
        g.step();
    }
    assert!(g.inst(ball).unwrap().y < y0, "ball should travel up");
}

#[test]
fn paddle_moves_with_keyboard() {
    let mut g = game();
    g.step();
    let paddle = g
        .run_code("instance_find(obj_paddle, 0)", -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    let x0 = g.inst(paddle).unwrap().x;
    press(&mut g, 39); // right arrow
    for _ in 0..10 {
        g.step();
    }
    idle(&mut g);
    assert!(g.inst(paddle).unwrap().x > x0 + 70.0);
    assert!(g.take_errors().is_empty());
}

#[test]
fn ball_bounces_off_right_wall() {
    let mut g = game();
    g.step();
    // place the ball near the right wall, flying right
    g.run_code(
        "with (obj_ball) { stuck = 0; x = 580; y = 240; direction = 0; speed = 5; }",
        -4,
    )
    .unwrap();
    for _ in 0..30 {
        g.step();
    }
    let hspeed = g
        .run_code("with (obj_ball) hspeed", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(hspeed < 0.0, "ball must have bounced, hspeed = {}", hspeed);
    let x = g
        .run_code("with (obj_ball) x", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(x < 600.0, "ball must not pass through the wall, x = {}", x);
    assert!(g.take_errors().is_empty());
}

#[test]
fn brick_collision_scores_and_destroys() {
    let mut g = game();
    g.step();
    // shoot the ball straight up into the brick grid
    g.run_code(
        "with (obj_ball) { stuck = 0; x = 112; y = 400; direction = 90; speed = 8; }",
        -4,
    )
    .unwrap();
    let mut hit = false;
    for _ in 0..120 {
        g.step();
        if g.run_code("score", -4).unwrap().as_f64().unwrap() > 0.0 {
            hit = true;
            break;
        }
    }
    assert!(hit, "ball should destroy a brick and add score");
    let bricks = g
        .run_code("instance_number(obj_brick_yellow)", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        bricks < 10.0,
        "one yellow brick (bottom row) should be destroyed, {} left",
        bricks
    );
    assert!(g.take_errors().is_empty(), "errors: {:?}", g.take_errors());
}

#[test]
fn losing_a_ball_decrements_lives() {
    let mut g = game();
    g.step();
    g.run_code(
        "with (obj_ball) { stuck = 0; x = 320; y = 470; direction = 270; speed = 6; }",
        -4,
    )
    .unwrap();
    for _ in 0..60 {
        g.step();
    }
    let lives = g.run_code("lives", -4).unwrap().as_f64().unwrap();
    assert_eq!(lives, 2.0, "falling out of the room must cost a life");
    // ball resets to the paddle
    let stuck = g
        .run_code("with (obj_ball) stuck", -4)
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(stuck, 1.0);
    assert!(g.take_errors().is_empty());
}

#[test]
fn draw_pass_produces_no_errors() {
    let mut g = game();
    press(&mut g, 32);
    g.step();
    idle(&mut g);
    g.draw();
    assert!(g.take_errors().is_empty());
}

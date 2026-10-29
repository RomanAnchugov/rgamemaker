use gmengine::{Game, NullBackend};
use gmformat::*;

fn game_with_paddle_dnd() -> Game {
    let mut p = Project::default();
    p.sprites.push(Sprite::new("spr_paddle", 32, 8));
    let mut paddle = GmObject::new("obj_paddle");
    paddle.sprite_index = 0;
    // Step event as D&D: if keyboard_check(vk_right) → hspeed = 5 else hspeed = 0
    let mut q = DndAction::new(4, 405);
    q.params = vec![DndParam::Str("keyboard_check(vk_right)".into())];
    let mut right = DndAction::new(1, 124);
    right.params = vec![DndParam::Real(5.0)];
    let mut else_m = DndAction::new(4, 422);
    else_m.kind = 3;
    let mut stop = DndAction::new(1, 124);
    stop.params = vec![DndParam::Real(0.0)];
    paddle.events.push(EventDef::Actions {
        target: Box::new(EventTarget::Step),
        actions: vec![q, right, else_m, stop],
    });
    p.objects.push(paddle);
    let mut room = Room::new("room0", 640, 480);
    room.instances.push(RoomInstance {
        x: 100.0,
        y: 100.0,
        object: 0,
    });
    p.rooms = vec![room];
    Game::new(p, Box::new(NullBackend::default())).expect("must compile")
}

#[test]
fn dnd_step_event_runs() {
    let mut g = game_with_paddle_dnd();
    let pid = g
        .run_code("instance_find(obj_paddle, 0)", -4)
        .unwrap()
        .as_f64()
        .unwrap() as i64;
    g.step();
    assert_eq!(
        g.inst(pid).unwrap().hspeed,
        0.0,
        "no key pressed → else branch → hspeed = 0"
    );
    // hold right arrow
    let mut down = std::collections::HashSet::new();
    down.insert(39i64);
    g.push_input(
        down,
        Default::default(),
        Default::default(),
        0.0,
        0.0,
        0,
        0,
        0,
        0,
    );
    g.step();
    assert_eq!(
        g.inst(pid).unwrap().hspeed,
        5.0,
        "right held → if branch → hspeed = 5"
    );
    assert!(g.take_errors().is_empty(), "{:?}", g.take_errors());
}

#[test]
fn dnd_project_json_roundtrip() {
    let mut p = Project::default();
    let mut o = GmObject::new("obj_test");
    let mut a = DndAction::new(4, 401);
    a.params = vec![DndParam::Str("x += 1;".into())];
    a.relative = true;
    o.events.push(EventDef::Actions {
        target: Box::new(EventTarget::Create),
        actions: vec![a],
    });
    p.objects.push(o);
    let json = serde_json::to_string(&p).unwrap();
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back.objects[0].name, p.objects[0].name);
    let (t1, a1, r1) = match (&back.objects[0].events[0], &p.objects[0].events[0]) {
        (
            EventDef::Actions {
                target: t1,
                actions: a1,
            },
            EventDef::Actions {
                target: t2,
                actions: a2,
            },
        ) => (t1 == t2, a1.len() == a2.len(), a1[0].relative),
        _ => (false, false, false),
    };
    assert!(t1 && a1 && r1);
    assert!(matches!(
        back.objects[0].events[0],
        EventDef::Actions { .. }
    ));
}

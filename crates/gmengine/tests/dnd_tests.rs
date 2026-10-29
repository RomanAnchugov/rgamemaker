//! D&D compiler tests: action lists → GML → executable programs.

use gmengine::dnd::{compile_event, find_def, ACTIONS};
use gmformat::{DndAction, DndParam};

fn act(lib: u32, id: u32) -> DndAction {
    DndAction::new(lib, id)
}

fn with_params(mut a: DndAction, params: Vec<DndParam>) -> DndAction {
    a.params = params;
    a
}

fn compile(actions: &[DndAction]) -> String {
    let objects = vec![gmformat::GmObject::new("obj_dummy")];
    compile_event(actions, &objects).expect("compile must succeed")
}

fn compile_valid(actions: &[DndAction]) -> String {
    let gml = compile(actions);
    gml::compile(&gml)
        .unwrap_or_else(|e| panic!("generated GML must compile: {}\n---\n{}", e, gml));
    gml
}

#[test]
fn every_def_emits_valid_gml() {
    // every non-question action with zero/default params must still compile
    // to valid GML or fail gracefully (markers need context, questions need bodies)
    for def in ACTIONS {
        if def.question || (def.lib == 4 && (420..=426).contains(&def.id)) {
            continue; // markers and questions need list context
        }
        let a = act(def.lib, def.id);
        let objects = vec![];
        match compile_event(std::slice::from_ref(&a), &objects) {
            Ok(gml) => {
                if let Err(e) = gml::compile(&gml) {
                    panic!("{} emitted invalid GML: {}\n{}", def.name, e, gml);
                }
            }
            Err(e) => panic!("{} failed to emit: {}", def.name, e),
        }
    }
}

#[test]
fn simple_motion() {
    let gml = compile_valid(&[with_params(
        act(1, 122),
        vec![DndParam::Real(90.0), DndParam::Real(5.0)],
    )]);
    assert!(gml.contains("motion_set(90, 5);"), "{}", gml);
}

#[test]
fn relative_motion_adds() {
    let mut a = with_params(act(1, 122), vec![DndParam::Real(0.0), DndParam::Real(2.0)]);
    a.relative = true;
    let gml = compile_valid(&[a]);
    assert!(gml.contains("motion_add(0, 2);"), "{}", gml);
}

#[test]
fn move_fixed_grid_picks_directions() {
    // bits 0,2,6 = up-left(135), up-right(45), down-left(225)
    let a = with_params(
        act(1, 121),
        vec![DndParam::Real(1.0 + 4.0 + 64.0), DndParam::Real(4.0)],
    );
    let gml = compile_valid(&[a]);
    assert!(
        gml.contains("motion_set(choose(135, 45, 225), 4);"),
        "{}",
        gml
    );
}

#[test]
fn move_fixed_center_is_stop() {
    let a = with_params(act(1, 121), vec![DndParam::Real(16.0), DndParam::Real(8.0)]);
    let gml = compile_valid(&[a]);
    assert!(gml.contains("speed = 0;"), "{}", gml);
}

#[test]
fn move_fixed_right_default() {
    let a = with_params(act(1, 121), vec![DndParam::Real(32.0), DndParam::Real(5.0)]);
    let gml = compile_valid(&[a]);
    assert!(gml.contains("motion_set(0, 5);"), "{}", gml);
}

#[test]
fn set_variable_relative() {
    let mut a = with_params(
        act(4, 403),
        vec![DndParam::Str("score".into()), DndParam::Real(10.0)],
    );
    a.relative = true;
    let gml = compile_valid(&[a]);
    assert!(gml.contains("score += 10;"), "{}", gml);
}

#[test]
fn question_with_block_and_else() {
    let mut q = with_params(
        act(4, 404),
        vec![
            DndParam::Str("lives".into()),
            DndParam::Str("<=".into()),
            DndParam::Real(0.0),
        ],
    );
    q.not = true;
    let a1 = with_params(act(1, 124), vec![DndParam::Real(0.0)]);
    let a2 = with_params(act(1, 125), vec![DndParam::Real(0.0)]);
    let mut begin = act(4, 420);
    begin.kind = 1;
    let mut end = act(4, 421);
    end.kind = 2;
    let mut else_m = act(4, 422);
    else_m.kind = 3;
    let gml = compile_valid(&[q, begin, a1, end, else_m, a2]);
    assert!(gml.contains("!(lives <= 0)"), "{}", gml);
    assert!(gml.contains("else"), "{}", gml);
    assert!(gml.contains("hspeed = 0;"), "{}", gml);
}

#[test]
fn question_single_statement() {
    let q = with_params(act(4, 405), vec![DndParam::Str("speed > 3".into())]);
    let a = with_params(act(2, 135), vec![]);
    let gml = compile_valid(&[q, a]);
    assert!(gml.contains("instance_destroy();"), "{}", gml);
    assert!(gml.trim_start().starts_with("if"), "{}", gml);
}

#[test]
fn repeat_wraps_block() {
    let mut r = with_params(act(4, 424), vec![DndParam::Real(5.0)]);
    r.kind = 5;
    let a = with_params(act(5, 501), vec![DndParam::Real(1.0)]);
    let gml = compile_valid(&[r, a]);
    assert!(gml.contains("repeat (5)"), "{}", gml);
    assert!(gml.contains("score = 1;"), "{}", gml);
}

#[test]
fn while_and_until() {
    let mut w = with_params(act(4, 425), vec![DndParam::Str("x < 100".into())]);
    w.kind = 6;
    let a1 = with_params(act(1, 124), vec![DndParam::Real(1.0)]);
    let mut u = with_params(act(4, 426), vec![DndParam::Str("y > 0".into())]);
    u.kind = 7;
    let a2 = with_params(act(1, 125), vec![DndParam::Real(-1.0)]);
    let gml = compile_valid(&[w, a1, u, a2]);
    assert!(gml.contains("while (x < 100)"), "{}", gml);
    assert!(gml.contains("until (!(y > 0))"), "{}", gml);
}

#[test]
fn code_action_inlines_source() {
    let a = with_params(act(4, 401), vec![DndParam::Str("x += 1;\ny += 1;".into())]);
    let gml = compile_valid(&[a]);
    assert!(gml.contains("x += 1;"), "{}", gml);
}

#[test]
fn applies_to_wraps_with() {
    let mut a = with_params(act(2, 135), vec![]);
    a.applies_to = -2;
    let gml = compile_valid(&[a]);
    assert!(gml.contains("with (other)"), "{}", gml);
    assert!(gml.contains("instance_destroy();"), "{}", gml);
}

#[test]
fn applies_to_object_uses_name() {
    let mut a = with_params(act(1, 129), vec![DndParam::Real(0.5)]);
    a.applies_to = 0;
    let objects = vec![gmformat::GmObject::new("obj_dummy")];
    let gml = compile_event(&[a], &objects).unwrap();
    assert!(gml.contains("with (obj_dummy)"), "{}", gml);
}

#[test]
fn begin_without_question_is_plain_block() {
    let mut begin = act(4, 420);
    begin.kind = 1;
    let a = with_params(act(5, 501), vec![DndParam::Real(7.0)]);
    let mut end = act(4, 421);
    end.kind = 2;
    let gml = compile_valid(&[begin, a, end]);
    assert!(gml.contains("score = 7;"), "{}", gml);
}

#[test]
fn unmatched_end_is_error() {
    let mut end = act(4, 421);
    end.kind = 2;
    let objects = vec![];
    assert!(compile_event(&[end], &objects).is_err());
}

#[test]
fn question_without_body_is_error() {
    let q = with_params(act(4, 405), vec![DndParam::Str("1".into())]);
    let objects = vec![];
    assert!(compile_event(&[q], &objects).is_err());
}

#[test]
fn def_table_sanity() {
    assert!(
        ACTIONS.len() >= 75,
        "expected a substantial action table, got {}",
        ACTIONS.len()
    );
    // unique lib/id pairs
    let mut seen = std::collections::HashSet::new();
    for a in ACTIONS {
        assert!(
            seen.insert((a.lib, a.id)),
            "duplicate def lib {} id {}",
            a.lib,
            a.id
        );
    }
    assert!(find_def(1, 121).is_some());
    assert!(find_def(4, 401).unwrap().name.contains("code"));
    // every marker has a def
    for id in [420u32, 421, 422, 423, 424, 425, 426] {
        assert!(find_def(4, id).is_some(), "marker def {} missing", id);
    }
}

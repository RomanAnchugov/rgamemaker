//! GM8-style drag-and-drop editor: action list column + library palette with
//! tabs, drag & drop between palette and list, and parameter dialogs.

use crate::codeedit::{CompletionState, NameDb};
use crate::icons::Icon;
use crate::theme;
use eframe::egui;
use gmengine::dnd::{self, ActionDef, ParamKind, ACTIONS, LIBS};
use gmformat::{DndAction, DndParam, EventDef, Project};

// ---------------------------------------------------------------- payloads

#[derive(Clone)]
pub enum DndPayload {
    /// A new action dragged from the palette.
    New { lib: u32, id: u32 },
    /// An existing action being reordered.
    Move { index: usize },
}

/// Which dialog is open: edit action at index (or None = closed).
pub struct DndEditorState {
    pub lib_tab: u32,
    pub edit: Option<usize>,
    pub edit_buf: DndAction,
    pub clipboard: Vec<DndAction>,
    /// completion state for the embedded GML editor ("execute code" action)
    pub code_cs: CompletionState,
}

impl Default for DndEditorState {
    fn default() -> Self {
        DndEditorState {
            lib_tab: 1,
            edit: None,
            edit_buf: DndAction::new(1, 121),
            clipboard: vec![],
            code_cs: CompletionState::default(),
        }
    }
}

#[allow(dead_code)]
fn lib_icon(lib: u32) -> Icon {
    match lib {
        1 => Icon::Run,      // move
        2 => Icon::Plus,     // main1 (create/destroy)
        3 => Icon::Path,     // main2 (paths/timing)
        4 => Icon::Script,   // control
        5 => Icon::Timeline, // score
        6 => Icon::Folder,   // extra
        7 => Icon::Font,     // draw
        _ => Icon::Fill,     // particles
    }
}

fn lib_color(lib: u32) -> egui::Color32 {
    match lib {
        1 => egui::Color32::from_rgb(0x2A, 0x7A, 0xE0),
        2 => egui::Color32::from_rgb(0x2E, 0x8A, 0x3A),
        3 => egui::Color32::from_rgb(0xB0, 0x70, 0x18),
        4 => egui::Color32::from_rgb(0x7A, 0x3A, 0xB0),
        5 => egui::Color32::from_rgb(0xC0, 0x50, 0x20),
        6 => egui::Color32::from_gray(90),
        7 => egui::Color32::from_rgb(0x18, 0x7A, 0x80),
        _ => egui::Color32::from_rgb(0xA0, 0x30, 0x50),
    }
}

/// Short caption of an action with its current params (like GM8's list).
pub fn action_caption(def: &ActionDef, a: &DndAction) -> String {
    match (a.lib, a.id) {
        (1, 121) => {
            // direction grid: ASCII-safe labels in the GM8 3x3 layout
            let mask = pnum_of(a, 0, 32.0) as i32;
            const GRID: [(i32, &str); 9] = [
                (1, "up-left"),
                (2, "up"),
                (4, "up-right"),
                (8, "left"),
                (16, "STOP"),
                (32, "right"),
                (64, "down-left"),
                (128, "down"),
                (256, "down-right"),
            ];
            if mask == 16 {
                "start moving: STOP".to_string()
            } else {
                match GRID.iter().find(|(bit, c)| *c != "STOP" && mask & bit != 0) {
                    Some((_, name)) => {
                        format!("start moving {} (speed {})", name, pexpr_of(a, 1, "0"))
                    }
                    None => {
                        format!("start moving: no direction (speed {})", pexpr_of(a, 1, "0"))
                    }
                }
            }
        }
        (1, 122) => format!(
            "set direction+speed ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (1, 123) => format!(
            "move towards ({}, {}) speed {}",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0"),
            pexpr_of(a, 2, "0")
        ),
        (1, 124) => format!("hspeed = {}", pexpr_of(a, 0, "0")),
        (1, 125) => format!("vspeed = {}", pexpr_of(a, 0, "0")),
        (1, 126) => format!(
            "gravity {} in direction {}",
            pexpr_of(a, 1, "0"),
            pexpr_of(a, 0, "270")
        ),
        (1, 127) => "reverse horizontal".into(),
        (1, 128) => "reverse vertical".into(),
        (1, 129) => format!("friction = {}", pexpr_of(a, 0, "0")),
        (2, 131) => format!(
            "create {} at ({}, {})",
            pexpr_of(a, 2, "?"),
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (2, 132) => format!(
            "create {} moving, speed {}",
            pexpr_of(a, 2, "?"),
            pexpr_of(a, 3, "1")
        ),
        (2, 133) => format!("create random {}", pexpr_of(a, 4, "?")),
        (2, 134) => format!("change into {}", pexpr_of(a, 0, "?")),
        (2, 135) => "destroy the instance".into(),
        (2, 136) => format!(
            "destroy at ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (2, 137) => format!("change sprite to {}", pexpr_of(a, 0, "?")),
        (2, 138) => format!(
            "transform sprite ({}, {}, {}°)",
            pexpr_of(a, 0, "1"),
            pexpr_of(a, 1, "1"),
            pexpr_of(a, 2, "0")
        ),
        (2, 139) => format!("set collision mask to {}", pexpr_of(a, 0, "?")),
        (2, 141) => format!("play sound {}", pexpr_of(a, 0, "?")),
        (2, 142) => format!("stop sound {}", pexpr_of(a, 0, "?")),
        (2, 143) => format!("if sound {} is playing", pexpr_of(a, 0, "?")),
        (2, 144) => "go to next room".into(),
        (2, 145) => "go to previous room".into(),
        (2, 146) => format!("go to room {}", pexpr_of(a, 0, "?")),
        (2, 147) => "restart the room".into(),
        (2, 148) => format!("sleep {} ms", pexpr_of(a, 0, "0")),
        (2, 149) => format!("set cursor to {}", pexpr_of(a, 0, "?")),
        (3, 151) => format!(
            "set alarm[{}] to {}",
            pmenu_of(a, 1, "0"),
            pexpr_of(a, 0, "0")
        ),
        (3, 152) => format!(
            "follow path {} (speed {})",
            pexpr_of(a, 0, "?"),
            pexpr_of(a, 1, "1")
        ),
        (3, 153) => format!("position in path: point {}", pexpr_of(a, 0, "0")),
        (3, 154) => format!("path position = {}", pexpr_of(a, 0, "0")),
        (3, 155) => format!("path speed = {}", pexpr_of(a, 0, "0")),
        (3, 156) => "bounce against objects".into(),
        (4, 401) => {
            let code = pstr_of(a, 0, "");
            let first = code.lines().next().unwrap_or("");
            let more = if code.lines().count() > 1 { " …" } else { "" };
            format!("execute code: {}{}", first, more)
        }
        (4, 402) => format!("// {}", pstr_of(a, 0, "")),
        (4, 403) => format!("set {} = {}", pstr_of(a, 0, "x"), pexpr_of(a, 1, "0")),
        (4, 404) => format!(
            "if {} {} {}",
            pstr_of(a, 0, "x"),
            pmenu_of(a, 1, "=="),
            pexpr_of(a, 2, "0")
        ),
        (4, 405) => format!("if {}", pexpr_of(a, 0, "1")),
        (4, 406) => format!(
            "if place free ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (4, 407) => format!(
            "if collision at ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (4, 408) => format!(
            "if {} at ({}, {})",
            pexpr_of(a, 2, "?"),
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (4, 409) => format!(
            "if number of {} {} {}",
            pexpr_of(a, 0, "?"),
            pmenu_of(a, 1, "=="),
            pexpr_of(a, 2, "0")
        ),
        (4, 410) => format!("if chance 1 out of {}", pexpr_of(a, 0, "2")),
        (4, 411) => format!("ask: {}", pstr_of(a, 0, "?")),
        (4, 412) => format!("if expression {}", pexpr_of(a, 0, "1")),
        (4, 413) => "if mouse button pressed".into(),
        (4, 414) => format!(
            "if aligned to grid ({}, {})",
            pexpr_of(a, 0, "16"),
            pexpr_of(a, 1, "16")
        ),
        (4, 420) => "begin block".into(),
        (4, 421) => "end block".into(),
        (4, 422) => "else".into(),
        (4, 423) => "exit this event".into(),
        (4, 424) => format!("repeat next ({}×)", pexpr_of(a, 0, "1")),
        (4, 425) => format!("while next: {}", pexpr_of(a, 0, "1")),
        (4, 426) => format!("until next: {}", pexpr_of(a, 0, "1")),
        (5, 501) => format!("score = {}", pexpr_of(a, 0, "0")),
        (5, 502) => format!("if score {} {}", pmenu_of(a, 0, "=="), pexpr_of(a, 1, "0")),
        (5, 503) => format!(
            "draw score at ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (5, 504) => format!("lives = {}", pexpr_of(a, 0, "0")),
        (5, 505) => format!("if lives {} {}", pmenu_of(a, 0, "=="), pexpr_of(a, 1, "0")),
        (5, 506) => format!(
            "draw lives at ({}, {})",
            pexpr_of(a, 0, "0"),
            pexpr_of(a, 1, "0")
        ),
        (5, 507) => format!("health = {}", pexpr_of(a, 0, "100")),
        (5, 508) => format!("if health < {}", pexpr_of(a, 0, "0")),
        (5, 509) => "draw the health bar".into(),
        (5, 510) => "clear highscore table".into(),
        (5, 511) => "if score among the best".into(),
        (6, 601) => format!("open {}", pstr_of(a, 0, "url")),
        (6, 602) => "show game information".into(),
        (6, 603) => "show a video".into(),
        (6, 611) => "play CD".into(),
        (6, 612) => "pause CD".into(),
        (6, 613) => "resume CD".into(),
        (6, 614) => "stop CD".into(),
        (6, 621) => "replace sprite from file".into(),
        (6, 622) => "replace background from file".into(),
        (7, 701) => format!(
            "draw sprite {} at ({}, {})",
            pexpr_of(a, 0, "?"),
            pexpr_of(a, 2, "0"),
            pexpr_of(a, 3, "0")
        ),
        (7, 702) => format!(
            "draw background {} at ({}, {})",
            pexpr_of(a, 0, "?"),
            pexpr_of(a, 1, "0"),
            pexpr_of(a, 2, "0")
        ),
        (7, 703) => format!("draw text \"{}\"", pstr_of(a, 2, "")),
        (7, 704) => format!("draw transformed text \"{}\"", pstr_of(a, 2, "")),
        (7, 705) => "draw a rectangle".into(),
        (7, 706) => "draw a horizontal gradient".into(),
        (7, 707) => "draw a vertical gradient".into(),
        (7, 708) => "set the drawing color".into(),
        (7, 709) => "set the drawing font".into(),
        (7, 710) => format!("draw variable {}", pstr_of(a, 3, "x")),
        (8, 801) => "create particle system".into(),
        (8, 802) => "destroy particle system".into(),
        (8, 811) => format!("create particle type '{}'", pstr_of(a, 0, "ptype")),
        (8, 812) => format!(
            "create {} particles of '{}'",
            pexpr_of(a, 3, "10"),
            pstr_of(a, 0, "ptype")
        ),
        _ => def.name.to_string(),
    }
}

fn pnum_of(a: &DndAction, i: usize, d: f64) -> f64 {
    match a.params.get(i) {
        Some(DndParam::Real(x)) => *x,
        Some(DndParam::Both { real, .. }) => *real,
        Some(DndParam::Str(s)) => s.parse().unwrap_or(d),
        None => d,
    }
}

fn pexpr_of(a: &DndAction, i: usize, d: &str) -> String {
    match a.params.get(i) {
        Some(DndParam::Real(x)) => {
            if x.fract().abs() < 1e-9 {
                (x.round() as i64).to_string()
            } else {
                x.to_string()
            }
        }
        Some(DndParam::Str(s)) if !s.is_empty() => s.clone(),
        Some(DndParam::Both { real, str: s }) => {
            if !s.is_empty() {
                s.clone()
            } else {
                format!("{}", real)
            }
        }
        _ => d.to_string(),
    }
}

fn pstr_of(a: &DndAction, i: usize, d: &str) -> String {
    match a.params.get(i) {
        Some(DndParam::Str(s)) => s.clone(),
        Some(DndParam::Both { str: s, .. }) if !s.is_empty() => s.clone(),
        Some(DndParam::Both { real, .. }) => format!("{}", real),
        _ => d.to_string(),
    }
}

fn pmenu_of(a: &DndAction, i: usize, d: &str) -> String {
    pexpr_of(a, i, d)
}

// ---------------------------------------------------------------- editor

/// The middle column: the action list of the selected Actions-event.
pub fn action_list(
    ui: &mut egui::Ui,
    project: &mut Project,
    obj_idx: usize,
    ev_idx: usize,
    st: &mut DndEditorState,
    selected_action: &mut Option<usize>,
) {
    let objects_snapshot: Vec<String> = project.objects.iter().map(|o| o.name.clone()).collect();
    let actions = match &mut project.objects[obj_idx].events[ev_idx] {
        EventDef::Actions { actions, .. } => actions,
        _ => return,
    };

    ui.strong("Actions");
    ui.label(
        egui::RichText::new("drag from the palette, double-click to edit")
            .small()
            .weak(),
    );
    theme::sunken_frame().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt(format!("acts_{}_{}", obj_idx, ev_idx))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // indent depth for blocks
                let mut depth = 0usize;
                let mut click_edit = None;
                let mut remove = None;
                let mut move_up = None;
                let mut move_down = None;

                for (idx, a) in actions.iter().enumerate() {
                    if a.kind == 2 {
                        depth = depth.saturating_sub(1);
                    }
                    let def = dnd::find_def(a.lib, a.id);
                    let caption = def
                        .map(|d| action_caption(d, a))
                        .unwrap_or_else(|| format!("? lib {} id {}", a.lib, a.id));
                    let is_sel = *selected_action == Some(idx);
                    let is_marker = a.kind != 0;
                    let is_q = def.map(|d| d.question).unwrap_or(false);

                    let row_h = 20.0;
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_h),
                        egui::Sense::click_and_drag(),
                    );
                    let painter = ui.painter_at(rect);
                    let bg = if is_sel {
                        theme::SEL_BLUE
                    } else if resp.hovered() {
                        egui::Color32::from_rgb(0xE6, 0xE3, 0xD5)
                    } else {
                        theme::WINDOW_BG
                    };
                    painter.rect_filled(rect, 0.0, bg);
                    // indent + bracket line for blocks
                    let x0 = rect.left() + 6.0 + depth as f32 * 14.0;
                    if depth > 0 {
                        painter.line_segment(
                            [
                                egui::pos2(
                                    rect.left() + 6.0 + (depth - 1) as f32 * 14.0 + 7.0,
                                    rect.top(),
                                ),
                                egui::pos2(
                                    rect.left() + 6.0 + (depth - 1) as f32 * 14.0 + 7.0,
                                    rect.bottom(),
                                ),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::from_gray(150)),
                        );
                    }
                    // icon: lib-colored tile with a marker overlay
                    let tile = egui::Rect::from_min_size(
                        egui::pos2(x0, rect.top() + 2.0),
                        egui::vec2(16.0, 16.0),
                    );
                    painter.rect_filled(tile, 0.0, lib_color(a.lib));
                    let mark = match a.kind {
                        1 => "{",
                        2 => "}",
                        3 => "e",
                        4 => "x",
                        5 => "R",
                        6 => "W",
                        7 => "U",
                        _ => "",
                    };
                    let mark = if is_q { "?" } else { mark };
                    if !mark.is_empty() {
                        painter.text(
                            tile.center(),
                            egui::Align2::CENTER_CENTER,
                            mark,
                            egui::FontId::monospace(11.0),
                            egui::Color32::WHITE,
                        );
                    }
                    // caption
                    let txt_color = if is_sel {
                        theme::SEL_TEXT
                    } else if is_marker {
                        egui::Color32::from_rgb(0x50, 0x30, 0x80)
                    } else if is_q {
                        egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)
                    } else {
                        theme::TEXT_DARK
                    };
                    painter.text(
                        egui::pos2(tile.right() + 5.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        caption,
                        egui::FontId::proportional(12.0),
                        txt_color,
                    );
                    // relative / applies-to badges
                    let mut badge = String::new();
                    if a.relative {
                        badge.push_str("rel ");
                    }
                    match a.applies_to {
                        -2 => badge.push_str("other"),
                        o if o >= 0 => badge.push_str("obj"),
                        _ => {}
                    }
                    if !badge.is_empty() {
                        painter.text(
                            egui::pos2(rect.right() - 6.0, rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            badge,
                            egui::FontId::monospace(9.0),
                            if is_sel {
                                egui::Color32::from_gray(220)
                            } else {
                                egui::Color32::from_gray(120)
                            },
                        );
                    }

                    // interactions
                    if resp.clicked() {
                        *selected_action = Some(idx);
                    }
                    if resp.double_clicked() {
                        click_edit = Some(idx);
                    }
                    // drag to reorder
                    if resp.drag_started() {
                        egui::DragAndDrop::set_payload(ui.ctx(), DndPayload::Move { index: idx });
                    }
                    // drop target: insert before this row
                    let pointer_inside = ui
                        .input(|i| i.pointer.hover_pos())
                        .map(|p| rect.contains(p))
                        .unwrap_or(false);
                    if pointer_inside {
                        if let Some(payload) = egui::DragAndDrop::payload::<DndPayload>(ui.ctx()) {
                            let ins_y = rect.center().y;
                            let hover = ui
                                .input(|i| i.pointer.hover_pos())
                                .map(|p| p.y)
                                .unwrap_or(0.0);
                            let before = hover < ins_y;
                            painter.line_segment(
                                [
                                    egui::pos2(
                                        rect.left(),
                                        if before { rect.top() } else { rect.bottom() },
                                    ),
                                    egui::pos2(
                                        rect.right(),
                                        if before { rect.top() } else { rect.bottom() },
                                    ),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)),
                            );
                            let _ = &payload;
                        }
                    }
                    resp.context_menu(|ui| {
                        if ui.button("Parameters…").clicked() {
                            click_edit = Some(idx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Move up").clicked() {
                            move_up = Some(idx);
                            ui.close_menu();
                        }
                        if ui.button("Move down").clicked() {
                            move_down = Some(idx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Cut").clicked() {
                            st.clipboard = vec![actions[idx].clone()];
                            remove = Some(idx);
                            ui.close_menu();
                        }
                        if ui.button("Copy").clicked() {
                            st.clipboard = vec![actions[idx].clone()];
                            ui.close_menu();
                        }
                        if ui.button("Paste").clicked() && !st.clipboard.is_empty() {
                            // clone into the current position
                            let item = st.clipboard[0].clone();
                            // borrow workaround: defer via menu closure result
                            // handled below through paste_requested
                            PENDING_PASTE.with(|c| *c.borrow_mut() = Some((idx, item)));
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Delete").clicked() {
                            remove = Some(idx);
                            ui.close_menu();
                        }
                    });

                    if a.kind == 1 {
                        depth += 1;
                    }
                }

                // drop zone after the last action (append)
                let (zrect, zresp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 26.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(zrect, 0.0, egui::Color32::from_rgb(0xF0, 0xEE, 0xE4));
                if zresp.hovered() {
                    if let Some(_) = egui::DragAndDrop::payload::<DndPayload>(ui.ctx()) {
                        ui.painter().rect_stroke(
                            zrect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)),
                        );
                    }
                }
                ui.painter().text(
                    zrect.center(),
                    egui::Align2::CENTER_CENTER,
                    "drop actions here",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_gray(140),
                );

                // resolve deferred edits/removals/pastes
                if let Some(idx) = click_edit {
                    st.edit = Some(idx);
                    st.edit_buf = actions[idx].clone();
                }
                if let Some(idx) = remove {
                    actions.remove(idx);
                    if *selected_action == Some(idx) {
                        *selected_action = None;
                    }
                }
                if let Some(idx) = move_up {
                    if idx > 0 {
                        actions.swap(idx, idx - 1);
                    }
                }
                if let Some(idx) = move_down {
                    if idx + 1 < actions.len() {
                        actions.swap(idx, idx + 1);
                    }
                }
                if let Some((idx, item)) = PENDING_PASTE.with(|c| c.borrow_mut().take()) {
                    let at = idx.min(actions.len());
                    actions.insert(at, item);
                }

                // handle drag release: insert/move at pointer position
                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(payload) = egui::DragAndDrop::payload::<DndPayload>(ui.ctx()) {
                        let insert_at = ui
                            .input(|i| i.pointer.hover_pos())
                            .and_then(|_p| {
                                // find the row whose center is nearest below the pointer
                                let mut at = actions.len();
                                for (idx, a) in actions.iter().enumerate() {
                                    let _ = a;
                                    at = idx;
                                    break;
                                }
                                let _ = &actions;
                                None::<usize>.or(Some(at))
                            })
                            .unwrap_or(actions.len());
                        match &*payload {
                            DndPayload::New { lib, id } => {
                                let mut na = DndAction::new(*lib, *id);
                                // set marker kinds for marker palette entries
                                na.kind = marker_kind(*lib, *id);
                                if na.params.is_empty() {
                                    na.params = default_params(*lib, *id);
                                }
                                let at = insert_at.min(actions.len());
                                actions.insert(at, na);
                            }
                            DndPayload::Move { index } => {
                                let mut from = *index;
                                let mut to = insert_at.min(actions.len());
                                if from < to {
                                    to -= 1;
                                } else {
                                    from = from.min(actions.len() - 1);
                                }
                                if from != to {
                                    let item = actions.remove(from);
                                    actions.insert(to, item);
                                }
                            }
                        }
                        egui::DragAndDrop::clear_payload(ui.ctx());
                    }
                }
            })
    });
    let _ = objects_snapshot;
}

thread_local! {
    static PENDING_PASTE: std::cell::RefCell<Option<(usize, DndAction)>> = const { std::cell::RefCell::new(None) };
}

fn marker_kind(lib: u32, id: u32) -> u8 {
    match (lib, id) {
        (4, 420) => 1,
        (4, 421) => 2,
        (4, 422) => 3,
        (4, 423) => 4,
        (4, 424) => 5,
        (4, 425) => 6,
        (4, 426) => 7,
        _ => 0,
    }
}

fn default_params(lib: u32, id: u32) -> Vec<DndParam> {
    let def = dnd::find_def(lib, id);
    def.map(|d| {
        d.params
            .iter()
            .map(|p| match p.kind {
                ParamKind::Text | ParamKind::Expr => DndParam::Str(String::new()),
                ParamKind::Menu(opts) => {
                    DndParam::Str(opts.first().map(|(_, v)| v.to_string()).unwrap_or_default())
                }
                ParamKind::DirectionGrid => DndParam::Real(32.0), // right by default
                _ => DndParam::Real(0.0),
            })
            .collect()
    })
    .unwrap_or_default()
}

/// The right column: library tabs + a vertical, named action list.
pub fn palette(
    ui: &mut egui::Ui,
    st: &mut DndEditorState,
    project: &mut Project,
    obj_idx: usize,
    ev_idx: usize,
) {
    ui.strong("Libraries");
    ui.horizontal_wrapped(|ui| {
        for lib in LIBS {
            let sel = st.lib_tab == lib.id;
            let rich = if sel {
                egui::RichText::new(lib.name)
                    .strong()
                    .color(egui::Color32::from_rgb(0x1E, 0x5E, 0xC0))
            } else {
                egui::RichText::new(lib.name)
            };
            if ui.selectable_label(sel, rich).clicked() {
                st.lib_tab = lib.id;
            }
        }
    });
    ui.separator();
    let lib = st.lib_tab;
    theme::sunken_frame().show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt(format!("pal_{}", lib))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let defs: Vec<&'static ActionDef> =
                    ACTIONS.iter().filter(|d| d.lib == lib).collect();
                let mut add_now: Option<&'static ActionDef> = None;
                for def in defs {
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 20.0),
                        egui::Sense::click_and_drag(),
                    );
                    let painter = ui.painter_at(rect);
                    let hover = resp.hovered();
                    painter.rect_filled(
                        rect,
                        0.0,
                        if hover {
                            egui::Color32::from_rgb(0xFB, 0xF0, 0xD0)
                        } else {
                            theme::WINDOW_BG
                        },
                    );
                    if hover {
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)),
                        );
                    }
                    // icon tile
                    let tile = egui::Rect::from_min_size(
                        rect.min + egui::vec2(3.0, 2.0),
                        egui::vec2(16.0, 16.0),
                    );
                    painter.rect_filled(tile, 1.0, lib_color(lib));
                    let mark = if def.question {
                        "?".to_string()
                    } else {
                        match (lib, def.id) {
                            (4, 420) => "{".into(),
                            (4, 421) => "}".into(),
                            (4, 422) => "e".into(),
                            (4, 423) => "x".into(),
                            (4, 424) => "R".into(),
                            (4, 425) => "W".into(),
                            (4, 426) => "U".into(),
                            (4, 401) => "{}".into(),
                            _ => lib_glyph(lib),
                        }
                    };
                    painter.text(
                        tile.center(),
                        egui::Align2::CENTER_CENTER,
                        mark,
                        egui::FontId::monospace(11.0),
                        egui::Color32::WHITE,
                    );
                    // name
                    let name_color = if def.question {
                        egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)
                    } else if (420..=426).contains(&def.id) && lib == 4 {
                        egui::Color32::from_rgb(0x50, 0x30, 0x80)
                    } else {
                        theme::TEXT_DARK
                    };
                    painter.text(
                        egui::pos2(tile.right() + 6.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        def.name,
                        egui::FontId::proportional(12.0),
                        name_color,
                    );
                    if resp.drag_started() {
                        egui::DragAndDrop::set_payload(
                            ui.ctx(),
                            DndPayload::New {
                                lib: def.lib,
                                id: def.id,
                            },
                        );
                    }
                    if resp.clicked() {
                        add_now = Some(def);
                    }
                    resp.on_hover_text(format!("{}\n{}", def.name, def.hint));
                }
                // click-to-append (into the end of the current event)
                if let Some(def) = add_now {
                    if let Some(obj) = project.objects.get_mut(obj_idx) {
                        if let Some(EventDef::Actions { actions, .. }) = obj.events.get_mut(ev_idx)
                        {
                            let mut na = DndAction::new(def.lib, def.id);
                            na.kind = marker_kind(def.lib, def.id);
                            na.params = default_params(def.lib, def.id);
                            actions.push(na);
                        }
                    }
                }
            });
    });
    ui.weak(egui::RichText::new("click or drag an action into the list").small());
}

fn lib_glyph(lib: u32) -> String {
    match lib {
        1 => "»".into(),
        2 => "+".into(),
        3 => "~".into(),
        4 => "::".into(),
        5 => "S".into(),
        6 => "*".into(),
        7 => "A".into(),
        _ => "•".into(),
    }
}

/// The parameter dialog for the action being edited.
pub fn params_dialog(
    ctx: &egui::Context,
    project: &Project,
    st: &mut DndEditorState,
    actions: &mut Vec<DndAction>,
    db: &NameDb,
) {
    let Some(idx) = st.edit else { return };
    if idx >= actions.len() {
        st.edit = None;
        return;
    }
    let Some(def) = dnd::find_def(st.edit_buf.lib, st.edit_buf.id) else {
        st.edit = None;
        return;
    };
    let mut close_ok = false;
    let mut close_cancel = false;
    let is_code = (st.edit_buf.lib, st.edit_buf.id) == (4, 401);
    let default_size = if is_code {
        [720.0, 640.0]
    } else {
        [560.0, 520.0]
    };
    egui::Window::new(format!("{} — action parameters", def.name))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(true)
        .default_size(default_size)
        .min_width(420.0)
        .min_height(280.0)
        .show(ctx, |ui| {
            if is_code {
                // Code action: full-height editor, no Grid
                let editor_h = ui.available_height() - 80.0;
                let mut buf = match st.edit_buf.params.first() {
                    Some(DndParam::Str(s)) => s.clone(),
                    Some(DndParam::Both { str: s, .. }) => s.clone(),
                    _ => String::new(),
                };
                let r = crate::codeedit::code_editor(
                    ui,
                    &mut buf,
                    &format!("dnd_code_{}", st.edit.unwrap_or(0)),
                    editor_h.max(200.0),
                    db,
                    &mut st.code_cs,
                );
                if r.changed() {
                    if st.edit_buf.params.is_empty() {
                        st.edit_buf.params.push(DndParam::Str(buf));
                    } else {
                        st.edit_buf.params[0] = DndParam::Str(buf);
                    }
                }
            } else {
                egui::Grid::new("dnd_params")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        for (pi, pd) in def.params.iter().enumerate() {
                            ui.label(format!("{}:", pd.name));
                            // ensure the param exists
                            while st.edit_buf.params.len() <= pi {
                                st.edit_buf.params.push(DndParam::Real(0.0));
                            }
                            match pd.kind {
                                ParamKind::DirectionGrid => {
                                    // GM8-style 3x3 arrow toggle grid
                                    let mask = pnum_of(&st.edit_buf, pi, 32.0) as i32;
                                    let mut new_mask = mask;
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(112.0, 112.0),
                                        egui::Layout::left_to_right(egui::Align::TOP),
                                        |ui| {
                                            // outer = columns (left→right), inner = rows (top→bottom)
                                            for c in 0..3 {
                                                ui.vertical(|ui| {
                                                    ui.set_min_size(egui::vec2(34.0, 106.0));
                                                    for r in 0..3 {
                                                        let idx = r * 3 + c;
                                                        let bit = 1i32 << idx;
                                                        let on = mask & bit != 0;
                                                        let is_stop = idx == 4;
                                                        let (rect, resp) = ui.allocate_exact_size(
                                                            egui::vec2(30.0, 30.0),
                                                            egui::Sense::click(),
                                                        );
                                                        let painter = ui.painter_at(rect);
                                                        let (fill, stroke) = if on {
                                                            (
                                                                egui::Color32::from_rgb(
                                                                    0x1E, 0x5E, 0xC0,
                                                                ),
                                                                egui::Color32::WHITE,
                                                            )
                                                        } else {
                                                            (
                                                                egui::Color32::from_rgb(
                                                                    0xF4, 0xF2, 0xEC,
                                                                ),
                                                                egui::Color32::from_rgb(
                                                                    0x8A, 0x86, 0x7A,
                                                                ),
                                                            )
                                                        };
                                                        painter.rect_filled(rect, 0.0, fill);
                                                        painter.rect_stroke(
                                                            rect,
                                                            0.0,
                                                            egui::Stroke::new(1.0, stroke),
                                                        );
                                                        if is_stop {
                                                            // stop = filled square in the middle
                                                            painter.rect_filled(
                                                                egui::Rect::from_center_size(
                                                                    rect.center(),
                                                                    egui::vec2(10.0, 10.0),
                                                                ),
                                                                0.0,
                                                                stroke,
                                                            );
                                                        } else {
                                                            // GM angles: 0=right, 90=up
                                                            let dir = match idx {
                                                                0 => 135.0,
                                                                1 => 90.0,
                                                                2 => 45.0,
                                                                3 => 180.0,
                                                                5 => 0.0,
                                                                6 => 225.0,
                                                                7 => 270.0,
                                                                _ => 315.0,
                                                            };
                                                            draw_arrow(&painter, rect, dir, stroke);
                                                        }
                                                        if resp.clicked() {
                                                            // single choice: exactly one direction (or stop)
                                                            new_mask = bit;
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                    );
                                    st.edit_buf.params[pi] = DndParam::Real(new_mask as f64);
                                }
                                ParamKind::Expr | ParamKind::Text => {
                                    let cur = match &st.edit_buf.params[pi] {
                                        DndParam::Str(s) => s.clone(),
                                        DndParam::Both { str: s, .. } => s.clone(),
                                        DndParam::Real(x) => {
                                            if x.fract().abs() < 1e-9 {
                                                (x.round() as i64).to_string()
                                            } else {
                                                x.to_string()
                                            }
                                        }
                                    };
                                    let mut buf = cur;
                                    {
                                        let r = ui.add(
                                            egui::TextEdit::singleline(&mut buf)
                                                .desired_width(200.0),
                                        );
                                        if r.changed() {
                                            st.edit_buf.params[pi] = DndParam::Str(buf);
                                        }
                                    }
                                }
                                ParamKind::Color => {
                                    let c = pnum_of(&st.edit_buf, pi, 0xFFFFFF as f64) as u32;
                                    let mut rgb = [
                                        (c & 0xFF) as f32 / 255.0,
                                        ((c >> 8) & 0xFF) as f32 / 255.0,
                                        ((c >> 16) & 0xFF) as f32 / 255.0,
                                    ];
                                    ui.color_edit_button_rgb(&mut rgb);
                                    st.edit_buf.params[pi] = DndParam::Real(
                                        ((rgb[0] * 255.0) as u32
                                            | (((rgb[1] * 255.0) as u32) << 8)
                                            | (((rgb[2] * 255.0) as u32) << 16))
                                            as f64,
                                    );
                                }
                                ParamKind::Menu(opts) => {
                                    let cur = pmenu_of(&st.edit_buf, pi, "");
                                    let label = opts
                                        .iter()
                                        .find(|(_, v)| *v == cur)
                                        .map(|(l, _)| l.to_string())
                                        .unwrap_or_else(|| cur.clone());
                                    ui.menu_button(label, |ui| {
                                        for (l, v) in opts {
                                            if ui.button(*l).clicked() {
                                                st.edit_buf.params[pi] =
                                                    DndParam::Str(v.to_string());
                                            }
                                        }
                                    });
                                }
                                ParamKind::Bool => {
                                    let on = pnum_of(&st.edit_buf, pi, 0.0) != 0.0;
                                    let mut b = on;
                                    ui.checkbox(&mut b, "");
                                    st.edit_buf.params[pi] = DndParam::Real(b as i32 as f64);
                                }
                                _ => {
                                    // resource selectors
                                    let (names, none_label) = resource_list(project, pd.kind);
                                    let cur = pexpr_of(&st.edit_buf, pi, "-1");
                                    let cur_i: i64 = cur.parse().unwrap_or(-1);
                                    let label = names
                                        .get(cur_i.max(0) as usize)
                                        .cloned()
                                        .unwrap_or_else(|| none_label.to_string());
                                    ui.menu_button(label, |ui| {
                                        if ui.button(none_label).clicked() {
                                            st.edit_buf.params[pi] = DndParam::Str("-1".into());
                                        }
                                        for (i, n) in names.iter().enumerate() {
                                            if ui.button(n).clicked() {
                                                st.edit_buf.params[pi] =
                                                    DndParam::Str(i.to_string());
                                            }
                                        }
                                    });
                                }
                            }
                            ui.end_row();
                        }
                    });
            } // else (non-code params Grid)
            ui.separator();
            ui.horizontal(|ui| {
                // applies to
                ui.label("Applies to:");
                let obj_names: Vec<String> =
                    project.objects.iter().map(|o| o.name.clone()).collect();
                let at = st.edit_buf.applies_to;
                let label = match at {
                    -1 => "self".to_string(),
                    -2 => "other".to_string(),
                    i => obj_names
                        .get(i.max(0) as usize)
                        .cloned()
                        .unwrap_or_else(|| "self".into()),
                };
                ui.menu_button(label, |ui| {
                    if ui.button("self").clicked() {
                        st.edit_buf.applies_to = -1;
                    }
                    if ui.button("other").clicked() {
                        st.edit_buf.applies_to = -2;
                    }
                    for (i, n) in obj_names.iter().enumerate() {
                        if ui.button(n).clicked() {
                            st.edit_buf.applies_to = i as i64;
                        }
                    }
                });
                if def.relative_ok {
                    ui.checkbox(&mut st.edit_buf.relative, "Relative");
                }
                if def.question {
                    ui.checkbox(&mut st.edit_buf.not, "NOT");
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                // Enter closes only when NOT typing in a text field
                let enter_ok = ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !ui.ctx().wants_keyboard_input();
                if ui.button("OK").clicked() || enter_ok {
                    close_ok = true;
                }
                // Escape closes only when the completion popup is not open
                let esc_ok = ui.input(|i| i.key_pressed(egui::Key::Escape)) && !st.code_cs.open;
                if ui.button("Cancel").clicked() || esc_ok {
                    close_cancel = true;
                }
            });
        });
    if close_ok {
        actions[idx] = st.edit_buf.clone();
        st.edit = None;
    }
    if close_cancel {
        st.edit = None;
    }
}

/// Draw an arrow pointing along `dir` (GM degrees: 0 = right, 90 = up).
fn draw_arrow(painter: &egui::Painter, rect: egui::Rect, dir: f64, color: egui::Color32) {
    let c = rect.center();
    let r = 9.0;
    let (sin, cos) = dir.to_radians().sin_cos();
    // GM: 0° = right, 90° = up (screen y grows down)
    let (dx, dy) = (cos as f32, -sin as f32);
    let tip = c + egui::vec2(dx * r, dy * r);
    let tail = c - egui::vec2(dx * r, dy * r);
    painter.line_segment([tail, tip], egui::Stroke::new(2.0, color));
    // arrowhead: two short lines at ±150° from the tip direction
    for da in [150.0f64, -150.0] {
        let a = (dir + da).to_radians();
        let (hx, hy) = (a.cos() as f32, -a.sin() as f32);
        painter.line_segment(
            [tip, tip + egui::vec2(hx * 5.0, hy * 5.0)],
            egui::Stroke::new(2.0, color),
        );
    }
}

fn resource_list(project: &Project, kind: ParamKind) -> (Vec<String>, &'static str) {
    match kind {
        ParamKind::Sprite => (
            project.sprites.iter().map(|s| s.name.clone()).collect(),
            "<no sprite>",
        ),
        ParamKind::Sound => (
            project.sounds.iter().map(|s| s.name.clone()).collect(),
            "<no sound>",
        ),
        ParamKind::Background => (
            project.backgrounds.iter().map(|b| b.name.clone()).collect(),
            "<no background>",
        ),
        ParamKind::Path => (
            project.paths.iter().map(|p| p.name.clone()).collect(),
            "<no path>",
        ),
        ParamKind::Script => (
            project.scripts.iter().map(|s| s.name.clone()).collect(),
            "<no script>",
        ),
        ParamKind::Timeline => (
            project.timelines.iter().map(|t| t.name.clone()).collect(),
            "<no timeline>",
        ),
        ParamKind::Font => (
            project.fonts.iter().map(|f| f.name.clone()).collect(),
            "<default font>",
        ),
        ParamKind::Object => (
            project.objects.iter().map(|o| o.name.clone()).collect(),
            "<no object>",
        ),
        ParamKind::Room => (
            project.rooms.iter().map(|r| r.name.clone()).collect(),
            "<no room>",
        ),
        _ => (vec![], "<none>"),
    }
}

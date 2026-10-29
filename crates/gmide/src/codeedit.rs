//! GML code editor: syntax highlighting, line numbers, autocompletion.

use eframe::egui;
use eframe::egui::text::LayoutJob;
use std::collections::HashSet;
use std::sync::OnceLock;

// ---------------------------------------------------------------- colors

fn c_keyword() -> egui::Color32 {
    egui::Color32::from_rgb(0x00, 0x00, 0xC0)
}
fn c_function() -> egui::Color32 {
    egui::Color32::from_rgb(0x10, 0x50, 0xB0)
}
fn c_constant() -> egui::Color32 {
    egui::Color32::from_rgb(0x80, 0x00, 0x80)
}
fn c_string() -> egui::Color32 {
    egui::Color32::from_rgb(0xA0, 0x20, 0x20)
}
fn c_number() -> egui::Color32 {
    egui::Color32::from_rgb(0xC0, 0x60, 0x00)
}
fn c_comment() -> egui::Color32 {
    egui::Color32::from_rgb(0x30, 0x8A, 0x30)
}
fn c_plain() -> egui::Color32 {
    egui::Color32::from_rgb(0x14, 0x14, 0x14)
}

pub const CODE_FONT_SIZE: f32 = 13.0;

fn builtin_functions() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| gmengine::names::FUNCTION_NAMES.iter().copied().collect())
}

fn builtin_constants() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| gmengine::names::CONSTANT_NAMES.iter().copied().collect())
}

/// Build a syntax-highlighted layout job.
pub fn highlight_job(code: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let fmt = |color: egui::Color32| egui::TextFormat {
        font_id: egui::FontId::monospace(CODE_FONT_SIZE),
        color,
        ..Default::default()
    };
    let plain = fmt(c_plain());
    let kw = fmt(c_keyword());
    let fun = fmt(c_function());
    let konst = fmt(c_constant());
    let str_c = fmt(c_string());
    let num_c = fmt(c_number());
    let com = fmt(c_comment());

    let items = gml::tokenize(code).unwrap_or_default();
    let b = code.as_bytes();
    let mut pos = 0usize;
    for i in 0..items.len() {
        let it = &items[i];
        if it.start > pos {
            job.append(&code[pos..it.start], 0.0, plain.clone());
        }
        let text = std::str::from_utf8(&b[it.start..it.end.min(b.len())]).unwrap_or("");
        let f = match &it.tok {
            gml::Tok::Num(_) => num_c.clone(),
            gml::Tok::Str(_) => str_c.clone(),
            gml::Tok::Comment(_) => com.clone(),
            gml::Tok::Id(name) => {
                let lower = name.to_ascii_lowercase();
                if builtin_constants().contains(lower.as_str()) {
                    konst.clone()
                } else if builtin_functions().contains(lower.as_str()) {
                    fun.clone()
                } else {
                    let next = items.get(i + 1).map(|n| &n.tok);
                    if matches!(next, Some(gml::Tok::LParen)) {
                        fun.clone()
                    } else {
                        plain.clone()
                    }
                }
            }
            gml::Tok::Eof => {
                pos = it.end;
                continue;
            }
            gml::Tok::KwIf
            | gml::Tok::KwElse
            | gml::Tok::KwWhile
            | gml::Tok::KwDo
            | gml::Tok::KwUntil
            | gml::Tok::KwFor
            | gml::Tok::KwRepeat
            | gml::Tok::KwSwitch
            | gml::Tok::KwCase
            | gml::Tok::KwDefault
            | gml::Tok::KwBreak
            | gml::Tok::KwExit
            | gml::Tok::KwWith
            | gml::Tok::KwVar
            | gml::Tok::KwGlobalVar
            | gml::Tok::KwDiv
            | gml::Tok::KwMod
            | gml::Tok::And
            | gml::Tok::Or
            | gml::Tok::Xor
            | gml::Tok::Not => kw.clone(),
            _ => plain.clone(),
        };
        job.append(text, 0.0, f);
        pos = it.end;
    }
    if pos < code.len() {
        job.append(&code[pos..], 0.0, plain.clone());
    }
    job
}

// ---------------------------------------------------------------- completion

#[derive(Clone, Copy, PartialEq)]
pub enum SuggestKind {
    Function,
    Keyword,
    Constant,
    Sprite,
    Sound,
    Background,
    Path,
    Script,
    Timeline,
    Font,
    Object,
    Room,
    InstanceVar,
}

impl SuggestKind {
    fn label(self) -> &'static str {
        match self {
            SuggestKind::Function => "fn",
            SuggestKind::Keyword => "kw",
            SuggestKind::Constant => "const",
            SuggestKind::Sprite => "sprite",
            SuggestKind::Sound => "sound",
            SuggestKind::Background => "bg",
            SuggestKind::Path => "path",
            SuggestKind::Script => "script",
            SuggestKind::Timeline => "timeline",
            SuggestKind::Font => "font",
            SuggestKind::Object => "object",
            SuggestKind::Room => "room",
            SuggestKind::InstanceVar => "var",
        }
    }
    fn color(self) -> egui::Color32 {
        match self {
            SuggestKind::Function => egui::Color32::from_rgb(0x10, 0x50, 0xB0),
            SuggestKind::Keyword => egui::Color32::from_rgb(0x00, 0x00, 0xC0),
            SuggestKind::Constant => egui::Color32::from_rgb(0x80, 0x00, 0x80),
            SuggestKind::InstanceVar => egui::Color32::from_rgb(0x00, 0x60, 0x60),
            _ => egui::Color32::from_rgb(0x50, 0x6A, 0x30),
        }
    }
}

pub struct Suggestion {
    pub name: String,
    pub kind: SuggestKind,
    pub hint: &'static str,
}

pub struct NameDb {
    pub entries: Vec<(String, SuggestKind)>,
}

impl NameDb {
    pub fn build(project: &gmformat::Project) -> Self {
        let mut entries: Vec<(String, SuggestKind)> = Vec::new();
        for f in gmengine::names::FUNCTION_NAMES {
            entries.push((f.to_string(), SuggestKind::Function));
        }
        for k in gml::KEYWORDS {
            entries.push((k.to_string(), SuggestKind::Keyword));
        }
        for c in gmengine::names::CONSTANT_NAMES {
            entries.push((c.to_string(), SuggestKind::Constant));
        }
        for v in INSTANCE_VARS {
            entries.push((v.to_string(), SuggestKind::InstanceVar));
        }
        let mut add = |items: &Vec<String>, kind: SuggestKind| {
            for n in items {
                let clean = n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && n.chars()
                        .next()
                        .map(|c| c.is_ascii_alphabetic() || c == '_')
                        .unwrap_or(false);
                if clean {
                    entries.push((n.clone(), kind));
                }
            }
        };
        add(
            &project.sprites.iter().map(|s| s.name.clone()).collect(),
            SuggestKind::Sprite,
        );
        add(
            &project.sounds.iter().map(|s| s.name.clone()).collect(),
            SuggestKind::Sound,
        );
        add(
            &project.backgrounds.iter().map(|b| b.name.clone()).collect(),
            SuggestKind::Background,
        );
        add(
            &project.paths.iter().map(|p| p.name.clone()).collect(),
            SuggestKind::Path,
        );
        add(
            &project.scripts.iter().map(|s| s.name.clone()).collect(),
            SuggestKind::Script,
        );
        add(
            &project.timelines.iter().map(|t| t.name.clone()).collect(),
            SuggestKind::Timeline,
        );
        add(
            &project.fonts.iter().map(|f| f.name.clone()).collect(),
            SuggestKind::Font,
        );
        add(
            &project.objects.iter().map(|o| o.name.clone()).collect(),
            SuggestKind::Object,
        );
        add(
            &project.rooms.iter().map(|r| r.name.clone()).collect(),
            SuggestKind::Room,
        );
        NameDb { entries }
    }

    fn complete(&self, prefix: &str) -> Vec<Suggestion> {
        let lower = prefix.to_ascii_lowercase();
        let mut out: Vec<Suggestion> = Vec::new();
        for (name, kind) in &self.entries {
            if name.to_ascii_lowercase().starts_with(&lower) && name.len() > prefix.len() {
                let hint = gmengine::names::FUNCTION_HINTS
                    .iter()
                    .find(|(f, _)| *f == name.as_str())
                    .map(|(_, h)| *h)
                    .unwrap_or("");
                out.push(Suggestion {
                    name: name.clone(),
                    kind: *kind,
                    hint,
                });
            }
        }
        out.sort_by_key(|s| match s.kind {
            SuggestKind::Function => 0,
            SuggestKind::Keyword => 1,
            SuggestKind::Constant => 2,
            SuggestKind::InstanceVar => 3,
            _ => 4,
        });
        out.truncate(12);
        out
    }
}

const INSTANCE_VARS: &[&str] = &[
    "x",
    "y",
    "xstart",
    "ystart",
    "xprevious",
    "yprevious",
    "hspeed",
    "vspeed",
    "speed",
    "direction",
    "friction",
    "gravity",
    "gravity_direction",
    "visible",
    "solid",
    "persistent",
    "depth",
    "sprite_index",
    "mask_index",
    "image_index",
    "image_speed",
    "image_xscale",
    "image_yscale",
    "image_angle",
    "image_alpha",
    "image_blend",
    "alarm",
    "score",
    "lives",
    "health",
    "mouse_x",
    "mouse_y",
    "room",
    "room_width",
    "room_height",
    "room_speed",
    "fps",
    "path_index",
    "path_position",
    "path_speed",
    "path_endaction",
    "path_absolute",
    "timeline_index",
    "timeline_running",
    "timeline_speed",
    "timeline_position",
    "timeline_loop",
    "object_index",
    "id",
    "sprite_width",
    "sprite_height",
    "bbox_left",
    "bbox_top",
    "bbox_right",
    "bbox_bottom",
];

pub struct CompletionState {
    pub open: bool,
    pub selected: usize,
    pub options: Vec<Suggestion>,
}
impl Default for CompletionState {
    fn default() -> Self {
        CompletionState {
            open: false,
            selected: 0,
            options: vec![],
        }
    }
}

// ---------------------------------------------------------------- editor

/// The code editor: gutter with line numbers + syntax-highlighted text +
/// autocompletion, all inside one scroll area.
pub fn code_editor(
    ui: &mut egui::Ui,
    buf: &mut String,
    id_salt: &str,
    height: f32,
    db: &NameDb,
    cs: &mut CompletionState,
) -> egui::Response {
    let line_count = buf.lines().count().max(1);

    // outer frame: white bg + thin border (TextEdit itself is frameless)
    let frame = egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(120)))
        .inner_margin(0.0);

    frame
        .show(ui, |ui| {
            let inner = egui::ScrollArea::vertical()
                .id_salt(format!("scr_{}", id_salt))
                .auto_shrink([false, false])
                .max_height(height)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // ---- gutter: line numbers as one multi-line label ----
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.set_min_width(32.0);
                            ui.add_space(4.0);
                            let nums: String = (1..=line_count)
                                .map(|i| format!("{:>3}\n", i))
                                .collect::<String>();
                            ui.monospace(
                                egui::RichText::new(nums.trim_end())
                                    .size(CODE_FONT_SIZE)
                                    .color(egui::Color32::from_gray(140)),
                            );
                        });
                        ui.separator();
                        ui.add_space(2.0);

                        // ---- code editor (frameless: no border, no hover effect) ----
                        let mut layouter =
                            |ui: &egui::Ui,
                             text: &str,
                             _wrap: f32|
                             -> std::sync::Arc<egui::Galley> {
                                let job = highlight_job(text);
                                ui.fonts(|f| f.layout_job(job))
                            };
                        let edit = egui::TextEdit::multiline(buf)
                            .desired_width(ui.available_width())
                            .code_editor()
                            .frame(false) // ← no border/hover around the text field
                            .margin(egui::Margin::symmetric(4.0, 4.0))
                            .id(egui::Id::new(id_salt))
                            .layouter(&mut layouter);
                        let output = edit.show(ui);
                        let resp = output.response;
                        let galley = output.galley.clone();

                        // ---------- completion ----------
                        // ---- Tab: handle IMMEDIATELY, before prefix/popup logic ----
                        // (the focused block below would close the popup because
                        //  \t makes the prefix empty, killing the completion)
                        let tab_now = cs.open
                            && !cs.options.is_empty()
                            && ui.input(|i| i.key_pressed(egui::Key::Tab));
                        if tab_now {
                            let opt = cs.options[cs.selected].name.clone();
                            let mut chars: Vec<char> = buf.chars().collect();
                            let mut cc = output
                                .cursor_range
                                .map(|cr| cr.primary.ccursor.index)
                                .unwrap_or(chars.len())
                                .min(chars.len());
                            // undo the \t the TextEdit just inserted
                            if cc > 0 && cc <= chars.len() && chars[cc - 1] == '\t' {
                                chars.remove(cc - 1);
                                cc -= 1;
                            }
                            // find word start
                            let mut start = cc;
                            while start > 0
                                && (chars[start - 1].is_ascii_alphanumeric()
                                    || chars[start - 1] == '_')
                            {
                                start -= 1;
                            }
                            // replace [start..cc] with the suggestion
                            let mut nc: Vec<char> = Vec::with_capacity(chars.len());
                            nc.extend_from_slice(&chars[..start]);
                            nc.extend(opt.chars());
                            nc.extend_from_slice(&chars[cc..]);
                            *buf = nc.into_iter().collect();
                            // set cursor after the inserted word
                            let new_cc = start + opt.chars().count();
                            let mut cursor = galley.from_ccursor(egui::text::CCursor::new(0));
                            cursor.ccursor = egui::text::CCursor::new(new_cc);
                            if let Some(mut state) =
                                egui::TextEdit::load_state(ui.ctx(), egui::Id::new(id_salt))
                            {
                                state.cursor.set_range(Some(
                                    egui::text_selection::CursorRange::one(cursor),
                                ));
                                state.store(ui.ctx(), egui::Id::new(id_salt));
                            }
                            cs.open = false;
                            ui.memory_mut(|m| m.request_focus(egui::Id::new(id_salt)));
                            ui.ctx().request_repaint();
                            return resp;
                        }

                        let cursor_char = output.cursor_range.map(|cr| cr.primary.ccursor.index);
                        let prefix = cursor_char
                            .map(|cc| {
                                let chars: Vec<char> = buf.chars().collect();
                                let cc = cc.min(chars.len());
                                let mut start = cc;
                                while start > 0
                                    && (chars[start - 1].is_ascii_alphanumeric()
                                        || chars[start - 1] == '_')
                                {
                                    start -= 1;
                                }
                                chars[start..cc].iter().collect::<String>()
                            })
                            .unwrap_or_default();

                        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Space))
                            && resp.has_focus()
                        {
                            cs.open = true;
                        }
                        // auto-open while typing (>= 2 chars)
                        if resp.changed()
                            && prefix.chars().count() >= 2
                            && !db.complete(&prefix).is_empty()
                        {
                            cs.open = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            cs.open = false;
                        }

                        let my_popup_id = egui::Id::new(format!("ac_{}", id_salt));
                        if !cs.open {
                            if ui.memory(|mem| mem.is_popup_open(my_popup_id)) {
                                ui.memory_mut(|mem| mem.close_popup());
                            }
                        }

                        let focused = resp.has_focus();
                        if focused && cs.open {
                            if prefix.is_empty() {
                                cs.open = false;
                            } else {
                                cs.options = db.complete(&prefix);
                                if cs.options.is_empty() {
                                    cs.open = false;
                                } else {
                                    if resp.changed() {
                                        cs.selected = 0;
                                    }
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                        cs.selected = (cs.selected + 1).min(cs.options.len() - 1);
                                    }
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                        cs.selected = cs.selected.saturating_sub(1);
                                    }

                                    // popup near the cursor position (not below the whole widget)
                                    let popup_pos = output
                                        .cursor_range
                                        .map(|cr| {
                                            let row = cr.primary.rcursor.row;
                                            let col = cr.primary.rcursor.column;
                                            // approximate character width for monospace 13px
                                            let char_w = 7.8;
                                            let line_h = 16.0;
                                            egui::pos2(
                                                output.galley_pos.x + col as f32 * char_w,
                                                output.galley_pos.y + (row + 1) as f32 * line_h,
                                            )
                                        })
                                        .unwrap_or(resp.rect.left_bottom());

                                    let mut accepted: Option<String> = None;
                                    let options = &cs.options;
                                    let sel = cs.selected;
                                    let popup_id = my_popup_id;

                                    egui::Area::new(popup_id)
                                        .order(egui::Order::Foreground)
                                        .fixed_pos(popup_pos)
                                        .show(ui.ctx(), |ui| {
                                            egui::Frame::none()
                                                .fill(egui::Color32::WHITE)
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    egui::Color32::from_gray(110),
                                                ))
                                                .inner_margin(2.0)
                                                .show(ui, |ui| {
                                                    ui.set_min_width(250.0);
                                                    ui.set_max_height(190.0);
                                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                                        for (i, opt) in options.iter().enumerate() {
                                                            let is_sel = i == sel;
                                                            let row = ui_selectable(
                                                                ui, &opt.name, opt, is_sel,
                                                            );
                                                            if row.clicked() {
                                                                accepted = Some(opt.name.clone());
                                                            }
                                                        }
                                                    });
                                                    ui.label(
                                                        egui::RichText::new(
                                                            "Tab: insert   |   Esc: close",
                                                        )
                                                        .small()
                                                        .weak(),
                                                    );
                                                });
                                        });

                                    if let Some(name) = accepted {
                                        if let Some(start_char) = last_word_start(buf) {
                                            let chars: Vec<char> = buf.chars().collect();
                                            let skip = prefix.chars().count();
                                            let mut nc: Vec<char> = Vec::new();
                                            nc.extend(chars[..start_char].iter());
                                            nc.extend(name.chars());
                                            if start_char + skip <= chars.len() {
                                                nc.extend(chars[start_char + skip..].iter());
                                            }
                                            *buf = nc.into_iter().collect();
                                        }
                                        cs.open = false;
                                    }
                                }
                            }
                        }
                        // ---- Tab/arrows: work even if egui stole focus ----
                        if cs.open && !cs.options.is_empty() {
                            let tab = ui.input(|i| i.key_pressed(egui::Key::Tab));
                            let adown = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                            let aup = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                            let edit_id = egui::Id::new(id_salt);
                            if tab {
                                // The TextEdit already inserted a \t at the cursor.
                                // Undo it, then apply the completion.
                                let opt = cs.options[cs.selected].name.clone();
                                let mut chars: Vec<char> = buf.chars().collect();
                                let mut cc = cursor_char.unwrap_or(chars.len()).min(chars.len());

                                // remove the \t that TextEdit just inserted
                                if cc > 0 && cc <= chars.len() && chars[cc - 1] == '\t' {
                                    chars.remove(cc - 1);
                                    cc -= 1;
                                }

                                // find the word start
                                let mut start = cc;
                                while start > 0
                                    && (chars[start - 1].is_ascii_alphanumeric()
                                        || chars[start - 1] == '_')
                                {
                                    start -= 1;
                                }

                                // replace [start..cc] with the suggestion
                                let mut nc: Vec<char> = Vec::with_capacity(chars.len());
                                nc.extend_from_slice(&chars[..start]);
                                nc.extend(opt.chars());
                                nc.extend_from_slice(&chars[cc..]);
                                *buf = nc.into_iter().collect();

                                let new_cc = start + opt.chars().count();
                                let mut cursor = galley.from_ccursor(egui::text::CCursor::new(0));
                                cursor.ccursor = egui::text::CCursor::new(new_cc);
                                if let Some(mut state) =
                                    egui::TextEdit::load_state(ui.ctx(), edit_id)
                                {
                                    state.cursor.set_range(Some(
                                        egui::text_selection::CursorRange::one(cursor),
                                    ));
                                    state.store(ui.ctx(), edit_id);
                                }
                                cs.open = false;
                                ui.memory_mut(|m| m.request_focus(edit_id));
                                ui.ctx().request_repaint();
                            } else if adown || aup {
                                ui.memory_mut(|m| m.request_focus(edit_id));
                            }
                        }

                        resp
                    })
                    .inner
                })
                .inner;
            inner
        })
        .inner
}

fn ui_selectable(ui: &mut egui::Ui, name: &str, opt: &Suggestion, sel: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::click());
    if sel {
        ui.painter().rect_filled(rect, 0.0, crate::theme::SEL_BLUE);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(0xE8, 0xF0, 0xFC));
    }
    ui.painter().text(
        rect.min + egui::vec2(4.0, 8.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::monospace(12.5),
        if sel {
            egui::Color32::WHITE
        } else {
            opt.kind.color()
        },
    );
    if !opt.hint.is_empty() {
        ui.painter().text(
            rect.min + egui::vec2(rect.width() - 4.0, 8.0),
            egui::Align2::RIGHT_CENTER,
            opt.hint,
            egui::FontId::monospace(11.0),
            if sel {
                egui::Color32::from_rgb(0xD8, 0xE4, 0xF8)
            } else {
                egui::Color32::from_gray(130)
            },
        );
    }
    ui.painter().text(
        rect.min + egui::vec2(rect.width() - 150.0, 8.0),
        egui::Align2::RIGHT_CENTER,
        opt.kind.label(),
        egui::FontId::monospace(10.0),
        if sel {
            egui::Color32::from_rgb(0xC8, 0xD8, 0xF0)
        } else {
            egui::Color32::from_gray(150)
        },
    );
    response
}

fn last_word_start(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = chars.len();
    let had = chars
        .iter()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == '_')
        .count();
    if had == 0 {
        return None;
    }
    i -= had;
    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section_colors(code: &str) -> Vec<(String, egui::Color32)> {
        let job = highlight_job(code);
        job.sections
            .iter()
            .map(|s| {
                (
                    code[s.byte_range.start..s.byte_range.end].to_string(),
                    s.format.color,
                )
            })
            .collect()
    }

    fn find(code: &str, text: &str) -> egui::Color32 {
        section_colors(code)
            .into_iter()
            .find(|(t, _)| t == text)
            .map(|(_, c)| c)
            .unwrap_or_else(|| panic!("no section '{}' in {:?}", text, section_colors(code)))
    }

    #[test]
    fn highlights_keywords_functions_constants() {
        assert_eq!(find("if (x) {}", "if"), c_keyword());
        assert_eq!(
            find("move_towards_point(1, 2)", "move_towards_point"),
            c_function()
        );
        assert_eq!(find("vk_left", "vk_left"), c_constant());
        assert_eq!(find("c_red", "c_red"), c_constant());
        assert_eq!(find("x = \"abc\"", "\"abc\""), c_string());
        assert_eq!(find("x = 42", "42"), c_number());
        assert_eq!(find("// hi", "// hi"), c_comment());
        assert_eq!(find("user_var", "user_var"), c_plain());
        assert_eq!(find("my_script(1)", "my_script"), c_function());
    }

    #[test]
    fn completion_finds_functions_and_constants() {
        let db = NameDb::build(&gmformat::Project::default());
        assert!(db
            .complete("instance_cre")
            .iter()
            .any(|o| o.name == "instance_create"));
        assert!(db.complete("vk_le").iter().any(|o| o.name == "vk_left"));
        assert!(db.complete("repea").iter().any(|o| o.name == "repeat"));
        let opts = db.complete("move_towards_p");
        assert!(opts
            .iter()
            .any(|o| o.name == "move_towards_point" && !o.hint.is_empty()));
    }

    #[test]
    fn completion_includes_project_resources() {
        let mut project = gmformat::Project::default();
        project.objects.push(gmformat::GmObject::new("obj_ball"));
        project
            .sprites
            .push(gmformat::Sprite::new("spr_ball", 8, 8));
        let db = NameDb::build(&project);
        assert!(db.complete("obj_ba").iter().any(|o| o.name == "obj_ball"));
        assert!(db.complete("spr_ba").iter().any(|o| o.name == "spr_ball"));
    }
}

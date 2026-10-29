#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod codeedit;
mod dndui;
mod editors;
mod icons;
mod theme;

use codeedit::{CompletionState, NameDb};
use eframe::egui;
use gmformat::*;
use icons::{icon_and_text, icon_button, Icon};
use std::collections::HashMap;
use std::path::PathBuf;

// ------------------------------------------------------------------ state

#[derive(Clone, Copy, PartialEq, Debug)]
enum ResRef {
    Sprite(usize),
    Sound(usize),
    Background(usize),
    Path(usize),
    Script(usize),
    Timeline(usize),
    Font(usize),
    Object(usize),
    Room(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum AddKind {
    Sprite,
    Sound,
    Background,
    Path,
    Script,
    Timeline,
    Font,
    Object,
    Room,
}

impl AddKind {
    fn icon(self) -> Icon {
        match self {
            AddKind::Sprite => Icon::Sprite,
            AddKind::Sound => Icon::Sound,
            AddKind::Background => Icon::Background,
            AddKind::Path => Icon::Path,
            AddKind::Script => Icon::Script,
            AddKind::Timeline => Icon::Timeline,
            AddKind::Font => Icon::Font,
            AddKind::Object => Icon::Object,
            AddKind::Room => Icon::Room,
        }
    }
    fn title(self) -> &'static str {
        match self {
            AddKind::Sprite => "Sprite",
            AddKind::Sound => "Sound",
            AddKind::Background => "Background",
            AddKind::Path => "Path",
            AddKind::Script => "Script",
            AddKind::Timeline => "Timeline",
            AddKind::Font => "Font",
            AddKind::Object => "Object",
            AddKind::Room => "Room",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
    Picker,
}

// console line levels
const INFO: u8 = 0;
const ERR: u8 = 1;
const OK: u8 = 2;

struct App {
    path: Option<PathBuf>,
    project: Project,
    sel: Option<ResRef>,
    rename: Option<(ResRef, String)>,
    confirm_delete: Option<ResRef>,
    console: Vec<(String, u8)>,
    child: Option<std::process::Child>,
    frames_cache: HashMap<usize, Vec<Vec<u8>>>,
    textures: HashMap<String, egui::TextureHandle>,
    thumbs_ready: std::collections::HashSet<String>,
    sel_event: HashMap<usize, usize>,
    add_event_for: Option<usize>,
    completion: HashMap<String, CompletionState>,
    // sprite editor
    spr_frame: usize,
    spr_zoom: f32,
    spr_tool: Tool,
    spr_color: [u8; 3],
    spr_drag: Option<(i32, i32)>,
    // room editor
    room_zoom: f32,
    room_snap: f64,
    room_sel_obj: i64,
    room_mode_tiles: bool,
    room_tile_bg: i64,
    room_tile_size: i32,
    room_tile_depth: i32,
    // path editor
    path_grid: f64,
    // audio preview
    audio: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
    sink: Option<rodio::Sink>,
    ui_font: String,
    mono_font: String,
    // help window
    help_open: bool,
    help_search: String,
    // D&D editor state
    dnd: dndui::DndEditorState,
    dnd_sel_action: Option<usize>,
    dnd_open_for: Option<(usize, usize)>,
    // export
    export_dialog: bool,
    export_name: String,
    export_dir: Option<PathBuf>,
    export_job: Option<ExportJob>,
}

/// A release build with an embedded project, running in the background.
struct ExportJob {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    out_dir: PathBuf,
    logs: std::sync::Arc<std::sync::Mutex<Vec<(String, u8)>>>,
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (ui_font, mono_font) = theme::apply(&cc.egui_ctx);
        let mut app = App {
            ui_font,
            mono_font,
            path: None,
            project: Project::default(),
            sel: None,
            rename: None,
            confirm_delete: None,
            console: vec![(
                "RGM IDE — a Game Maker 8.1 clone. Create resources in the tree on the left."
                    .into(),
                INFO,
            )],
            child: None,
            frames_cache: HashMap::new(),
            textures: HashMap::new(),
            thumbs_ready: std::collections::HashSet::new(),
            sel_event: HashMap::new(),
            add_event_for: None,
            completion: HashMap::new(),
            spr_frame: 0,
            spr_zoom: 6.0,
            spr_tool: Tool::Pencil,
            spr_color: [255, 255, 255],
            spr_drag: None,
            room_zoom: 1.0,
            room_snap: 16.0,
            room_sel_obj: -1,
            room_mode_tiles: false,
            room_tile_bg: -1,
            room_tile_size: 32,
            room_tile_depth: 0,
            path_grid: 16.0,
            audio: None,
            sink: None,
            help_open: false,
            help_search: String::new(),
            dnd: dndui::DndEditorState::default(),
            dnd_sel_action: None,
            dnd_open_for: None,
            export_dialog: false,
            export_name: String::new(),
            export_dir: None,
            export_job: None,
        };
        app.console.insert(
            0,
            (
                format!("UI font: {} | code font: {}", app.ui_font, app.mono_font),
                INFO,
            ),
        );
        app
    }

    fn log(&mut self, level: u8, msg: impl std::convert::Into<String>) {
        let msg: String = msg.into();
        self.console.push((msg, level));
        if self.console.len() > 400 {
            self.console.drain(..self.console.len() - 400);
        }
    }

    fn unique_name(project: &Project, base: &str) -> String {
        let mut name = base.to_string();
        let mut n = 0;
        loop {
            let taken = project.sprites.iter().any(|s| s.name == name)
                || project.sounds.iter().any(|s| s.name == name)
                || project.backgrounds.iter().any(|b| b.name == name)
                || project.paths.iter().any(|p| p.name == name)
                || project.scripts.iter().any(|s| s.name == name)
                || project.timelines.iter().any(|t| t.name == name)
                || project.fonts.iter().any(|f| f.name == name)
                || project.objects.iter().any(|o| o.name == name)
                || project.rooms.iter().any(|r| r.name == name);
            if !taken {
                return name;
            }
            n += 1;
            name = format!("{}_{}", base, n);
        }
    }

    // -------- sprite frame cache --------

    fn frames(&mut self, idx: usize) -> &Vec<Vec<u8>> {
        let want = self.project.sprites[idx].frames.len();
        if self.frames_cache.get(&idx).map(|f| f.len()) != Some(want) {
            let frames: Vec<Vec<u8>> = self.project.sprites[idx]
                .frames
                .iter()
                .map(|f| decode_rgba(f))
                .collect();
            self.frames_cache.insert(idx, frames);
        }
        self.frames_cache.get(&idx).unwrap()
    }

    fn frames_mut(&mut self, idx: usize) -> &mut Vec<Vec<u8>> {
        self.frames(idx);
        self.frames_cache.get_mut(&idx).unwrap()
    }

    fn sync_frames(&mut self, idx: usize) {
        if let Some(cache) = self.frames_cache.get(&idx) {
            let frames: Vec<String> = cache.iter().map(|f| encode_rgba(f)).collect();
            self.project.sprites[idx].frames = frames;
        }
        // thumbnail for this sprite is now stale
        self.thumbs_ready.remove(&format!("thumb_spr_{}", idx));
        self.textures.remove(&format!("thumb_spr_{}", idx));
    }

    fn invalidate_thumbs(&mut self) {
        self.thumbs_ready.clear();
        self.textures.retain(|k, _| !k.starts_with("thumb_"));
    }

    // -------- texture helpers --------

    fn sprite_texture(&mut self, ctx: &egui::Context, spr: usize, frame: usize) -> egui::TextureId {
        let key = format!("spr_{}_{}", spr, frame);
        let (w, h) = (
            self.project.sprites[spr].width,
            self.project.sprites[spr].height,
        );
        let data = self
            .frames(spr)
            .get(frame)
            .cloned()
            .unwrap_or_else(|| vec![0; (w * h * 4) as usize]);
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &data);
        if let Some(tex) = self.textures.get_mut(&key) {
            tex.set(image, egui::TextureOptions::NEAREST);
            tex.id()
        } else {
            let tex = ctx.load_texture(&key, image, egui::TextureOptions::NEAREST);
            let id = tex.id();
            self.textures.insert(key, tex);
            id
        }
    }

    fn image_texture(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        rgba: &[u8],
        w: u32,
        h: u32,
    ) -> egui::TextureId {
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba);
        if let Some(tex) = self.textures.get_mut(key) {
            tex.set(image, egui::TextureOptions::NEAREST);
            tex.id()
        } else {
            let tex = ctx.load_texture(key, image, egui::TextureOptions::NEAREST);
            let id = tex.id();
            self.textures.insert(key.to_string(), tex);
            id
        }
    }

    /// Create a texture once; later calls return the cached id without
    /// re-uploading (used for tree thumbnails and the room editor).
    fn texture_static(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        rgba: &[u8],
        w: u32,
        h: u32,
    ) -> egui::TextureId {
        if self.thumbs_ready.contains(key) {
            return self
                .textures
                .get(key)
                .map(|t| t.id())
                .unwrap_or(egui::TextureId::default());
        }
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba);
        let tex = ctx.load_texture(key, image, egui::TextureOptions::NEAREST);
        let id = tex.id();
        self.textures.insert(key.to_string(), tex);
        self.thumbs_ready.insert(key.to_string());
        id
    }

    /// Thumbnail (frame 0) of a sprite: (texture, width, height).
    fn sprite_thumb(
        &mut self,
        ctx: &egui::Context,
        spr: usize,
    ) -> Option<(egui::TextureId, f32, f32)> {
        if spr >= self.project.sprites.len() || self.project.sprites[spr].frames.is_empty() {
            return None;
        }
        let (w, h) = (
            self.project.sprites[spr].width,
            self.project.sprites[spr].height,
        );
        if w == 0 || h == 0 {
            return None;
        }
        let data = self.frames(spr).first().cloned().unwrap_or_default();
        let tex = self.texture_static(ctx, &format!("thumb_spr_{}", spr), &data, w, h);
        Some((tex, w as f32, h as f32))
    }

    /// Thumbnail of an object (its sprite, frame 0).
    fn object_thumb(
        &mut self,
        ctx: &egui::Context,
        obj: usize,
    ) -> Option<(egui::TextureId, f32, f32)> {
        let spr = self.project.objects.get(obj)?.sprite_index;
        if spr < 0 {
            return None;
        }
        self.sprite_thumb(ctx, spr as usize)
    }

    fn comp(&mut self, key: &str) -> &mut CompletionState {
        self.completion.entry(key.to_string()).or_default()
    }

    // -------- actions --------

    fn do_open(&mut self, path: PathBuf) {
        match load(&path) {
            Ok(p) => {
                self.project = p;
                self.path = Some(path.clone());
                self.frames_cache.clear();
                self.invalidate_thumbs();
                self.sel = None;
                self.completion.clear();
                self.log(OK, format!("opened {}", path.display()));
            }
            Err(e) => self.log(ERR, format!("open failed: {}", e)),
        }
    }

    fn flush_caches(&mut self) {
        let idxs: Vec<usize> = self.frames_cache.keys().copied().collect();
        for i in idxs {
            self.sync_frames(i);
        }
    }

    fn do_save(&mut self, ask_path: bool) {
        let path = if ask_path || self.path.is_none() {
            match rfd::FileDialog::new()
                .add_filter("RGM project", &["gmproj"])
                .set_file_name(format!("{}.gmproj", self.project.name))
                .save_file()
            {
                Some(p) => {
                    self.path = Some(p.clone());
                    p
                }
                None => return,
            }
        } else {
            self.path.clone().unwrap()
        };
        self.flush_caches();
        match save(&self.project, &path) {
            Ok(()) => self.log(OK, format!("saved {}", path.display())),
            Err(e) => self.log(ERR, format!("save failed: {}", e)),
        }
    }

    fn do_check(&mut self) {
        let mut errors = 0;
        let mut msgs: Vec<String> = Vec::new();
        let check = |what: &str, code: &str, errors: &mut usize, msgs: &mut Vec<String>| {
            if code.trim().is_empty() {
                return;
            }
            if let Err(e) = gml::compile(code) {
                *errors += 1;
                msgs.push(format!("{}: {}", what, e));
            }
        };
        for s in &self.project.scripts {
            check(
                &format!("script {}", s.name),
                &s.code,
                &mut errors,
                &mut msgs,
            );
        }
        for o in &self.project.objects {
            for ev in &o.events {
                check(
                    &format!("object {} / {}", o.name, ev.label(&self.project.objects)),
                    ev.code(),
                    &mut errors,
                    &mut msgs,
                );
            }
        }
        for t in &self.project.timelines {
            for m in &t.moments {
                check(
                    &format!("timeline {} / moment {}", t.name, m.step),
                    &m.code,
                    &mut errors,
                    &mut msgs,
                );
            }
        }
        for r in &self.project.rooms {
            check(
                &format!("room {} creation code", r.name),
                &r.creation_code,
                &mut errors,
                &mut msgs,
            );
        }
        if errors == 0 {
            self.log(OK, "check: no errors");
        } else {
            for m in msgs {
                self.log(ERR, m);
            }
            self.log(ERR, format!("check: {} error(s)", errors));
        }
    }

    fn do_run(&mut self) {
        if let Some(child) = &mut self.child {
            if let Ok(Some(status)) = child.try_wait() {
                self.log(INFO, format!("game exited: {}", status));
                self.child = None;
            } else {
                let _ = child.kill();
                self.child = None;
            }
        }
        if self.path.is_none() {
            self.do_save(true);
            if self.path.is_none() {
                self.log(ERR, "run cancelled: project not saved");
                return;
            }
        }
        self.flush_caches();
        if let Some(p) = &self.path {
            if let Err(e) = save(&self.project, p) {
                self.log(ERR, format!("run: cannot save project: {}", e));
                return;
            }
        }
        let rt = match find_runtime() {
            Some(r) => r,
            None => {
                self.log(
                    ERR,
                    "run: gmruntime binary not found (build it or set RGM_RUNTIME)",
                );
                return;
            }
        };
        let proj = self.path.clone().unwrap();
        match std::process::Command::new(&rt).arg(&proj).spawn() {
            Ok(c) => {
                self.child = Some(c);
                self.log(OK, format!("running {} ({})", proj.display(), rt.display()));
            }
            Err(e) => self.log(ERR, format!("run failed: {}", e)),
        }
    }

    /// Find the cargo workspace root (this IDE usually runs from it, or the
    /// binary lives in <root>/target/<mode>/).
    fn workspace_root() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("RGM_WORKSPACE") {
            let pb = PathBuf::from(p);
            if pb.join("Cargo.toml").exists() {
                return Some(pb);
            }
        }
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe() {
            let mut d = exe.parent().map(|p| p.to_path_buf());
            for _ in 0..3 {
                if let Some(dd) = d.clone() {
                    candidates.push(dd.clone());
                    d = dd.parent().map(|p| p.to_path_buf());
                }
            }
        }
        candidates
            .into_iter()
            .find(|c| c.join("Cargo.toml").exists() && c.join("crates").is_dir())
    }

    /// Start a release export of the current project in the background.
    fn start_export(&mut self) {
        if self.export_job.is_some() {
            self.log(ERR, "export: a build is already running");
            return;
        }
        // the project must be saved (we embed the file into the binary)
        if self.path.is_none() {
            self.do_save(true);
            if self.path.is_none() {
                self.log(ERR, "export cancelled: project not saved");
                return;
            }
        }
        self.flush_caches();
        let Some(proj) = self.path.clone() else {
            return;
        };
        let Some(out_dir) = self.export_dir.clone() else {
            self.log(ERR, "export: choose an output folder first");
            return;
        };
        let Some(ws) = Self::workspace_root() else {
            self.log(ERR, "export: cargo workspace not found (set RGM_WORKSPACE)");
            return;
        };
        let name = sanitize_name(self.export_name.trim()).replace('_', "-");
        let name = if name.is_empty() { "game".into() } else { name };
        let exe_name = format!("{}{}", name, std::env::consts::EXE_SUFFIX);

        let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::<(String, u8)>::new()));
        let finished = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let log_sink = logs.clone();
        let done_flag = finished.clone();

        let exe_name_t = exe_name.clone();
        let out_dir_t = out_dir.clone();
        let handle = std::thread::spawn(move || {
            let exe_name = exe_name_t;
            let out_dir = out_dir_t;
            let push = |line: String, lvl: u8| {
                log_sink.lock().unwrap().push((line, lvl));
            };
            push(
                format!(
                    "export: building release runtime with embedded {}…",
                    proj.display()
                ),
                INFO,
            );
            let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
            let mut cmd = std::process::Command::new(&cargo);
            cmd.args(["build", "--release", "-p", "gmruntime"])
                .env("RGM_EMBED", &proj)
                .current_dir(&ws)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    push(format!("export: cannot run cargo: {}", e), ERR);
                    done_flag.store(true, std::sync::atomic::Ordering::Release);
                    return;
                }
            };
            // stream stdout/stderr
            let out_logs = log_sink.clone();
            if let Some(mut so) = child.stdout.take() {
                std::thread::spawn(move || {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(&mut so);
                    for line in reader.lines().map_while(|l| l.ok()) {
                        out_logs.lock().unwrap().push((line, INFO));
                    }
                });
            }
            let err_logs = log_sink.clone();
            if let Some(mut se) = child.stderr.take() {
                std::thread::spawn(move || {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(&mut se);
                    for line in reader.lines().map_while(|l| l.ok()) {
                        err_logs.lock().unwrap().push((line, INFO));
                    }
                });
            }
            let ok = matches!(child.wait(), Ok(st) if st.success());
            if !ok {
                push("export: build failed".into(), ERR);
                done_flag.store(true, std::sync::atomic::Ordering::Release);
                return;
            }
            // copy the artifact
            let src = ws.join("target/release/gmruntime");
            let src = src.with_extension(std::env::consts::EXE_EXTENSION.trim_start_matches('.'));
            let dst = out_dir.join(&exe_name);
            match std::fs::copy(&src, &dst) {
                Ok(bytes) => push(
                    format!("export: wrote {} ({} KB)", dst.display(), bytes / 1024),
                    OK,
                ),
                Err(e) => {
                    push(format!("export: cannot copy {}: {}", dst.display(), e), ERR);
                    done_flag.store(true, std::sync::atomic::Ordering::Release);
                    return;
                }
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755));
            }
            let readme = format!(
                "{} — an RGM game\n\nRun {} (double-click or from a terminal).\nBuilt from {} on {}.\nEngine: RGM — a Rust Game Maker 8.1 clone.\n",
                name,
                exe_name,
                proj.display(),
                chrono_free_date(),
            );
            let _ = std::fs::write(out_dir.join("README.txt"), readme);
            push(
                "export: done — a standalone game binary with the project embedded".into(),
                OK,
            );
            done_flag.store(true, std::sync::atomic::Ordering::Release);
        });

        self.export_job = Some(ExportJob {
            name: exe_name.clone(),
            out_dir,
            logs,
            finished,
            handle: Some(handle),
        });
    }

    /// Poll the background export: drain its logs, finish when done.
    fn poll_export(&mut self, ctx: &egui::Context) {
        let Some(job) = &mut self.export_job else {
            return;
        };
        {
            let mut got = job.logs.lock().unwrap();
            for (line, lvl) in got.drain(..) {
                self.console.push((line, lvl));
            }
        }
        if job.finished.load(std::sync::atomic::Ordering::Acquire) {
            let job = self.export_job.take().unwrap();
            if let Some(h) = job.handle {
                let _ = h.join();
            }
        } else {
            ctx.request_repaint(); // keep streaming
        }
    }

    fn ensure_audio(&mut self) -> bool {
        if self.audio.is_none() {
            match rodio::OutputStream::try_default() {
                Ok((out, handle)) => {
                    self.audio = Some((out, handle));
                    return true;
                }
                Err(e) => {
                    self.log(ERR, format!("audio unavailable: {}", e));
                    return false;
                }
            }
        }
        true
    }

    fn play_sound_preview(&mut self, idx: usize) {
        if !self.ensure_audio() {
            return;
        }
        let bytes = decode_rgba(&self.project.sounds[idx].data);
        if bytes.is_empty() {
            self.log(ERR, "sound has no data");
            return;
        }
        let handle = self.audio.as_ref().unwrap().1.clone();
        match rodio::Decoder::new(std::io::Cursor::new(bytes)) {
            Ok(dec) => match rodio::Sink::try_new(&handle) {
                Ok(sink) => {
                    sink.append(dec);
                    self.sink = Some(sink);
                }
                Err(e) => self.log(ERR, format!("play failed: {}", e)),
            },
            Err(e) => self.log(ERR, format!("cannot decode sound: {}", e)),
        }
    }

    fn stop_preview(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
    }
}

fn find_runtime() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("RGM_RUNTIME") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Some(p);
        }
    }
    let mut candidates = vec![
        PathBuf::from("target/debug/gmruntime"),
        PathBuf::from("target/release/gmruntime"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for base in ["..", "../..", "../../.."] {
                for mode in ["debug", "release"] {
                    candidates.push(dir.join(base).join("target").join(mode).join("gmruntime"));
                }
            }
        }
    }
    candidates.into_iter().find(|p| p.exists())
}

fn chrono_free_date() -> String {
    // yyyymmdd without external crates
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn sanitize_name(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() {
        out = "unnamed".into();
    }
    if !out.chars().next().unwrap().is_ascii_alphabetic() && !out.starts_with('_') {
        out = format!("_{}", out);
    }
    out
}

// ------------------------------------------------------------------ layout

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let name_db = NameDb::build(&self.project);

        self.handle_shortcuts(ctx);
        self.poll_export(ctx);

        self.menu_and_toolbar(ctx);
        self.status_bar(ctx);
        self.resource_tree(ctx);
        self.console_panel(ctx);
        self.rename_dialog(ctx);
        self.delete_confirm_dialog(ctx);
        self.export_dialog_ui(ctx);
        self.help_window(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(6.0))
            .show(ctx, |ui| match self.sel {
                Some(ResRef::Sprite(i)) if i < self.project.sprites.len() => {
                    self.sprite_editor(ui, i)
                }
                Some(ResRef::Sound(i)) if i < self.project.sounds.len() => self.sound_editor(ui, i),
                Some(ResRef::Background(i)) if i < self.project.backgrounds.len() => {
                    self.background_editor(ui, i, ctx)
                }
                Some(ResRef::Path(i)) if i < self.project.paths.len() => self.path_editor(ui, i),
                Some(ResRef::Script(i)) if i < self.project.scripts.len() => {
                    self.script_editor(ui, i, &name_db)
                }
                Some(ResRef::Timeline(i)) if i < self.project.timelines.len() => {
                    self.timeline_editor(ui, i, &name_db)
                }
                Some(ResRef::Font(i)) if i < self.project.fonts.len() => self.font_editor(ui, i),
                Some(ResRef::Object(i)) if i < self.project.objects.len() => {
                    self.object_editor(ui, i, &name_db)
                }
                Some(ResRef::Room(i)) if i < self.project.rooms.len() => {
                    self.room_editor(ui, i, ctx)
                }
                _ => {
                    ui.heading("Welcome to RGM");
                    ui.label("Create resources in the tree on the left (use the + buttons).");
                    ui.label(
                        "Workflow: Sprites -> Objects (GML events) -> Rooms. Press F5 to run.",
                    );
                }
            });

        self.add_event_dialog(ctx);
        self.dnd_params_dialog(ctx);
    }
}

impl App {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // F5 / Ctrl+S / Ctrl+O / F7
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.do_run();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.do_save(false);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("RGM project", &["gmproj"])
                .pick_file()
            {
                self.do_open(p);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F7)) {
            self.do_check();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) && !ctx.wants_keyboard_input() {
            self.help_open = true;
        }
        // Delete priority: D&D action → event → resource (with confirmation).
        // Ignored while typing in a text editor.
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) && !ctx.wants_keyboard_input() {
            if !self.try_delete_action()
                && !self.try_delete_event()
                && self.confirm_delete.is_none()
            {
                if let Some(sel) = self.sel {
                    self.confirm_delete = Some(sel);
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) && !ctx.wants_keyboard_input() {
            if self.rename.is_none() {
                if let Some(sel) = self.sel {
                    let name = self.res_name(sel);
                    self.rename = Some((sel, name));
                }
            }
        }
    }

    /// Delete the selected D&D action (object editor with an Actions event).
    fn try_delete_action(&mut self) -> bool {
        let Some(ai) = self.dnd_sel_action else {
            return false;
        };
        let Some(ResRef::Object(oi)) = self.sel else {
            return false;
        };
        let ev = *self.sel_event.entry(oi).or_insert(0);
        let Some(obj) = self.project.objects.get_mut(oi) else {
            return false;
        };
        let Some(EventDef::Actions { actions, .. }) = obj.events.get_mut(ev) else {
            return false;
        };
        if ai < actions.len() {
            actions.remove(ai);
            self.dnd_sel_action = None;
            true
        } else {
            self.dnd_sel_action = None;
            false
        }
    }

    /// Delete the selected event of the selected object.
    fn try_delete_event(&mut self) -> bool {
        let Some(ResRef::Object(oi)) = self.sel else {
            return false;
        };
        let ev = *self.sel_event.entry(oi).or_insert(0);
        let Some(obj) = self.project.objects.get_mut(oi) else {
            return false;
        };
        if ev < obj.events.len() {
            obj.events.remove(ev);
            self.dnd_sel_action = None;
            self.dnd_open_for = None;
            if ev >= obj.events.len() {
                self.sel_event
                    .insert(oi, obj.events.len().saturating_sub(1));
            }
            true
        } else {
            false
        }
    }

    fn res_name(&self, res: ResRef) -> String {
        match res {
            ResRef::Sprite(i) => self
                .project
                .sprites
                .get(i)
                .map(|s| s.name.clone())
                .unwrap_or_default(),
            ResRef::Sound(i) => self
                .project
                .sounds
                .get(i)
                .map(|s| s.name.clone())
                .unwrap_or_default(),
            ResRef::Background(i) => self
                .project
                .backgrounds
                .get(i)
                .map(|b| b.name.clone())
                .unwrap_or_default(),
            ResRef::Path(i) => self
                .project
                .paths
                .get(i)
                .map(|p| p.name.clone())
                .unwrap_or_default(),
            ResRef::Script(i) => self
                .project
                .scripts
                .get(i)
                .map(|s| s.name.clone())
                .unwrap_or_default(),
            ResRef::Timeline(i) => self
                .project
                .timelines
                .get(i)
                .map(|t| t.name.clone())
                .unwrap_or_default(),
            ResRef::Font(i) => self
                .project
                .fonts
                .get(i)
                .map(|f| f.name.clone())
                .unwrap_or_default(),
            ResRef::Object(i) => self
                .project
                .objects
                .get(i)
                .map(|o| o.name.clone())
                .unwrap_or_default(),
            ResRef::Room(i) => self
                .project
                .rooms
                .get(i)
                .map(|r| r.name.clone())
                .unwrap_or_default(),
        }
    }

    fn menu_and_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New project").clicked() {
                        self.project = Project::default();
                        self.path = None;
                        self.frames_cache.clear();
                        self.sel = None;
                        self.completion.clear();
                        self.log(OK, "new project");
                        ui.close_menu();
                    }
                    if ui.button("Open…  (Ctrl+O)").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("RGM project", &["gmproj"])
                            .pick_file()
                        {
                            self.do_open(p);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save  (Ctrl+S)").clicked() {
                        self.do_save(false);
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        self.do_save(true);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export game…").clicked() {
                        self.export_name = sanitize_name(&self.project.name).replace('_', "-");
                        self.export_dialog = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Rename selected…").clicked() {
                        if let Some(sel) = self.sel {
                            let name = match sel {
                                ResRef::Sprite(i) => self.project.sprites[i].name.clone(),
                                ResRef::Sound(i) => self.project.sounds[i].name.clone(),
                                ResRef::Background(i) => self.project.backgrounds[i].name.clone(),
                                ResRef::Path(i) => self.project.paths[i].name.clone(),
                                ResRef::Script(i) => self.project.scripts[i].name.clone(),
                                ResRef::Timeline(i) => self.project.timelines[i].name.clone(),
                                ResRef::Font(i) => self.project.fonts[i].name.clone(),
                                ResRef::Object(i) => self.project.objects[i].name.clone(),
                                ResRef::Room(i) => self.project.rooms[i].name.clone(),
                            };
                            self.rename = Some((sel, name));
                        }
                        ui.close_menu();
                    }
                    if ui.button("Delete selected").clicked() {
                        if let Some(sel) = self.sel {
                            self.delete_resource(sel);
                        }
                        ui.close_menu();
                    }
                });
                ui.menu_button("Resources", |ui| {
                    for kind in [
                        AddKind::Sprite,
                        AddKind::Sound,
                        AddKind::Background,
                        AddKind::Path,
                        AddKind::Script,
                        AddKind::Timeline,
                        AddKind::Font,
                        AddKind::Object,
                        AddKind::Room,
                    ] {
                        if ui.button(format!("New {}…", kind.title())).clicked() {
                            self.add_resource(kind);
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("GML reference  (F1)").clicked() {
                        self.help_open = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Run", |ui| {
                    if ui.button("Run game  (F5)").clicked() {
                        self.do_run();
                        ui.close_menu();
                    }
                    if ui.button("Check code  (F7)").clicked() {
                        self.do_check();
                        ui.close_menu();
                    }
                });
            });
        });
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(6.0, 2.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if icon_button(ui, Icon::New, false) {
                        self.project = Project::default();
                        self.path = None;
                        self.frames_cache.clear();
                        self.sel = None;
                        self.log(OK, "new project");
                    }
                    if icon_button(ui, Icon::Open, false) {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("RGM project", &["gmproj"])
                            .pick_file()
                        {
                            self.do_open(p);
                        }
                    }
                    if icon_button(ui, Icon::Save, false) {
                        self.do_save(false);
                    }
                    ui.separator();
                    if icon_button(ui, Icon::Check, false) {
                        self.do_check();
                    }
                    if icon_button(ui, Icon::Run, false) {
                        self.do_run();
                    }
                    ui.separator();
                    ui.label("Game name:");
                    let mut name = self.project.name.clone();
                    let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(140.0));
                    if r.changed() {
                        self.project.name = name;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} sprites | {} objects | {} rooms",
                                self.project.sprites.len(),
                                self.project.objects.len(),
                                self.project.rooms.len()
                            ))
                            .small()
                            .weak(),
                        );
                    });
                });
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL_BG)
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let sel_txt = match self.sel {
                        None => None,
                        Some(ResRef::Sprite(i)) => self
                            .project
                            .sprites
                            .get(i)
                            .map(|s| format!("Sprite: {}", s.name)),
                        Some(ResRef::Sound(i)) => self
                            .project
                            .sounds
                            .get(i)
                            .map(|s| format!("Sound: {}", s.name)),
                        Some(ResRef::Background(i)) => self
                            .project
                            .backgrounds
                            .get(i)
                            .map(|b| format!("Background: {}", b.name)),
                        Some(ResRef::Path(i)) => self
                            .project
                            .paths
                            .get(i)
                            .map(|p| format!("Path: {}", p.name)),
                        Some(ResRef::Script(i)) => self
                            .project
                            .scripts
                            .get(i)
                            .map(|s| format!("Script: {}", s.name)),
                        Some(ResRef::Timeline(i)) => self
                            .project
                            .timelines
                            .get(i)
                            .map(|t| format!("Timeline: {}", t.name)),
                        Some(ResRef::Font(i)) => self
                            .project
                            .fonts
                            .get(i)
                            .map(|f| format!("Font: {}", f.name)),
                        Some(ResRef::Object(i)) => self
                            .project
                            .objects
                            .get(i)
                            .map(|o| format!("Object: {}", o.name)),
                        Some(ResRef::Room(i)) => self
                            .project
                            .rooms
                            .get(i)
                            .map(|r| format!("Room: {}", r.name)),
                    }
                    .unwrap_or_else(|| "nothing selected".to_string());
                    ui.label(egui::RichText::new(sel_txt).small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let saved = self
                            .path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "unsaved".into());
                        ui.label(egui::RichText::new(saved).small().weak());
                    });
                });
            });
    }

    fn console_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("console")
            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(4.0))
            .default_height(110.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Output");
                    if ui.small_button("Clear").clicked() {
                        self.console.clear();
                    }
                });
                theme::sunken_frame().show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("console_scroll")
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for (line, level) in &self.console {
                                let color = match *level {
                                    crate::ERR => egui::Color32::from_rgb(0xB0, 0x20, 0x20),
                                    crate::OK => egui::Color32::from_rgb(0x1E, 0x7A, 0x1E),
                                    _ => egui::Color32::from_rgb(0x33, 0x33, 0x33),
                                };
                                ui.monospace(egui::RichText::new(line).small().color(color));
                            }
                        });
                });
            });
    }

    fn resource_tree(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("resources")
            .frame(egui::Frame::none().fill(theme::PANEL_BG).inner_margin(4.0))
            .resizable(true)
            .default_width(215.0)
            .show(ctx, |ui| {
                theme::sunken_frame().show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let sections: Vec<(&str, Vec<String>, fn(usize) -> ResRef, AddKind)> = vec![
                            (
                                "Sprites",
                                self.project
                                    .sprites
                                    .iter()
                                    .map(|s| s.name.clone())
                                    .collect(),
                                ResRef::Sprite,
                                AddKind::Sprite,
                            ),
                            (
                                "Sounds",
                                self.project.sounds.iter().map(|s| s.name.clone()).collect(),
                                ResRef::Sound,
                                AddKind::Sound,
                            ),
                            (
                                "Backgrounds",
                                self.project
                                    .backgrounds
                                    .iter()
                                    .map(|b| b.name.clone())
                                    .collect(),
                                ResRef::Background,
                                AddKind::Background,
                            ),
                            (
                                "Paths",
                                self.project.paths.iter().map(|p| p.name.clone()).collect(),
                                ResRef::Path,
                                AddKind::Path,
                            ),
                            (
                                "Scripts",
                                self.project
                                    .scripts
                                    .iter()
                                    .map(|s| s.name.clone())
                                    .collect(),
                                ResRef::Script,
                                AddKind::Script,
                            ),
                            (
                                "Timelines",
                                self.project
                                    .timelines
                                    .iter()
                                    .map(|t| t.name.clone())
                                    .collect(),
                                ResRef::Timeline,
                                AddKind::Timeline,
                            ),
                            (
                                "Fonts",
                                self.project.fonts.iter().map(|f| f.name.clone()).collect(),
                                ResRef::Font,
                                AddKind::Font,
                            ),
                            (
                                "Objects",
                                self.project
                                    .objects
                                    .iter()
                                    .map(|o| o.name.clone())
                                    .collect(),
                                ResRef::Object,
                                AddKind::Object,
                            ),
                            (
                                "Rooms",
                                self.project.rooms.iter().map(|r| r.name.clone()).collect(),
                                ResRef::Room,
                                AddKind::Room,
                            ),
                        ];
                        for (title, names, make, kind) in sections {
                            self.tree_section(ui, ctx, title, &names, make, kind);
                        }
                    });
                });
            });
    }

    fn tree_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        title: &str,
        names: &[String],
        make: fn(usize) -> ResRef,
        kind: AddKind,
    ) {
        let mut action = false;
        let mut del: Option<ResRef> = None;
        let id = egui::Id::new(format!("cat_{}", title));
        let state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
        state
            .show_header(ui, |ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                Icon::Folder.draw(ui.painter(), r);
                ui.strong(format!("{} ({})", title, names.len()));
            })
            .body(|ui| {
                ui.indent(id.with("rows"), |ui| {
                    for (i, name) in names.iter().enumerate() {
                        let res = make(i);
                        let is_sel = self.sel == Some(res);
                        let thumb = match kind {
                            AddKind::Sprite => self.sprite_thumb(ctx, i),
                            AddKind::Object => self.object_thumb(ctx, i),
                            _ => None,
                        };
                        let row = match thumb {
                            Some((tex, tw, th)) => {
                                crate::icons::thumb_and_text(ui, tex, (tw, th), name, is_sel, None)
                            }
                            None if kind == AddKind::Object => crate::icons::thumb_and_text(
                                ui,
                                egui::TextureId::default(),
                                (0.0, 0.0),
                                name,
                                is_sel,
                                Some(crate::object_color(i)),
                            ),
                            None => icon_and_text(ui, kind.icon(), name, is_sel),
                        };
                        if row.clicked() {
                            self.sel = Some(res);
                            if let ResRef::Sprite(_) = res {
                                self.spr_frame = 0;
                            }
                        }
                        // right-click menu: rename / delete
                        row.context_menu(|ui| {
                            ui.label(egui::RichText::new(name).strong());
                            ui.separator();
                            if ui.button("Rename…  (F2)").clicked() {
                                self.rename = Some((res, name.clone()));
                                ui.close_menu();
                            }
                            if ui.button("Delete…  (Del)").clicked() {
                                del = Some(res);
                                ui.close_menu();
                            }
                        });
                    }
                    if ui.button("+ add").clicked() {
                        action = true;
                    }
                });
            });
        if action {
            self.add_resource(kind);
        }
        if let Some(res) = del {
            self.delete_resource(res);
        }
    }

    fn export_dialog_ui(&mut self, ctx: &egui::Context) {
        if !self.export_dialog {
            return;
        }
        let busy = self.export_job.is_some();
        let default_name = sanitize_name(&self.project.name).replace('_', "-");
        self.export_dialog_ui_inner(ctx, &default_name, busy);
    }

    fn export_dialog_ui_inner(&mut self, ctx: &egui::Context, default_name: &str, busy: bool) {
        let mut open = self.export_dialog;
        egui::Window::new("Export game")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Builds a standalone release binary with the project");
                ui.label("embedded inside, for the current operating system.");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Game name:");
                    let r = ui.add(egui::TextEdit::singleline(&mut self.export_name).desired_width(160.0));
                    if self.export_name.is_empty() {
                        self.export_name = default_name.to_string();
                        let _ = r;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Output folder:");
                    let dir_txt = self
                        .export_dir
                        .as_ref()
                        .map(|d| d.display().to_string())
                        .unwrap_or_else(|| "<not chosen>".into());
                    ui.monospace(egui::RichText::new(dir_txt).small());
                    if ui.button("Browse…").clicked() {
                        if let Some(d) = rfd::FileDialog::new().pick_folder() {
                            self.export_dir = Some(d);
                        }
                    }
                });
                ui.add_space(6.0);
                if busy {
                    ui.label(egui::RichText::new("building… (see Output)").color(egui::Color32::from_rgb(0x1E, 0x5E, 0xC0)));
                } else {
                    let ready = self.export_dir.is_some() && !self.export_name.trim().is_empty();
                    let btn = ui.add_enabled(ready, egui::Button::new("Build"));
                    if btn.clicked() {
                        let dir = self.export_dialog = false;
                        let _ = dir;
                        self.start_export();
                    }
                }
                ui.add_space(4.0);
                ui.weak(
                    egui::RichText::new(
                        "Cross-platform builds: run the CI workflow (.github/workflows/builds.yml) \\
                        or build on the target OS with:  RGM_EMBED=game.gmproj cargo build --release -p gmruntime",
                    )
                    .small(),
                );
            });
        if !open {
            self.export_dialog = false;
        }
    }

    fn delete_confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(res) = self.confirm_delete else {
            return;
        };
        let name = self.res_name(res);
        let kind = match res {
            ResRef::Sprite(_) => "sprite",
            ResRef::Sound(_) => "sound",
            ResRef::Background(_) => "background",
            ResRef::Path(_) => "path",
            ResRef::Script(_) => "script",
            ResRef::Timeline(_) => "timeline",
            ResRef::Font(_) => "font",
            ResRef::Object(_) => "object",
            ResRef::Room(_) => "room",
        };
        let mut do_delete = false;
        egui::Window::new("Delete resource")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Delete {} \"{}\"?", kind, name));
                ui.label(egui::RichText::new("This cannot be undone.").weak().small());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let del_btn = ui.button(
                        egui::RichText::new("Delete")
                            .color(egui::Color32::from_rgb(0xB0, 0x20, 0x20)),
                    );
                    let enter_ok = ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !ui.ctx().wants_keyboard_input();
                    if del_btn.clicked() || enter_ok {
                        do_delete = true;
                    }
                    if ui.button("Cancel").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        self.confirm_delete = None;
                    }
                });
            });
        if do_delete {
            self.delete_resource(res);
            self.confirm_delete = None;
            self.log(0, format!("deleted {} \"{}\"", kind, name));
        }
    }

    fn rename_dialog(&mut self, ctx: &egui::Context) {
        if let Some((res, mut buf)) = self.rename.clone() {
            egui::Window::new("Rename")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("New name (letters, digits, _):");
                    let r = ui.add(egui::TextEdit::singleline(&mut buf));
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.rename = None;
                        return;
                    }
                    ui.horizontal(|ui| {
                        let ok = ui.button("OK");
                        if (ok.clicked()
                            || (r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                            && !buf.is_empty()
                        {
                            let clean = sanitize_name(&buf);
                            match res {
                                ResRef::Sprite(i) => self.project.sprites[i].name = clean,
                                ResRef::Sound(i) => self.project.sounds[i].name = clean,
                                ResRef::Background(i) => self.project.backgrounds[i].name = clean,
                                ResRef::Path(i) => self.project.paths[i].name = clean,
                                ResRef::Script(i) => self.project.scripts[i].name = clean,
                                ResRef::Timeline(i) => self.project.timelines[i].name = clean,
                                ResRef::Font(i) => self.project.fonts[i].name = clean,
                                ResRef::Object(i) => self.project.objects[i].name = clean,
                                ResRef::Room(i) => self.project.rooms[i].name = clean,
                            }
                            self.rename = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.rename = None;
                        }
                    });
                });
            if self.rename.is_some() {
                self.rename = Some((res, buf));
            }
        }
    }
}

fn object_color(idx: usize) -> egui::Color32 {
    let palette = [
        (231, 76, 60),
        (46, 204, 113),
        (52, 152, 219),
        (241, 196, 15),
        (155, 89, 182),
        (230, 126, 34),
        (26, 188, 156),
        (242, 120, 172),
    ];
    let (r, g, b) = palette[idx % palette.len()];
    egui::Color32::from_rgb(r, g, b)
}

/// Pinch (trackpad) / wheel zoom applied while the pointer hovers over a canvas.
/// `wheel` = plain mouse wheel also zooms (for canvases without their own scrolling).
pub(crate) fn canvas_zoom(
    ui: &egui::Ui,
    rect: egui::Rect,
    zoom: &mut f32,
    min: f32,
    max: f32,
    wheel: bool,
) {
    let Some(pos) = ui.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    if !rect.contains(pos) {
        return;
    }
    // covers macOS trackpad pinch and ctrl/cmd + wheel on all platforms
    let zd = ui.input(|i| i.zoom_delta());
    if (zd - 1.0).abs() > 1e-6 {
        *zoom = (*zoom * zd).clamp(min, max);
    }
    if wheel {
        let sd = ui.input(|i| i.smooth_scroll_delta).y;
        if sd != 0.0 {
            *zoom = (*zoom * (1.0 + sd / 250.0)).clamp(min, max);
        }
    }
}

pub(crate) fn import_image(path: &std::path::Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((w, h, rgba.into_raw()))
}

// ------------------------------------------------------------------ main

fn main() -> eframe::Result<()> {
    // open project passed on the command line, if any
    let arg_project = std::env::args().nth(1);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_position([60.0, 40.0])
            .with_title("RGM IDE — Game Maker 8.1 clone"),
        ..Default::default()
    };
    eframe::run_native(
        "RGM IDE",
        options,
        Box::new(|cc| {
            let mut app = App::new(cc);
            if let Some(p) = &arg_project {
                app.do_open(std::path::PathBuf::from(p));
            }
            // test hook: "execute code" D&D action with the params dialog open
            if std::env::var("RGM_CODE_ACTION_TEST").is_ok() {
                if let Some(o) = app.project.objects.get_mut(7) {
                    o.events.push(EventDef::Actions {
                        target: Box::new(EventTarget::Step),
                        actions: vec![{
                            let mut a = gmformat::DndAction::new(4, 401);
                            a.params = vec![gmformat::DndParam::Str(
                                "// move right slowly\nif (x < 100) {\n    x += 4;\n}".into(),
                            )];
                            a
                        }],
                    });
                    app.sel = Some(ResRef::Object(7));
                    app.sel_event.insert(7, o.events.len() - 1);
                    app.dnd_open_for = Some((7, o.events.len() - 1));
                    app.dnd.edit = Some(0);
                    app.dnd.edit_buf = match &o.events[o.events.len() - 1] {
                        EventDef::Actions { actions, .. } => actions[0].clone(),
                        _ => unreachable!(),
                    };
                }
            }
            // test hook: open the Help window
            if std::env::var("RGM_HELP_TEST").is_ok() {
                app.help_open = true;
            }
            // test hook: open the Add Event dialog
            if std::env::var("RGM_ADD_DND_TEST").is_ok() {
                app.add_event_for = Some(6);
            }
            // test hook: open the params dialog of a "start moving in a direction" action
            if std::env::var("RGM_MOVE_TEST").is_ok() {
                if let Some(o) = app.project.objects.get_mut(7) {
                    o.events.push(EventDef::Actions {
                        target: Box::new(EventTarget::Create),
                        actions: vec![{
                            let mut a = gmformat::DndAction::new(1, 121);
                            a.params = vec![
                                gmformat::DndParam::Real(32.0),
                                gmformat::DndParam::Real(4.0),
                            ];
                            a
                        }],
                    });
                    app.sel = Some(ResRef::Object(7));
                    app.sel_event.insert(7, o.events.len() - 1);
                    app.dnd_open_for = Some((7, o.events.len() - 1));
                    app.dnd.edit = Some(0);
                    app.dnd.edit_buf = match &o.events[o.events.len() - 1] {
                        EventDef::Actions { actions, .. } => actions[0].clone(),
                        _ => unreachable!(),
                    };
                }
            }
            // test hook: inject a D&D event into object 7 and select it
            if std::env::var("RGM_DND_TEST").is_ok() {
                if let Some(o) = app.project.objects.get_mut(7) {
                    o.events.push(EventDef::Actions {
                        target: Box::new(EventTarget::Create),
                        actions: vec![
                            {
                                let mut q = gmformat::DndAction::new(4, 405);
                                q.params = vec![gmformat::DndParam::Str("speed == 0".into())];
                                q
                            },
                            {
                                let mut b = gmformat::DndAction::new(4, 420);
                                b.kind = 1;
                                b
                            },
                            {
                                let mut mv = gmformat::DndAction::new(1, 122);
                                mv.params = vec![
                                    gmformat::DndParam::Real(0.0),
                                    gmformat::DndParam::Real(3.0),
                                ];
                                mv
                            },
                            {
                                let mut e = gmformat::DndAction::new(4, 421);
                                e.kind = 2;
                                e
                            },
                            {
                                let mut el = gmformat::DndAction::new(4, 422);
                                el.kind = 3;
                                el
                            },
                            {
                                let mut mv2 = gmformat::DndAction::new(1, 122);
                                mv2.params = vec![
                                    gmformat::DndParam::Real(90.0),
                                    gmformat::DndParam::Real(2.0),
                                ];
                                mv2
                            },
                        ],
                    });
                    app.sel = Some(ResRef::Object(7));
                    app.sel_event.insert(7, o.events.len() - 1);
                }
            }
            // test hook: pre-open the export dialog
            if std::env::var("RGM_EXPORT_TEST").is_ok() {
                app.export_dialog = true;
                app.export_name = "mygame".into();
                app.export_dir = Some(std::env::temp_dir().join("rgm_export_test"));
            }
            // test hook: RGM_SELECT="object:6" / "script:0" / "room:0"
            if let Ok(sel) = std::env::var("RGM_SELECT") {
                let parts: Vec<&str> = sel.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let idx: usize = parts[1].parse().unwrap_or(0);
                    let r = match parts[0] {
                        "sprite" => Some(ResRef::Sprite(idx)),
                        "sound" => Some(ResRef::Sound(idx)),
                        "background" => Some(ResRef::Background(idx)),
                        "path" => Some(ResRef::Path(idx)),
                        "script" => Some(ResRef::Script(idx)),
                        "timeline" => Some(ResRef::Timeline(idx)),
                        "font" => Some(ResRef::Font(idx)),
                        "object" => Some(ResRef::Object(idx)),
                        "room" => Some(ResRef::Room(idx)),
                        _ => None,
                    };
                    app.sel = r;
                    if let Some(ResRef::Object(_)) = r {
                        // default to the second event (Step) if present
                        if let Some(ResRef::Object(oi)) = r {
                            if app
                                .project
                                .objects
                                .get(oi)
                                .map(|o| o.events.len())
                                .unwrap_or(0)
                                > 1
                            {
                                app.sel_event.insert(oi, 1);
                            }
                        }
                    }
                }
            }
            Ok(Box::new(app))
        }),
    )
}

// ------------------------------------------------------------------ add event dialog

impl App {
    fn help_window(&mut self, ctx: &egui::Context) {
        if !self.help_open {
            return;
        }
        let search = self.help_search.to_lowercase();
        let mut open = self.help_open;
        egui::Window::new("GML reference")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(true)
            .default_size([760.0, 620.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.help_search)
                            .desired_width(240.0)
                            .hint_text("function, constant, variable…"),
                    );
                    ui.weak(egui::RichText::new(format!(
                        "{} functions | {} constants",
                        gmengine::names::FUNCTION_NAMES.len(),
                        gmengine::names::CONSTANT_NAMES.len()
                    )).small());
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("gml_help")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // ---- syntax cheat sheet ----
                        egui::CollapsingHeader::new("Syntax basics")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.monospace(egui::RichText::new(
"// comment;  statements end with ;  (or a newline)
if (x < 10) { ... } else { ... }
while (cond) { ... }   do { ... } until (cond)
for (i = 0; i < 10; i += 1) { ... }   repeat (5) { ... }
with (obj_enemy) { hp -= 10; }   // run code on other instances
switch (state) { case 1: ...; break; default: ...; }
var local_var = 5;   global.global_var = 5;   // scopes
instance_destroy();   // called on self by default
exit;   // stop running this event/script"
                                ).small());
                            });

                        // ---- common events reminder ----
                        egui::CollapsingHeader::new("Instance variables (built-in)")
                            .default_open(false)
                            .show(ui, |ui| {
                                const VARS: &[&str] = &[
                                    "x, y — position", "xstart, ystart — spawn position",
                                    "xprevious, yprevious — position last step",
                                    "hspeed, vspeed, speed, direction — motion",
                                    "friction, gravity, gravity_direction",
                                    "sprite_index, image_index, image_speed",
                                    "image_xscale, image_yscale, image_angle, image_alpha, image_blend",
                                    "visible, solid, persistent, depth",
                                    "alarm[0..11]", "id — instance id", "object_index",
                                    "bbox_left/top/right/bottom",
                                    "path_index, path_position, path_speed, path_endaction, path_absolute",
                                    "timeline_index, timeline_running, timeline_speed, timeline_position, timeline_loop",
                                ];
                                for v in VARS {
                                    ui.monospace(egui::RichText::new(*v).small());
                                }
                            });

                        // ---- functions ----
                        egui::CollapsingHeader::new("Functions")
                            .default_open(true)
                            .show(ui, |ui| {
                                for f in gmengine::names::FUNCTION_NAMES {
                                    if !search.is_empty() && !f.contains(&search) {
                                        continue;
                                    }
                                    let hint = gmengine::names::FUNCTION_HINTS
                                        .iter()
                                        .find(|(n, _)| n == f)
                                        .map(|(_, h)| *h)
                                        .unwrap_or("");
                                    ui.horizontal(|ui| {
                                        ui.monospace(
                                            egui::RichText::new(*f)
                                                .strong()
                                                .color(egui::Color32::from_rgb(0x10, 0x50, 0xB0)),
                                        );
                                        if !hint.is_empty() {
                                            ui.monospace(egui::RichText::new(hint).small().weak());
                                        }
                                    });
                                }
                            });

                        // ---- constants ----
                        egui::CollapsingHeader::new("Constants")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for c in gmengine::names::CONSTANT_NAMES {
                                        if !search.is_empty() && !c.contains(&search) {
                                            continue;
                                        }
                                        ui.monospace(
                                            egui::RichText::new(*c)
                                                .small()
                                                .color(egui::Color32::from_rgb(0x80, 0x00, 0x80)),
                                        );
                                    }
                                });
                            });

                        // ---- keywords ----
                        egui::CollapsingHeader::new("Keywords")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for k in gml::KEYWORDS {
                                        if !search.is_empty() && !k.contains(&search) {
                                            continue;
                                        }
                                        ui.monospace(
                                            egui::RichText::new(*k)
                                                .small()
                                                .color(egui::Color32::from_rgb(0x00, 0x00, 0xC0)),
                                        );
                                    }
                                });
                            });

                        ui.separator();
                        ui.weak(egui::RichText::new(
                            "Scripts are called by name: my_script(arg0, …). Values return from the last executed expression. Full docs: README.md",
                        ).small());
                    });
        });
        self.help_open = open;
    }

    fn dnd_params_dialog(&mut self, ctx: &egui::Context) {
        let Some((obj, ev)) = self.dnd_open_for else {
            return;
        };
        let Some(edit_idx) = self.dnd.edit else {
            return;
        };
        if obj >= self.project.objects.len() || ev >= self.project.objects[obj].events.len() {
            self.dnd.edit = None;
            return;
        }
        // take the list out to avoid double borrows, run the dialog, write back
        let mut actions = match &mut self.project.objects[obj].events[ev] {
            EventDef::Actions { actions, .. } => std::mem::take(actions),
            _ => return,
        };
        if edit_idx >= actions.len() {
            self.dnd.edit = None;
        } else {
            let db = NameDb::build(&self.project);
            let project_snapshot = self.project.clone();
            crate::dndui::params_dialog(ctx, &project_snapshot, &mut self.dnd, &mut actions, &db);
        }
        if let EventDef::Actions { actions: dst, .. } = &mut self.project.objects[obj].events[ev] {
            *dst = actions;
        }
    }

    fn add_event_dialog(&mut self, ctx: &egui::Context) {
        let Some(obj) = self.add_event_for else {
            return;
        };
        if obj >= self.project.objects.len() {
            self.add_event_for = None;
            return;
        }
        let mut close = false;
        egui::Window::new("Add event")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .default_size([340.0, 460.0])
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("new events contain drag & drop actions; add a \"Execute a piece of code\" action for GML")
                        .small()
                        .weak(),
                );
                ui.separator();
                let objects: Vec<String> = self
                    .project
                    .objects
                    .iter()
                    .map(|o| o.name.clone())
                    .collect();
                let add = |def: EventDef, app: &mut App| {
                    let def = {
                        match &def {
                            EventDef::Create { .. } => dnd_actions_def(EventTarget::Create),
                            EventDef::Destroy { .. } => dnd_actions_def(EventTarget::Destroy),
                            EventDef::Alarm { n, .. } => {
                                dnd_actions_def(EventTarget::Alarm { n: *n })
                            }
                            EventDef::Step { .. } => dnd_actions_def(EventTarget::Step),
                            EventDef::BeginStep { .. } => dnd_actions_def(EventTarget::BeginStep),
                            EventDef::EndStep { .. } => dnd_actions_def(EventTarget::EndStep),
                            EventDef::Collision { object, .. } => {
                                dnd_actions_def(EventTarget::Collision { object: *object })
                            }
                            EventDef::Keyboard { key, .. } => {
                                dnd_actions_def(EventTarget::Keyboard { key: *key })
                            }
                            EventDef::KeyPress { key, .. } => {
                                dnd_actions_def(EventTarget::KeyPress { key: *key })
                            }
                            EventDef::KeyRelease { key, .. } => {
                                dnd_actions_def(EventTarget::KeyRelease { key: *key })
                            }
                            EventDef::Mouse { kind, .. } => {
                                dnd_actions_def(EventTarget::Mouse { kind: kind.clone() })
                            }
                            EventDef::Draw { .. } => dnd_actions_def(EventTarget::Draw),
                            EventDef::Other { kind, .. } => {
                                dnd_actions_def(EventTarget::Other { kind: kind.clone() })
                            }
                            a => a.clone(),
                        }
                    };
                    if !app.project.objects[obj].events.iter().any(|e| e == &def) {
                        let e = app.sel_event.entry(obj).or_insert(0);
                        app.project.objects[obj].events.push(def);
                        *e = app.project.objects[obj].events.len() - 1;
                    }
                };
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.strong("Common");
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Create").clicked() {
                            add(
                                EventDef::Create {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                        if ui.button("Destroy").clicked() {
                            add(
                                EventDef::Destroy {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                        if ui.button("Draw").clicked() {
                            add(
                                EventDef::Draw {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                    });
                    ui.strong("Alarm");
                    ui.horizontal_wrapped(|ui| {
                        for n in 0..8u8 {
                            if ui.small_button(format!("{}", n)).clicked() {
                                add(
                                    EventDef::Alarm {
                                        n,
                                        code: String::new(),
                                    },
                                    self,
                                );
                                close = true;
                            }
                        }
                    });
                    ui.strong("Step");
                    ui.horizontal(|ui| {
                        if ui.button("Normal").clicked() {
                            add(
                                EventDef::Step {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                        if ui.button("Begin").clicked() {
                            add(
                                EventDef::BeginStep {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                        if ui.button("End").clicked() {
                            add(
                                EventDef::EndStep {
                                    code: String::new(),
                                },
                                self,
                            );
                            close = true;
                        }
                    });
                    ui.strong("Collision with object");
                    ui.horizontal_wrapped(|ui| {
                        for (oi, oname) in objects.iter().enumerate() {
                            if ui.small_button(oname).clicked() {
                                add(
                                    EventDef::Collision {
                                        object: oi,
                                        code: String::new(),
                                    },
                                    self,
                                );
                                close = true;
                            }
                        }
                    });
                    ui.strong("Keyboard");
                    let mut picked: Option<(i64, u8)> = None;
                    keyboard_events(ui, |key, mode| picked = Some((key, mode)));
                    if let Some((key, mode)) = picked {
                        match mode {
                            0 => add(
                                EventDef::Keyboard {
                                    key,
                                    code: String::new(),
                                },
                                self,
                            ),
                            1 => add(
                                EventDef::KeyPress {
                                    key,
                                    code: String::new(),
                                },
                                self,
                            ),
                            _ => add(
                                EventDef::KeyRelease {
                                    key,
                                    code: String::new(),
                                },
                                self,
                            ),
                        }
                        close = true;
                    }
                    ui.strong("Mouse");
                    ui.horizontal_wrapped(|ui| {
                        for (kind, label) in [
                            ("lmb_down", "LMB"),
                            ("lmb_pressed", "LMB pressed"),
                            ("lmb_released", "LMB released"),
                            ("rmb_down", "RMB"),
                            ("rmb_pressed", "RMB pressed"),
                            ("rmb_released", "RMB released"),
                            ("mmb_down", "MMB"),
                            ("glmb_pressed", "Global LMB press"),
                            ("glmb_released", "Global LMB release"),
                            ("wheel_up", "Wheel up"),
                            ("wheel_down", "Wheel down"),
                            ("enter", "Mouse enter"),
                            ("leave", "Mouse leave"),
                            ("no_button", "No button"),
                        ] {
                            if ui.small_button(label).clicked() {
                                add(
                                    EventDef::Mouse {
                                        kind: kind.into(),
                                        code: String::new(),
                                    },
                                    self,
                                );
                                close = true;
                            }
                        }
                    });
                    ui.strong("Other");
                    ui.horizontal_wrapped(|ui| {
                        for (kind, label) in [
                            ("animation_end", "Animation end"),
                            ("outside_room", "Outside room"),
                            ("intersect_boundary", "Boundary"),
                            ("game_start", "Game start"),
                            ("game_end", "Game end"),
                            ("room_start", "Room start"),
                            ("room_end", "Room end"),
                            ("no_more_lives", "No more lives"),
                            ("no_more_health", "No more health"),
                        ] {
                            if ui.small_button(label).clicked() {
                                add(
                                    EventDef::Other {
                                        kind: kind.into(),
                                        code: String::new(),
                                    },
                                    self,
                                );
                                close = true;
                            }
                        }
                    });
                });
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            self.add_event_for = None;
        }
    }
}

fn dnd_actions_def(target: EventTarget) -> EventDef {
    EventDef::Actions {
        target: Box::new(target),
        actions: vec![],
    }
}

fn keyboard_events(ui: &mut egui::Ui, mut on_pick: impl FnMut(i64, u8)) {
    let keys: [(i64, &str); 26] = [
        (65, "A"),
        (66, "B"),
        (67, "C"),
        (68, "D"),
        (69, "E"),
        (70, "F"),
        (71, "G"),
        (72, "H"),
        (73, "I"),
        (74, "J"),
        (75, "K"),
        (76, "L"),
        (77, "M"),
        (78, "N"),
        (79, "O"),
        (80, "P"),
        (81, "Q"),
        (82, "R"),
        (83, "S"),
        (84, "T"),
        (85, "U"),
        (86, "V"),
        (87, "W"),
        (88, "X"),
        (89, "Y"),
        (90, "Z"),
    ];
    egui::CollapsingHeader::new("letters (held / press / release)").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (vk, name) in keys {
                ui.menu_button(name, |ui| {
                    if ui.button("held").clicked() {
                        on_pick(vk, 0);
                    }
                    if ui.button("press").clicked() {
                        on_pick(vk, 1);
                    }
                    if ui.button("release").clicked() {
                        on_pick(vk, 2);
                    }
                });
            }
        });
    });
    egui::CollapsingHeader::new("specials (held / press / release)").show(ui, |ui| {
        let specials: [(i64, &str); 22] = [
            (32, "Space"),
            (13, "Enter"),
            (27, "Escape"),
            (8, "Backspace"),
            (9, "Tab"),
            (37, "Left"),
            (38, "Up"),
            (39, "Right"),
            (40, "Down"),
            (16, "Shift"),
            (17, "Control"),
            (18, "Alt"),
            (33, "PageUp"),
            (34, "PageDown"),
            (35, "End"),
            (36, "Home"),
            (45, "Insert"),
            (46, "Delete"),
            (112, "F1"),
            (113, "F2"),
            (114, "F3"),
            (115, "F4"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (vk, name) in specials {
                ui.menu_button(name, |ui| {
                    if ui.button("held").clicked() {
                        on_pick(vk, 0);
                    }
                    if ui.button("press").clicked() {
                        on_pick(vk, 1);
                    }
                    if ui.button("release").clicked() {
                        on_pick(vk, 2);
                    }
                });
            }
        });
    });
}

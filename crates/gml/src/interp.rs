use crate::ast::*;
use crate::value::*;
use std::collections::{HashMap, HashSet};

const OP_LIMIT: u64 = 30_000_000;
pub const SELF: i64 = -1;
pub const OTHER: i64 = -2;
pub const ALL: i64 = -3;
pub const NOONE: i64 = -4;

/// Control flow signal out of a statement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Control {
    Normal,
    Break,
    Exit,
}

/// The host environment the interpreter runs against: variables,
/// constants, instances and native/script functions. Implemented by
/// the game engine (gmengine::Game).
pub trait Runtime {
    /// Read a variable (builtin or user) from an instance.
    fn read_instance_var(&mut self, inst: i64, name: &str) -> Option<Value>;
    /// Write a variable on an instance; unknown names are created.
    fn write_instance_var(&mut self, inst: i64, name: &str, v: Value) -> Result<(), String>;
    fn read_user_global(&mut self, name: &str) -> Option<Value>;
    fn write_user_global(&mut self, name: &str, v: Value);
    /// score, mouse_x, room, ...
    fn read_global_builtin(&mut self, name: &str) -> Option<Value>;
    fn write_global_builtin(&mut self, name: &str, v: Value) -> Result<(), String>;
    /// Resource names (obj_*, spr_*, rm_*), c_*, vk_*, ev_* ...
    fn read_constant(&mut self, name: &str) -> Option<Value>;
    /// obj.foo = v sets the variable on ALL instances of the object.
    fn write_object_var(&mut self, obj: i64, name: &str, v: Value);
    /// obj.foo read returns the value from the first matching instance.
    fn read_first_instance_var(&mut self, obj: i64, name: &str) -> Result<Value, String>;
    /// Resolve a with()/object reference into a list of instance ids.
    fn instances_matching(&mut self, target: &Value) -> Result<Vec<i64>, String>;
    fn is_alive(&mut self, id: i64) -> bool;
    /// Native functions and scripts.
    fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        self_id: i64,
        other_id: i64,
    ) -> Result<Value, String>;
}

enum Tgt {
    Inst(i64),
    Obj(i64),
    Global,
    All,
}

enum Holder {
    Local(String),
    Args,
    Inst(i64, String),
    GlobalUser(String),
    GlobalBuiltin(String),
}

struct Frame {
    vars: HashMap<String, Value>,
    globals: HashSet<String>,
}

/// Tree-walking GML interpreter. Create one per script/event invocation.
pub struct Interp<'r> {
    rt: &'r mut dyn Runtime,
    locals: Vec<Frame>,
    args: Vec<Vec<Value>>,
    self_id: i64,
    other_id: i64,
    last: Value,
    ops: u64,
}

fn at_line(err: String, line: Line) -> String {
    if err.starts_with("line ") {
        err
    } else {
        format!("line {}: {}", line, err)
    }
}

impl<'r> Interp<'r> {
    pub fn new(rt: &'r mut dyn Runtime) -> Self {
        Interp {
            rt,
            locals: Vec::new(),
            args: Vec::new(),
            self_id: NOONE,
            other_id: NOONE,
            last: Value::Real(0.0),
            ops: 0,
        }
    }

    /// Run a compiled program as a script call. Returns the value of the
    /// last executed expression (legacy GM script return semantics).
    pub fn run(
        &mut self,
        prog: &Program,
        args: Vec<Value>,
        self_id: i64,
        other_id: i64,
    ) -> Result<Value, String> {
        self.locals.push(Frame {
            vars: HashMap::new(),
            globals: HashSet::new(),
        });
        self.args.push(args);
        let old_self = self.self_id;
        let old_other = self.other_id;
        self.self_id = self_id;
        self.other_id = other_id;
        let r = self.exec_block(&prog.stmts);
        self.self_id = old_self;
        self.other_id = old_other;
        self.args.pop();
        self.locals.pop();
        match r? {
            Control::Break => Err("break used outside of a loop".into()),
            _ => Ok(self.last.clone()),
        }
    }

    pub fn self_id(&self) -> i64 {
        self.self_id
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Control, String> {
        for s in stmts {
            match self.exec_stmt(s)? {
                Control::Normal => {}
                c => return Ok(c),
            }
        }
        Ok(Control::Normal)
    }

    fn exec_stmt(&mut self, s: &Stmt) -> Result<Control, String> {
        self.ops += 1;
        if self.ops > OP_LIMIT {
            return Err("possible infinite loop detected (over 30 million operations)".into());
        }
        match s {
            Stmt::Empty => Ok(Control::Normal),
            Stmt::Block(stmts, _) => self.exec_block(stmts),
            Stmt::Expr(e, _) => {
                self.last = self.eval(e)?;
                Ok(Control::Normal)
            }
            Stmt::VarDecl(names, _) => {
                if let Some(f) = self.locals.last_mut() {
                    for n in names {
                        f.vars.insert(n.clone(), Value::Real(0.0));
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::GlobalVarDecl(names, _) => {
                if let Some(f) = self.locals.last_mut() {
                    for n in names {
                        f.globals.insert(n.clone());
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::If(c, t, e, line) => {
                let cv = self.eval(c).map_err(|err| at_line(err, *line))?.as_bool()?;
                if cv {
                    self.exec_stmt(t)
                } else if let Some(eb) = e {
                    self.exec_stmt(eb)
                } else {
                    Ok(Control::Normal)
                }
            }
            Stmt::While(c, b, line) => {
                loop {
                    let cv = self.eval(c).map_err(|err| at_line(err, *line))?.as_bool()?;
                    if !cv {
                        break;
                    }
                    match self.exec_stmt(b)? {
                        Control::Break => break,
                        Control::Exit => return Ok(Control::Exit),
                        Control::Normal => {}
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::DoUntil(b, c, line) => {
                loop {
                    match self.exec_stmt(b)? {
                        Control::Break => break,
                        Control::Exit => return Ok(Control::Exit),
                        Control::Normal => {}
                    }
                    let cv = self.eval(c).map_err(|err| at_line(err, *line))?.as_bool()?;
                    if cv {
                        break;
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::Repeat(n, b, line) => {
                let count = self
                    .eval(n)
                    .map_err(|err| at_line(err, *line))?
                    .as_f64()?
                    .floor()
                    .max(0.0) as i64;
                for _ in 0..count {
                    match self.exec_stmt(b)? {
                        Control::Break => break,
                        Control::Exit => return Ok(Control::Exit),
                        Control::Normal => {}
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::For(init, cond, step, body, line) => {
                self.exec_stmt(init)?;
                loop {
                    if let Some(c) = cond {
                        let cv = self.eval(c).map_err(|err| at_line(err, *line))?.as_bool()?;
                        if !cv {
                            break;
                        }
                    }
                    match self.exec_stmt(body)? {
                        Control::Break => break,
                        Control::Exit => return Ok(Control::Exit),
                        Control::Normal => {}
                    }
                    if let Some(st) = step {
                        self.exec_stmt(st)?;
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::Switch(subject, arms, line) => {
                let sv = self.eval(subject).map_err(|err| at_line(err, *line))?;
                let mut chosen: Option<&SwitchArm> = None;
                let mut default_arm: Option<&SwitchArm> = None;
                for arm in arms {
                    match &arm.value {
                        Some(v) => {
                            if chosen.is_none() {
                                let av = self.eval(v)?;
                                if value_eq(&sv, &av)? {
                                    chosen = Some(arm);
                                }
                            }
                        }
                        None => default_arm = Some(arm),
                    }
                }
                let arm = chosen.or(default_arm);
                // GM8 switch has no fall-through: execute only the matching arm.
                if let Some(arm) = arm {
                    match self.exec_block(&arm.body)? {
                        Control::Break | Control::Normal => {}
                        Control::Exit => return Ok(Control::Exit),
                    }
                }
                Ok(Control::Normal)
            }
            Stmt::Break(_) => Ok(Control::Break),
            Stmt::Exit(_) => Ok(Control::Exit),
            Stmt::With(target, body, line) => {
                let v = self.eval(target).map_err(|err| at_line(err, *line))?;
                let ids = self
                    .rt
                    .instances_matching(&v)
                    .map_err(|e| at_line(e, *line))?;
                let outer_self = self.self_id;
                let outer_other = self.other_id;
                for id in ids {
                    if !self.rt.is_alive(id) {
                        continue;
                    }
                    self.self_id = id;
                    self.other_id = outer_self;
                    match self.exec_stmt(body)? {
                        Control::Break => break,
                        Control::Exit => {
                            self.self_id = outer_self;
                            self.other_id = outer_other;
                            return Ok(Control::Exit);
                        }
                        Control::Normal => {}
                    }
                }
                self.self_id = outer_self;
                self.other_id = outer_other;
                Ok(Control::Normal)
            }
        }
    }

    // ---------- expression evaluation ----------

    fn eval(&mut self, e: &Expr) -> Result<Value, String> {
        self.ops += 1;
        if self.ops > OP_LIMIT {
            return Err("possible infinite loop detected".into());
        }
        match e {
            Expr::Num(x) => Ok(Value::Real(*x)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Ident(name, line) => match name.as_str() {
                "self" => Ok(Value::Real(self.self_id as f64)),
                "other" => Ok(Value::Real(self.other_id as f64)),
                "all" => Ok(Value::Real(ALL as f64)),
                "noone" => Ok(Value::Real(NOONE as f64)),
                "global" => Err("'global' must be followed by a variable name".into()),
                _ => self.read_name(name).map_err(|err| at_line(err, *line)),
            },
            Expr::Bin(op, a, b, line) => {
                let av = self.eval(a)?;
                let bv = self.eval(b)?;
                bin_op(*op, av, bv).map_err(|err| at_line(err, *line))
            }
            Expr::Un(op, a, line) => {
                let av = self.eval(a).map_err(|err| at_line(err, *line))?;
                match op {
                    UnOp::Neg => match av {
                        Value::Real(x) => Ok(Value::Real(-x)),
                        v => Err(at_line(format!("cannot negate a {}", v.type_name()), *line)),
                    },
                    UnOp::Not => {
                        let bv = av.as_bool().map_err(|e| at_line(e, *line))?;
                        Ok(Value::Real((!bv) as i32 as f64))
                    }
                }
            }
            Expr::Ternary(c, a, b, line) => {
                let cv = self.eval(c).map_err(|err| at_line(err, *line))?.as_bool()?;
                if cv {
                    self.eval(a)
                } else {
                    self.eval(b)
                }
            }
            Expr::Assign(lv, op, rhs, line) => {
                let rv = self.eval(rhs).map_err(|err| at_line(err, *line))?;
                let v = if *op == AssignOp::Set {
                    rv
                } else {
                    let cur = self.read_lvalue(lv).map_err(|e| at_line(e, *line))?;
                    let bop = match op {
                        AssignOp::Add => BinOp::Add,
                        AssignOp::Sub => BinOp::Sub,
                        AssignOp::Mul => BinOp::Mul,
                        AssignOp::Div => BinOp::Div,
                        AssignOp::Set => unreachable!(),
                    };
                    bin_op(bop, cur, rv).map_err(|e| at_line(e, *line))?
                };
                self.write_lvalue(lv, v.clone())
                    .map_err(|e| at_line(e, *line))?;
                Ok(v)
            }
            Expr::PreInc(lv, line) => {
                let cur = self
                    .read_lvalue(lv)
                    .map_err(|e| at_line(e, *line))?
                    .as_f64()?;
                let nv = Value::Real(cur + 1.0);
                self.write_lvalue(lv, nv.clone())
                    .map_err(|e| at_line(e, *line))?;
                Ok(nv)
            }
            Expr::PreDec(lv, line) => {
                let cur = self
                    .read_lvalue(lv)
                    .map_err(|e| at_line(e, *line))?
                    .as_f64()?;
                let nv = Value::Real(cur - 1.0);
                self.write_lvalue(lv, nv.clone())
                    .map_err(|e| at_line(e, *line))?;
                Ok(nv)
            }
            Expr::PostInc(lv, line) => {
                let cur = self.read_lvalue(lv).map_err(|e| at_line(e, *line))?;
                let nv = Value::Real(cur.as_f64().map_err(|e| at_line(e, *line))? + 1.0);
                self.write_lvalue(lv, nv).map_err(|e| at_line(e, *line))?;
                Ok(cur)
            }
            Expr::PostDec(lv, line) => {
                let cur = self.read_lvalue(lv).map_err(|e| at_line(e, *line))?;
                let nv = Value::Real(cur.as_f64().map_err(|e| at_line(e, *line))? - 1.0);
                self.write_lvalue(lv, nv).map_err(|e| at_line(e, *line))?;
                Ok(cur)
            }
            Expr::Call(name, args, line) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a).map_err(|err| at_line(err, *line))?);
                }
                self.rt
                    .call_function(name, vals, self.self_id, self.other_id)
                    .map_err(|e| at_line(format!("in function {}(): {}", name, e), *line))
            }
            Expr::Member(base, name, line) => {
                let t = self.resolve_target(base).map_err(|e| at_line(e, *line))?;
                self.read_target(t, name).map_err(|e| at_line(e, *line))
            }
            Expr::Index(base, idxs, line) => {
                let (r, c) = self.eval_indices(idxs, *line)?;
                let base_val = self
                    .eval_or_read_base(base)
                    .map_err(|e| at_line(e, *line))?;
                match &base_val {
                    Value::Array { .. } => array_get(&base_val, r, c).cloned().ok_or_else(|| {
                        at_line(
                            format!(
                                "array index out of range [{}{}]",
                                r,
                                if c > 0 {
                                    format!(", {}", c)
                                } else {
                                    String::new()
                                }
                            ),
                            *line,
                        )
                    }),
                    v => Err(at_line(
                        format!("tried to index a {} value", v.type_name()),
                        *line,
                    )),
                }
            }
        }
    }

    fn eval_or_read_base(&mut self, base: &Expr) -> Result<Value, String> {
        // Try to read through holders so `argument`/locals keep identity.
        match base {
            Expr::Ident(name, _) => self.read_name(name),
            Expr::Member(_, _, _) => {
                // fall through to eval
                self.eval(base)
            }
            _ => self.eval(base),
        }
    }

    fn eval_indices(&mut self, idxs: &[Expr], line: Line) -> Result<(usize, usize), String> {
        let r = self
            .eval(&idxs[0])?
            .as_f64()
            .map_err(|e| at_line(e, line))?;
        if r < 0.0 || r.fract() != 0.0 {
            return Err(at_line(format!("invalid array index {}", r), line));
        }
        let c = if idxs.len() > 1 {
            let c = self
                .eval(&idxs[1])?
                .as_f64()
                .map_err(|e| at_line(e, line))?;
            if c < 0.0 || c.fract() != 0.0 {
                return Err(at_line(format!("invalid array index {}", c), line));
            }
            c as usize
        } else {
            0
        };
        Ok((r as usize, c))
    }

    fn resolve_target(&mut self, base: &Expr) -> Result<Tgt, String> {
        match base {
            Expr::Ident(s, _) if s == "global" => Ok(Tgt::Global),
            Expr::Ident(s, _) if s == "self" => Ok(Tgt::Inst(self.self_id)),
            Expr::Ident(s, _) if s == "other" => Ok(Tgt::Inst(self.other_id)),
            Expr::Ident(s, _) if s == "all" => Ok(Tgt::All),
            _ => {
                let v = self.eval(base)?;
                match v {
                    Value::Real(id) if id >= 100001.0 && id == id.trunc() => {
                        Ok(Tgt::Inst(id as i64))
                    }
                    Value::Real(o) if (0.0..100001.0).contains(&o) => Ok(Tgt::Obj(o as i64)),
                    _ => Err("invalid instance or object reference".into()),
                }
            }
        }
    }

    fn read_target(&mut self, t: Tgt, name: &str) -> Result<Value, String> {
        match t {
            Tgt::Global => self
                .rt
                .read_user_global(name)
                .or_else(|| self.rt.read_global_builtin(name))
                .ok_or_else(|| format!("unknown variable global.{}", name)),
            Tgt::Inst(id) => {
                if !self.rt.is_alive(id) {
                    return Err(format!(
                        "cannot read variable '{}' of a destroyed instance",
                        name
                    ));
                }
                self.rt
                    .read_instance_var(id, name)
                    .ok_or_else(|| format!("unknown variable '{}'", name))
            }
            Tgt::Obj(o) => self.rt.read_first_instance_var(o, name),
            Tgt::All => Err("cannot read a variable from 'all'".into()),
        }
    }

    fn write_target(&mut self, t: Tgt, name: &str, v: Value) -> Result<(), String> {
        match t {
            Tgt::Global => {
                self.rt.write_user_global(name, v);
                Ok(())
            }
            Tgt::Inst(id) => {
                if !self.rt.is_alive(id) {
                    return Err(format!(
                        "cannot set variable '{}' of a destroyed instance",
                        name
                    ));
                }
                self.rt.write_instance_var(id, name, v)
            }
            Tgt::Obj(o) => {
                self.rt.write_object_var(o, name, v);
                Ok(())
            }
            Tgt::All => {
                self.rt.write_object_var(ALL, name, v);
                Ok(())
            }
        }
    }

    fn resolve_holder(&mut self, base: &Expr) -> Result<Holder, String> {
        match base {
            Expr::Ident(s, _) => {
                if s == "argument" {
                    return Ok(Holder::Args);
                }
                if let Some(f) = self.locals.last() {
                    if f.vars.contains_key(s) {
                        return Ok(Holder::Local(s.clone()));
                    }
                    if f.globals.contains(s) {
                        return Ok(Holder::GlobalUser(s.clone()));
                    }
                }
                if self.rt.read_global_builtin(s).is_some() {
                    return Ok(Holder::GlobalBuiltin(s.clone()));
                }
                Ok(Holder::Inst(self.self_id, s.clone()))
            }
            Expr::Member(b, name, _) => match self.resolve_target(b)? {
                Tgt::Global => Ok(Holder::GlobalUser(name.clone())),
                Tgt::Inst(id) => Ok(Holder::Inst(id, name.clone())),
                _ => Err("cannot index a variable of an object like this".into()),
            },
            _ => Err("invalid array variable".into()),
        }
    }

    fn holder_get(&mut self, h: &Holder) -> Result<Value, String> {
        match h {
            Holder::Local(s) => self
                .locals
                .last()
                .and_then(|f| f.vars.get(s).cloned())
                .ok_or_else(|| format!("unknown variable '{}'", s)),
            Holder::Args => Ok(Value::Array {
                data: self.args.last().cloned().unwrap_or_default(),
                w: 1,
            }),
            Holder::Inst(id, s) => {
                if !self.rt.is_alive(*id) {
                    return Err("instance does not exist".into());
                }
                self.rt
                    .read_instance_var(*id, s)
                    .ok_or_else(|| format!("unknown variable '{}'", s))
            }
            Holder::GlobalUser(s) => self
                .rt
                .read_user_global(s)
                .or_else(|| self.rt.read_global_builtin(s))
                .ok_or_else(|| format!("unknown variable global.{}", s)),
            Holder::GlobalBuiltin(s) => self
                .rt
                .read_global_builtin(s)
                .ok_or_else(|| format!("unknown variable '{}'", s)),
        }
    }

    fn holder_set(&mut self, h: &Holder, v: Value) -> Result<(), String> {
        match h {
            Holder::Local(s) => {
                if let Some(f) = self.locals.last_mut() {
                    f.vars.insert(s.clone(), v);
                    Ok(())
                } else {
                    Err("no local scope".into())
                }
            }
            Holder::Args => {
                if let Value::Array { data, .. } = v {
                    if let Some(a) = self.args.last_mut() {
                        *a = data;
                    }
                    Ok(())
                } else {
                    Err("cannot assign a non-array value to argument".into())
                }
            }
            Holder::Inst(id, s) => {
                if !self.rt.is_alive(*id) {
                    return Err("instance does not exist".into());
                }
                self.rt.write_instance_var(*id, s, v)
            }
            Holder::GlobalUser(s) => {
                self.rt.write_user_global(s, v);
                Ok(())
            }
            Holder::GlobalBuiltin(s) => self.rt.write_global_builtin(s, v),
        }
    }

    fn read_name(&mut self, name: &str) -> Result<Value, String> {
        if let Some(args) = self.args.last() {
            if name == "argument_count" {
                return Ok(Value::Real(args.len() as f64));
            }
            if name == "argument" {
                return Ok(Value::Array {
                    data: args.clone(),
                    w: 1,
                });
            }
            if let Some(rest) = name.strip_prefix("argument") {
                if !rest.is_empty() && rest.len() <= 2 && rest.bytes().all(|c| c.is_ascii_digit()) {
                    let idx: usize = rest.parse().unwrap();
                    return args
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| format!("argument{} was not supplied to the script", idx));
                }
            }
        }
        if let Some(f) = self.locals.last() {
            if let Some(v) = f.vars.get(name) {
                return Ok(v.clone());
            }
            if f.globals.contains(name) {
                return Ok(self.rt.read_user_global(name).unwrap_or(Value::Real(0.0)));
            }
        }
        if let Some(v) = self.rt.read_instance_var(self.self_id, name) {
            return Ok(v);
        }
        if let Some(v) = self.rt.read_global_builtin(name) {
            return Ok(v);
        }
        match name {
            "true" => return Ok(Value::Real(1.0)),
            "false" => return Ok(Value::Real(0.0)),
            "pi" => return Ok(Value::Real(std::f64::consts::PI)),
            _ => {}
        }
        if let Some(v) = self.rt.read_constant(name) {
            return Ok(v);
        }
        Err(format!("unknown variable '{}'", name))
    }

    fn write_name(&mut self, name: &str, v: Value) -> Result<(), String> {
        if let Some(args) = self.args.last_mut() {
            if let Some(rest) = name.strip_prefix("argument") {
                if !rest.is_empty() && rest.len() <= 2 && rest.bytes().all(|c| c.is_ascii_digit()) {
                    let idx: usize = rest.parse().unwrap();
                    if idx >= args.len() {
                        return Err(format!("cannot assign to argument{}", idx));
                    }
                    args[idx] = v;
                    return Ok(());
                }
            }
        }
        if let Some(f) = self.locals.last_mut() {
            if f.vars.contains_key(name) {
                f.vars.insert(name.to_string(), v);
                return Ok(());
            }
            if f.globals.contains(name) {
                self.rt.write_user_global(name, v);
                return Ok(());
            }
        }
        if self.rt.read_global_builtin(name).is_some() {
            return self.rt.write_global_builtin(name, v);
        }
        self.rt.write_instance_var(self.self_id, name, v)
    }

    fn read_lvalue(&mut self, lv: &LValue) -> Result<Value, String> {
        match lv {
            LValue::Ident(name, line) => self.read_name(name).map_err(|e| at_line(e, *line)),
            LValue::Member(base, name, line) => {
                let t = self.resolve_target(base).map_err(|e| at_line(e, *line))?;
                self.read_target(t, name).map_err(|e| at_line(e, *line))
            }
            LValue::Index(base, idxs, line) => {
                let (r, c) = self.eval_indices(idxs, *line)?;
                let h = self.resolve_holder(base).map_err(|e| at_line(e, *line))?;
                let v = self.holder_get(&h).map_err(|e| at_line(e, *line))?;
                match &v {
                    Value::Array { .. } => array_get(&v, r, c).cloned().ok_or_else(|| {
                        at_line(
                            format!(
                                "array index out of range [{}{}]",
                                r,
                                if c > 0 {
                                    format!(", {}", c)
                                } else {
                                    String::new()
                                }
                            ),
                            *line,
                        )
                    }),
                    x => Err(at_line(
                        format!("tried to index a {} value", x.type_name()),
                        *line,
                    )),
                }
            }
        }
    }

    fn write_lvalue(&mut self, lv: &LValue, v: Value) -> Result<(), String> {
        match lv {
            LValue::Ident(name, line) => self.write_name(name, v).map_err(|e| at_line(e, *line)),
            LValue::Member(base, name, line) => {
                let t = self.resolve_target(base).map_err(|e| at_line(e, *line))?;
                self.write_target(t, name, v).map_err(|e| at_line(e, *line))
            }
            LValue::Index(base, idxs, line) => {
                let (r, c) = self.eval_indices(idxs, *line)?;
                let h = self.resolve_holder(base).map_err(|e| at_line(e, *line))?;
                let cur = match self.holder_get(&h) {
                    Ok(v) => v,
                    Err(_) => Value::Array { data: vec![], w: 0 },
                };
                let mut cur = cur;
                array_set(&mut cur, r, c, v);
                self.holder_set(&h, cur).map_err(|e| at_line(e, *line))
            }
        }
    }
}

pub fn value_eq(a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (Value::Real(x), Value::Real(y)) => Ok(x == y),
        (Value::Str(x), Value::Str(y)) => Ok(x == y),
        (Value::Array { .. }, _) | (_, Value::Array { .. }) => Err("cannot compare arrays".into()),
        _ => Ok(false),
    }
}

fn mixed_concat(a: Value, b: Value) -> Result<Value, String> {
    Ok(Value::Str(format!(
        "{}{}",
        a.as_display_string(),
        b.as_display_string()
    )))
}

pub fn bin_op(op: BinOp, a: Value, b: Value) -> Result<Value, String> {
    match op {
        BinOp::Add => match (&a, &b) {
            (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x + y)),
            _ => mixed_concat(a, b),
        },
        BinOp::Sub => {
            let (x, y) = two_reals(&a, &b)?;
            Ok(Value::Real(x - y))
        }
        BinOp::Mul => match (&a, &b) {
            (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x * y)),
            (Value::Str(s), Value::Real(n)) | (Value::Real(n), Value::Str(s)) => {
                let count = (n.max(0.0).min(100_000.0)) as usize;
                Ok(Value::Str(s.repeat(count)))
            }
            _ => Err("invalid operands for *".into()),
        },
        BinOp::Div => {
            let (x, y) = two_reals(&a, &b)?;
            if y == 0.0 {
                return Err("division by 0".into());
            }
            Ok(Value::Real(x / y))
        }
        BinOp::IntDiv => {
            let (x, y) = two_reals(&a, &b)?;
            let (xi, yi) = (x as i64, y as i64);
            if yi == 0 {
                return Err("division by 0".into());
            }
            Ok(Value::Real((xi / yi) as f64))
        }
        BinOp::Mod => {
            let (x, y) = two_reals(&a, &b)?;
            if y == 0.0 {
                return Err("division by 0".into());
            }
            Ok(Value::Real(x - (x / y).trunc() * y))
        }
        BinOp::Eq => Ok(Value::Real(value_eq(&a, &b)? as i32 as f64)),
        BinOp::Ne => Ok(Value::Real((!value_eq(&a, &b)?) as i32 as f64)),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = match (&a, &b) {
                (Value::Real(x), Value::Real(y)) => {
                    x.partial_cmp(y).ok_or("cannot compare values")?
                }
                (Value::Str(x), Value::Str(y)) => {
                    x.partial_cmp(y).ok_or("cannot compare values")?
                }
                _ => {
                    return Err(format!(
                        "cannot compare a {} with a {}",
                        a.type_name(),
                        b.type_name()
                    ))
                }
            };
            use std::cmp::Ordering::*;
            let r = match op {
                BinOp::Lt => ord == Less,
                BinOp::Le => ord != Greater,
                BinOp::Gt => ord == Greater,
                BinOp::Ge => ord != Less,
                _ => unreachable!(),
            };
            Ok(Value::Real(r as i32 as f64))
        }
        BinOp::And => Ok(Value::Real((a.as_bool()? && b.as_bool()?) as i32 as f64)),
        BinOp::Or => Ok(Value::Real((a.as_bool()? || b.as_bool()?) as i32 as f64)),
        BinOp::Xor => Ok(Value::Real((a.as_bool()? ^ b.as_bool()?) as i32 as f64)),
    }
}

fn two_reals(a: &Value, b: &Value) -> Result<(f64, f64), String> {
    match (a, b) {
        (Value::Real(x), Value::Real(y)) => Ok((*x, *y)),
        _ => Err(format!(
            "expected numbers, got a {} and a {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

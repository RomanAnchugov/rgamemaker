use gml::{compile, tokenize, Tok};

/// A tiny host used to test the interpreter without the engine.
struct Mock {
    vars: std::collections::HashMap<(i64, String), gml::Value>,
    globals: std::collections::HashMap<String, gml::Value>,
    calls: Vec<String>,
}

impl Mock {
    fn new() -> Self {
        Mock {
            vars: std::collections::HashMap::new(),
            globals: std::collections::HashMap::new(),
            calls: Vec::new(),
        }
    }
}

impl gml::Runtime for Mock {
    fn read_instance_var(&mut self, inst: i64, name: &str) -> Option<gml::Value> {
        self.vars.get(&(inst, name.to_string())).cloned()
    }
    fn write_instance_var(&mut self, inst: i64, name: &str, v: gml::Value) -> Result<(), String> {
        self.vars.insert((inst, name.to_string()), v);
        Ok(())
    }
    fn read_user_global(&mut self, name: &str) -> Option<gml::Value> {
        self.globals.get(name).cloned()
    }
    fn write_user_global(&mut self, name: &str, v: gml::Value) {
        self.globals.insert(name.to_string(), v);
    }
    fn read_global_builtin(&mut self, _name: &str) -> Option<gml::Value> {
        None
    }
    fn write_global_builtin(&mut self, name: &str, _v: gml::Value) -> Result<(), String> {
        Err(format!("unknown variable '{}'", name))
    }
    fn read_constant(&mut self, name: &str) -> Option<gml::Value> {
        match name {
            "obj_test" => Some(gml::Value::Real(0.0)),
            _ => None,
        }
    }
    fn write_object_var(&mut self, _obj: i64, _name: &str, _v: gml::Value) {}
    fn read_first_instance_var(&mut self, _obj: i64, name: &str) -> Result<gml::Value, String> {
        self.vars
            .get(&(1, name.to_string()))
            .cloned()
            .ok_or(format!("unknown variable '{}'", name))
    }
    fn instances_matching(&mut self, target: &gml::Value) -> Result<Vec<i64>, String> {
        match target.as_f64()? as i64 {
            -3 => Ok(vec![1, 2, 3]),
            0 => Ok(vec![1, 2, 3]),
            -1 => Ok(vec![1]),
            -2 => Ok(vec![2]),
            -4 => Ok(vec![]),
            id @ 1..=3 => Ok(vec![id]),
            id if id >= 100001 => Ok(vec![id]),
            _ => Ok(vec![]),
        }
    }
    fn is_alive(&mut self, id: i64) -> bool {
        id <= 3 || id >= 100001
    }
    fn call_function(
        &mut self,
        name: &str,
        args: Vec<gml::Value>,
        _s: i64,
        _o: i64,
    ) -> Result<gml::Value, String> {
        self.calls.push(name.to_string());
        match name {
            "add" => {
                let a = args.get(0).ok_or("missing arg")?.as_f64()?;
                let b = args.get(1).ok_or("missing arg")?.as_f64()?;
                Ok(gml::Value::Real(a + b))
            }
            "get_argument_count" => Ok(gml::Value::Real(args.len() as f64)),
            "fail" => Err("oops".into()),
            _ => Err(format!("unknown function '{}'", name)),
        }
    }
}

fn run(src: &str) -> Result<gml::Value, String> {
    let prog = compile(src)?;
    let mut mock = Mock::new();
    let mut it = gml::Interp::new(&mut mock);
    it.run(&prog, vec![], 1, 2)
}

fn run_expect(src: &str, expected: f64) {
    let v = run(src).unwrap_or_else(|e| panic!("'{}' failed: {}", src, e));
    match v {
        gml::Value::Real(x) => assert!(
            (x - expected).abs() < 1e-9,
            "'{}' = {:?}, expected {}",
            src,
            x,
            expected
        ),
        other => panic!("'{}' returned {:?}, expected number", src, other),
    }
}

#[test]
fn arithmetic_and_precedence() {
    run_expect("1 + 2 * 3", 7.0);
    run_expect("(1 + 2) * 3", 9.0);
    run_expect("2 * 3 + 4 * 5", 26.0);
    run_expect("10 div 3", 3.0);
    run_expect("-7 div 2", -3.0);
    run_expect("7 mod 3", 1.0);
    run_expect("2 + 3 * 4 - 6 / 2", 11.0);
}

#[test]
fn comparisons_and_logic() {
    run_expect("1 < 2", 1.0);
    run_expect("2 <= 1", 0.0);
    run_expect("1 == 1 && 2 == 2", 1.0);
    run_expect("1 == 2 || 3 == 3", 1.0);
    run_expect("1 == 1 xor 1 == 1", 0.0);
    run_expect("!0", 1.0);
    run_expect("3 > 2 > 1", 0.0); // GM chains left-to-right: (3>2)>1 = 1>1 = false
}

#[test]
fn strings() {
    let v = run(r#""ab" + 1"#).unwrap();
    assert_eq!(v.as_display_string(), "ab1");
    let v = run(r#""x" * 3"#).unwrap();
    assert_eq!(v.as_display_string(), "xxx");
    let v = run(r#""a" == "a""#).unwrap();
    run_expect(r#""a" == "a""#, 1.0);
    let v = run(r#""5" + 5.0"#).unwrap();
    assert_eq!(v.as_display_string(), "55");
    let v = run("string_num = 1.5; string_num + 0.5").unwrap();
    assert_eq!(v.as_display_string(), "2");
}

#[test]
fn control_flow() {
    run_expect("a = 0; for (i = 0; i < 10; i += 1) a += i; a", 45.0);
    run_expect("a = 0; i = 0; while (i < 5) { a += i; i += 1; } a", 10.0);
    run_expect("a = 0; repeat (4) a += 2; a", 8.0);
    run_expect("a = 0; do { a += 1; } until (a >= 3); a", 3.0);
    run_expect("a = 0; if (1) a = 5; else a = 6; a", 5.0);
    run_expect("a = 0; if (0) a = 5; else a = 6; a", 6.0);
    // no fall-through switch
    run_expect(
        "v = 2; r = 0; switch (v) { case 1: r = 10; case 2: r = 20; case 3: r = 30; default: r = 40; } r",
        20.0,
    );
    run_expect(
        "v = 9; r = 0; switch (v) { case 1: r = 10; default: r = 99; } r",
        99.0,
    );
    run_expect(
        "a = 0; for (i = 0; i < 10; i += 1) { if (i == 5) break; a = i; } a",
        4.0,
    );
    run_expect("x = 3; y = (x = 5) + 2; x + y", 12.0); // assignment is an expression
    run_expect("a = 0; if (1) a = 5; else a = 6; a", 5.0);
}

#[test]
fn with_statement() {
    run_expect("with (all) { hits = id_var = 1; } hits", 1.0);
    // every instance got the variable
    let prog = compile("with (all) score_v = 2;").unwrap();
    let mut mock = Mock::new();
    let mut it = gml::Interp::new(&mut mock);
    it.run(&prog, vec![], 1, 2).unwrap();
    for id in 1..=3 {
        assert_eq!(
            mock.vars
                .get(&(id, "score_v".into()))
                .unwrap()
                .as_f64()
                .unwrap(),
            2.0
        );
    }
    // with(other) inside self
    run_expect("with (other) { flag = 7; } other.flag", 7.0);
}

#[test]
fn arrays() {
    run_expect("a[5] = 3; a[5]", 3.0);
    run_expect("a[0] = 1; a[1] = 2; a[0] + a[1]", 3.0);
    run_expect("m[2, 3] = 9; m[2, 3]", 9.0);
    run_expect("a[3] = 1; a[1] = 5; a[1] + a[3]", 6.0);
    run_expect("i = 1; a[i] = 8; i += 1; a[i - 1]", 8.0);
}

#[test]
fn vars_and_globals() {
    run_expect("var q; q = 4; q * 2", 8.0);
    run_expect("global.gv = 11; global.gv", 11.0);
    run_expect("globalvar gv2; gv2 = 5; gv2", 5.0);
}

#[test]
fn functions_and_args() {
    run_expect("add(2, 3)", 5.0);
    run_expect("add(add(1, 2), 3) * 2", 12.0);
    // script args
    let prog = compile("argument0 + argument1 + argument_count * 100").unwrap();
    let mut mock = Mock::new();
    let mut it = gml::Interp::new(&mut mock);
    let v = it
        .run(
            &prog,
            vec![gml::Value::Real(1.0), gml::Value::Real(2.0)],
            1,
            2,
        )
        .unwrap();
    assert_eq!(v.as_f64().unwrap(), 203.0); // 1 + 2 + 2*100
                                            // argument[] array
    let prog = compile("a = argument; a[0] * 10").unwrap();
    let v = it.run(&prog, vec![gml::Value::Real(4.0)], 1, 2).unwrap();
    assert_eq!(v.as_f64().unwrap(), 40.0);
}

#[test]
fn increment_decrement() {
    run_expect("i = 5; i++; i", 6.0);
    run_expect("i = 5; ++i; i", 6.0);
    run_expect("i = 5; i++ + ++i", 12.0); // 5 + 7
    run_expect("i = 5; i--; i", 4.0);
}

#[test]
fn ternary_and_hex() {
    run_expect("1 ? 10 : 20", 10.0);
    run_expect("0 ? 10 : 20", 20.0);
    run_expect("$FF", 255.0);
    run_expect("$10 * 2", 32.0);
}

#[test]
fn multiline_continuation() {
    run_expect("1 +\n2", 3.0);
    run_expect("a = 1\nb = 2\na + b", 3.0);
    run_expect("x = 5;\ny = 6;\nx * y", 30.0);
}

#[test]
fn comments() {
    run_expect("// comment\n1 + /* block\ncomment */ 2", 3.0);
}

#[test]
fn errors() {
    assert!(run("nonexistent_var").is_err());
    assert!(run("1 / 0").is_err());
    assert!(run("5 div 0").is_err());
    assert!(run(r#""a" < 1"#).is_err());
    assert!(run("fail()").is_err());
    assert!(run("x = ").is_err());
    assert!(run("if 1").is_err());
    // infinite loop protection
    assert!(run("while (1) { }").is_err());
}

#[test]
fn lexer_newlines_in_brackets() {
    // Newlines inside brackets are ignored
    run_expect("a[\n1\n] = 5\na[1]", 5.0);
    let toks = tokenize("a\nb").unwrap();
    let nl = toks.iter().any(|t| t.tok == Tok::Newline);
    assert!(nl);
    let toks = tokenize("f(\n1,\n2\n)").unwrap();
    assert!(!toks.iter().any(|t| t.tok == Tok::Newline));
}

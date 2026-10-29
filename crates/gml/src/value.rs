/// A GML value. GML is dynamically typed: reals, strings and 2D arrays.
#[derive(Clone, Debug)]
pub enum Value {
    Real(f64),
    Str(String),
    /// Row-major 2D array. `w` = number of columns (0 = empty).
    /// 1D access `a[i]` is equivalent to `a[i, 0]`.
    Array {
        data: Vec<Value>,
        w: usize,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn real(x: f64) -> Value {
        Value::Real(x)
    }

    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(s.into())
    }

    pub fn as_f64(&self) -> Result<f64, String> {
        match self {
            Value::Real(x) => Ok(*x),
            Value::Str(s) => Err(format!("expected a number, got the string '{}'", s)),
            Value::Array { .. } => Err("expected a number, got an array".into()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Real(x) => Ok(*x != 0.0),
            Value::Str(s) => Err(format!("a string was used in a condition: '{}'", s)),
            Value::Array { .. } => Err("an array was used in a condition".into()),
        }
    }

    /// Convert to a display string using GM number formatting.
    pub fn as_display_string(&self) -> String {
        match self {
            Value::Real(x) => format_gm_number(*x),
            Value::Str(s) => s.clone(),
            Value::Array { .. } => String::new(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Real(_) => "number",
            Value::Str(_) => "string",
            Value::Array { .. } => "array",
        }
    }
}

/// GM8 formats reals with at most 2 decimals and no trailing zeros.
pub fn format_gm_number(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x.fract().abs() < 1e-9 && x.abs() < 1e15 {
        return format!("{}", x.round() as i64);
    }
    let mut s = format!("{:.2}", x);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

pub fn array_get(v: &Value, r: usize, c: usize) -> Option<&Value> {
    if let Value::Array { data, w } = v {
        if *w == 0 || c >= *w {
            return None;
        }
        data.get(r * *w + c)
    } else {
        None
    }
}

/// Set element, growing the array as needed (GM arrays auto-grow on write).
pub fn array_set(v: &mut Value, r: usize, c: usize, val: Value) {
    if !matches!(v, Value::Array { .. }) {
        *v = Value::Array { data: vec![], w: 0 };
    }
    if let Value::Array { data, w } = v {
        if c >= *w {
            // Widen (redistribute rows, filling with 0).
            let new_w = c + 1;
            let rows = if *w == 0 { 0 } else { data.len() / *w };
            let mut nd = Vec::with_capacity(rows.max(r + 1) * new_w);
            for row in 0..rows {
                for col in 0..new_w {
                    nd.push(if col < *w {
                        data[row * *w + col].clone()
                    } else {
                        Value::Real(0.0)
                    });
                }
            }
            *data = nd;
            *w = new_w;
        }
        let need = (r + 1) * *w;
        if data.len() < need {
            data.resize(need, Value::Real(0.0));
        }
        data[r * *w + c] = val;
    }
}

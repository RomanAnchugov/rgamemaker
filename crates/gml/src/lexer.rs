/// A single token with source position info (line + byte range) for
/// error reporting and IDE syntax highlighting.
#[derive(Clone, Debug, PartialEq)]
pub struct TokItem {
    pub tok: Tok,
    pub line: u32,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Num(f64),
    Str(String),
    Id(String),
    Comment(String),
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Semi,
    Colon,
    Question,
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And, // && / and
    Or,  // || / or
    Xor, // xor
    Not, // ! / not
    Inc,
    Dec,
    Newline,
    Eof,
    KwIf,
    KwElse,
    KwWhile,
    KwDo,
    KwUntil,
    KwFor,
    KwRepeat,
    KwSwitch,
    KwCase,
    KwDefault,
    KwBreak,
    KwExit,
    KwWith,
    KwVar,
    KwGlobalVar,
    KwDiv,
    KwMod,
}

/// GML keywords (for IDE highlighting and autocompletion).
pub const KEYWORDS: &[&str] = &[
    "if",
    "else",
    "while",
    "do",
    "until",
    "for",
    "repeat",
    "switch",
    "case",
    "default",
    "break",
    "exit",
    "with",
    "var",
    "globalvar",
    "div",
    "mod",
    "and",
    "or",
    "xor",
    "not",
    "true",
    "false",
    "self",
    "other",
    "all",
    "noone",
    "global",
];

fn keyword(s: &str) -> Option<Tok> {
    Some(match s {
        "if" => Tok::KwIf,
        "else" => Tok::KwElse,
        "while" => Tok::KwWhile,
        "do" => Tok::KwDo,
        "until" => Tok::KwUntil,
        "for" => Tok::KwFor,
        "repeat" => Tok::KwRepeat,
        "switch" => Tok::KwSwitch,
        "case" => Tok::KwCase,
        "default" => Tok::KwDefault,
        "break" => Tok::KwBreak,
        "exit" => Tok::KwExit,
        "with" => Tok::KwWith,
        "var" => Tok::KwVar,
        "globalvar" => Tok::KwGlobalVar,
        "div" => Tok::KwDiv,
        "mod" => Tok::KwMod,
        "and" => Tok::And,
        "or" => Tok::Or,
        "xor" => Tok::Xor,
        "not" => Tok::Not,
        _ => return None,
    })
}

fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >= 0xC0 && b < 0xE0 {
        2
    } else if b >= 0xE0 && b < 0xF0 {
        3
    } else {
        4
    }
}

fn item(tok: Tok, line: u32, start: usize, end: usize) -> TokItem {
    TokItem {
        tok,
        line,
        start,
        end,
    }
}

/// Tokenize GML source. Newlines are significant (statement separators)
/// except inside parentheses/brackets, where they are skipped.
pub fn tokenize(src: &str) -> Result<Vec<TokItem>, String> {
    let b = src.as_bytes();
    let n = b.len();
    let mut i = 0usize;
    let mut line = 1u32;
    let mut depth = 0i32;
    let mut out: Vec<TokItem> = Vec::new();
    let is_id_start = |c: u8| c.is_ascii_alphabetic() || c == b'_';
    let is_id = |c: u8| c.is_ascii_alphanumeric() || c == b'_';

    while i < n {
        let c = b[i];
        let start = i;
        match c {
            b'\n' => {
                if depth == 0 {
                    out.push(item(Tok::Newline, line, start, i));
                }
                line += 1;
                i += 1;
            }
            b'\r' | b' ' | b'\t' | 0x0b | 0x0c => {
                i += 1;
            }
            b'/' if i + 1 < n && b[i + 1] == b'/' => {
                while i < n && b[i] != b'\n' {
                    i += 1;
                }
                out.push(item(Tok::Comment(String::new()), line, start, i));
            }
            b'/' if i + 1 < n && b[i + 1] == b'*' => {
                let ls = line;
                i += 2;
                let mut buf = String::new();
                while i + 1 < n && !(b[i] == b'*' && b[i + 1] == b'/') {
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    let l = utf8_len(b[i]);
                    match std::str::from_utf8(&b[i..(i + l).min(n)]) {
                        Ok(s) => buf.push_str(s),
                        Err(_) => buf.push('?'),
                    }
                    i += l;
                }
                if i + 1 >= n {
                    return Err(format!("line {}: unterminated block comment", ls));
                }
                i += 2;
                out.push(item(Tok::Comment(buf), ls, start, i));
            }
            b'"' | b'\'' => {
                let q = c;
                let ls = line;
                i += 1;
                let mut s = String::new();
                while i < n && b[i] != q {
                    if b[i] == b'\n' {
                        return Err(format!("line {}: unterminated string", ls));
                    }
                    if b[i] == b'\\' && i + 1 < n && (b[i + 1] == b'\\' || b[i + 1] == q) {
                        s.push(b[i + 1] as char);
                        i += 2;
                        continue;
                    }
                    let l = utf8_len(b[i]);
                    match std::str::from_utf8(&b[i..(i + l).min(n)]) {
                        Ok(x) => s.push_str(x),
                        Err(_) => return Err(format!("line {}: invalid utf-8 in string", ls)),
                    }
                    i += l;
                }
                if i >= n {
                    return Err(format!("line {}: unterminated string", ls));
                }
                i += 1;
                out.push(item(Tok::Str(s), ls, start, i));
            }
            b'$' => {
                let ls = line;
                i += 1;
                let hs = i;
                while i < n && b[i].is_ascii_hexdigit() {
                    i += 1;
                }
                if i == hs {
                    return Err(format!("line {}: bad hex literal", ls));
                }
                let v = u64::from_str_radix(std::str::from_utf8(&b[hs..i]).unwrap(), 16).unwrap();
                out.push(item(Tok::Num(v as f64), ls, start, i));
            }
            b'0'..=b'9' => {
                let ls = line;
                while i < n && b[i].is_ascii_digit() {
                    i += 1;
                }
                if i + 1 < n && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < n && b[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let v: f64 = std::str::from_utf8(&b[start..i]).unwrap().parse().unwrap();
                out.push(item(Tok::Num(v), ls, start, i));
            }
            b'.' if i + 1 < n && b[i + 1].is_ascii_digit() => {
                let ls = line;
                i += 1;
                while i < n && b[i].is_ascii_digit() {
                    i += 1;
                }
                let v: f64 = std::str::from_utf8(&b[start..i]).unwrap().parse().unwrap();
                out.push(item(Tok::Num(v), ls, start, i));
            }
            _ if is_id_start(c) => {
                let ls = line;
                while i < n && is_id(b[i]) {
                    i += 1;
                }
                let raw = std::str::from_utf8(&b[start..i])
                    .unwrap()
                    .to_ascii_lowercase();
                let tok = keyword(&raw).unwrap_or(Tok::Id(raw));
                out.push(item(tok, ls, start, i));
            }
            b'(' => {
                depth += 1;
                i += 1;
                out.push(item(Tok::LParen, line, start, i));
            }
            b')' => {
                depth = (depth - 1).max(0);
                i += 1;
                out.push(item(Tok::RParen, line, start, i));
            }
            b'[' => {
                depth += 1;
                i += 1;
                out.push(item(Tok::LBrack, line, start, i));
            }
            b']' => {
                depth = (depth - 1).max(0);
                i += 1;
                out.push(item(Tok::RBrack, line, start, i));
            }
            b'{' => {
                i += 1;
                out.push(item(Tok::LBrace, line, start, i));
            }
            b'}' => {
                i += 1;
                out.push(item(Tok::RBrace, line, start, i));
            }
            b',' => {
                i += 1;
                out.push(item(Tok::Comma, line, start, i));
            }
            b'.' => {
                i += 1;
                out.push(item(Tok::Dot, line, start, i));
            }
            b';' => {
                i += 1;
                out.push(item(Tok::Semi, line, start, i));
            }
            b':' => {
                i += 1;
                out.push(item(Tok::Colon, line, start, i));
            }
            b'?' => {
                i += 1;
                out.push(item(Tok::Question, line, start, i));
            }
            b'+' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::PlusAssign, line, start, i));
                } else if i + 1 < n && b[i + 1] == b'+' {
                    i += 2;
                    out.push(item(Tok::Inc, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Plus, line, start, i));
                }
            }
            b'-' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::MinusAssign, line, start, i));
                } else if i + 1 < n && b[i + 1] == b'-' {
                    i += 2;
                    out.push(item(Tok::Dec, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Minus, line, start, i));
                }
            }
            b'*' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::StarAssign, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Star, line, start, i));
                }
            }
            b'/' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::SlashAssign, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Slash, line, start, i));
                }
            }
            b'=' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::Eq, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Assign, line, start, i));
                }
            }
            b'!' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::Ne, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Not, line, start, i));
                }
            }
            b'<' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::Le, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Lt, line, start, i));
                }
            }
            b'>' => {
                if i + 1 < n && b[i + 1] == b'=' {
                    i += 2;
                    out.push(item(Tok::Ge, line, start, i));
                } else {
                    i += 1;
                    out.push(item(Tok::Gt, line, start, i));
                }
            }
            b'&' => {
                if i + 1 < n && b[i + 1] == b'&' {
                    i += 2;
                    out.push(item(Tok::And, line, start, i));
                } else {
                    return Err(format!(
                        "line {}: unexpected character '&' (bitwise ops are not in GM8)",
                        line
                    ));
                }
            }
            b'|' => {
                if i + 1 < n && b[i + 1] == b'|' {
                    i += 2;
                    out.push(item(Tok::Or, line, start, i));
                } else {
                    return Err(format!("line {}: unexpected character '|'", line));
                }
            }
            _ => {
                return Err(format!(
                    "line {}: unexpected character '{}'",
                    line, c as char
                ));
            }
        }
    }
    out.push(item(Tok::Eof, line, n, n));
    Ok(out)
}

use crate::ast::*;
use crate::lexer::{Tok, TokItem};

/// Parse a token stream (comments already filtered out) into a Program.
pub fn parse_tokens(items: &[TokItem]) -> Result<Program, String> {
    let filtered: Vec<TokItem> = items
        .iter()
        .filter(|i| !matches!(i.tok, Tok::Comment(_)))
        .cloned()
        .collect();
    let mut p = Parser { t: filtered, i: 0 };
    let stmts = p.parse_stmts_until(&[Tok::Eof])?;
    Ok(Program { stmts })
}

struct Parser {
    t: Vec<TokItem>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.t[self.i.min(self.t.len() - 1)].tok
    }
    fn line(&self) -> u32 {
        self.t[self.i.min(self.t.len() - 1)].line
    }
    fn bump(&mut self) -> Tok {
        let t = self.t[self.i.min(self.t.len() - 1)].tok.clone();
        if self.i < self.t.len() - 1 {
            self.i += 1;
        }
        t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == t {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: Tok, what: &str) -> Result<(), String> {
        if self.eat(&t) {
            Ok(())
        } else {
            Err(format!(
                "line {}: expected {} but found {:?}",
                self.line(),
                what,
                self.peek()
            ))
        }
    }
    /// Skip statement separators (newlines and semicolons).
    fn skip_sep(&mut self) {
        while matches!(self.peek(), Tok::Newline | Tok::Semi) {
            self.bump();
        }
    }
    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Tok::Newline) {
            self.bump();
        }
    }
    fn skip_line_seps(&mut self) {
        while matches!(self.peek(), Tok::Newline | Tok::Semi) {
            self.bump();
        }
    }

    fn parse_stmts_until(&mut self, stops: &[Tok]) -> Result<Vec<Stmt>, String> {
        let mut out = Vec::new();
        loop {
            self.skip_sep();
            if stops.contains(self.peek()) || matches!(self.peek(), Tok::Eof) {
                break;
            }
            out.push(self.parse_stmt()?);
        }
        Ok(out)
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        let line = self.line();
        self.expect(Tok::LBrace, "'{'")?;
        let stmts = self.parse_stmts_until(&[Tok::RBrace])?;
        self.expect(Tok::RBrace, "'}'")?;
        Ok(Stmt::Block(stmts, line))
    }

    /// Optionally parenthesized expression: `( e )` or `e`.
    fn parse_cond(&mut self) -> Result<Expr, String> {
        if self.eat(&Tok::LParen) {
            let e = self.parse_expr()?;
            self.expect(Tok::RParen, "')'")?;
            Ok(e)
        } else {
            self.parse_expr()
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        self.skip_sep();
        let line = self.line();
        match self.peek().clone() {
            Tok::LBrace => self.parse_block(),
            Tok::KwIf => {
                self.bump();
                let cond = self.parse_cond()?;
                self.skip_sep();
                let then_b = Box::new(self.parse_stmt()?);
                let mut else_b = None;
                let save = self.i;
                self.skip_line_seps();
                if matches!(self.peek(), Tok::KwElse) {
                    self.bump();
                    self.skip_sep();
                    else_b = Some(Box::new(self.parse_stmt()?));
                } else {
                    self.i = save;
                }
                Ok(Stmt::If(cond, then_b, else_b, line))
            }
            Tok::KwWhile => {
                self.bump();
                let cond = self.parse_cond()?;
                self.skip_sep();
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::While(cond, body, line))
            }
            Tok::KwDo => {
                self.bump();
                self.skip_sep();
                let body = Box::new(self.parse_stmt()?);
                self.skip_sep();
                self.expect(Tok::KwUntil, "'until'")?;
                let cond = self.parse_cond()?;
                Ok(Stmt::DoUntil(body, cond, line))
            }
            Tok::KwRepeat => {
                self.bump();
                let count = self.parse_cond()?;
                self.skip_sep();
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::Repeat(count, body, line))
            }
            Tok::KwFor => {
                self.bump();
                self.expect(Tok::LParen, "'(' after 'for'")?;
                self.skip_sep();
                let init = if matches!(self.peek(), Tok::Semi) {
                    Stmt::Empty
                } else if matches!(self.peek(), Tok::KwVar) {
                    let (names, l) = self.parse_var_decl()?;
                    Stmt::VarDecl(names, l)
                } else {
                    let e = self.parse_expr()?;
                    Stmt::Expr(e, line)
                };
                self.expect(Tok::Semi, "';' in 'for'")?;
                self.skip_sep();
                let cond = if matches!(self.peek(), Tok::Semi) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(Tok::Semi, "';' in 'for'")?;
                self.skip_sep();
                let step = if matches!(self.peek(), Tok::RParen) {
                    None
                } else {
                    Some(Box::new(Stmt::Expr(self.parse_expr()?, line)))
                };
                self.expect(Tok::RParen, "')' after 'for'")?;
                self.skip_sep();
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::For(Box::new(init), cond, step, body, line))
            }
            Tok::KwSwitch => {
                self.bump();
                let subject = self.parse_cond()?;
                self.skip_sep();
                self.expect(Tok::LBrace, "'{' after 'switch'")?;
                let mut arms: Vec<SwitchArm> = Vec::new();
                loop {
                    self.skip_sep();
                    match self.peek() {
                        Tok::RBrace => {
                            self.bump();
                            break;
                        }
                        Tok::KwCase => {
                            self.bump();
                            let v = self.parse_expr()?;
                            self.expect(Tok::Colon, "':' after case value")?;
                            let body = self.parse_stmts_until(&[
                                Tok::KwCase,
                                Tok::KwDefault,
                                Tok::RBrace,
                            ])?;
                            arms.push(SwitchArm {
                                value: Some(v),
                                body,
                            });
                        }
                        Tok::KwDefault => {
                            self.bump();
                            self.expect(Tok::Colon, "':' after 'default'")?;
                            let body = self.parse_stmts_until(&[
                                Tok::KwCase,
                                Tok::KwDefault,
                                Tok::RBrace,
                            ])?;
                            arms.push(SwitchArm { value: None, body });
                        }
                        Tok::Eof => {
                            return Err(format!(
                                "line {}: unexpected end of file in switch",
                                self.line()
                            ))
                        }
                        t => {
                            return Err(format!(
                                "line {}: expected 'case' or 'default' in switch, found {:?}",
                                self.line(),
                                t
                            ))
                        }
                    }
                }
                Ok(Stmt::Switch(subject, arms, line))
            }
            Tok::KwBreak => {
                self.bump();
                Ok(Stmt::Break(line))
            }
            Tok::KwExit => {
                self.bump();
                Ok(Stmt::Exit(line))
            }
            Tok::KwWith => {
                self.bump();
                let target = self.parse_cond()?;
                self.skip_sep();
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::With(target, body, line))
            }
            Tok::KwVar => {
                let (names, l) = self.parse_var_decl()?;
                Ok(Stmt::VarDecl(names, l))
            }
            Tok::KwGlobalVar => {
                let (names, l) = self.parse_var_decl()?;
                Ok(Stmt::GlobalVarDecl(names, l))
            }
            _ => {
                let e = self.parse_expr()?;
                Ok(Stmt::Expr(e, line))
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<(Vec<String>, u32), String> {
        let line = self.line();
        if !matches!(self.peek(), Tok::KwVar | Tok::KwGlobalVar) {
            return Err(format!("line {}: expected 'var' or 'globalvar'", line));
        }
        self.bump();
        let mut names = Vec::new();
        loop {
            match self.bump() {
                Tok::Id(s) => names.push(s),
                t => {
                    return Err(format!(
                        "line {}: expected variable name, found {:?}",
                        line, t
                    ))
                }
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.eat(&Tok::Semi);
        Ok((names, line))
    }

    // ---------- expressions ----------

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Result<Expr, String> {
        let line = self.line();
        let lhs = self.parse_ternary()?;
        let op = match self.peek() {
            Tok::Assign => AssignOp::Set,
            Tok::PlusAssign => AssignOp::Add,
            Tok::MinusAssign => AssignOp::Sub,
            Tok::StarAssign => AssignOp::Mul,
            Tok::SlashAssign => AssignOp::Div,
            _ => return Ok(lhs),
        };
        self.bump();
        let lv =
            to_lvalue(&lhs).ok_or_else(|| format!("line {}: invalid assignment target", line))?;
        let rhs = self.parse_assign()?;
        Ok(Expr::Assign(Box::new(lv), op, Box::new(rhs), line))
    }

    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let line = self.line();
        let c = self.parse_bin(1)?;
        if self.eat(&Tok::Question) {
            let a = self.parse_assign()?;
            self.expect(Tok::Colon, "':' in ternary")?;
            let b = self.parse_assign()?;
            return Ok(Expr::Ternary(Box::new(c), Box::new(a), Box::new(b), line));
        }
        Ok(c)
    }

    /// Look ahead past newlines; if a binary operator follows, skip the
    /// newlines and return it (line continuation after operators).
    fn peek_binop(&mut self) -> Option<(BinOp, u8, u32)> {
        let mut j = self.i;
        let mut line = self.line();
        if matches!(self.t[j].tok, Tok::Newline) {
            while j < self.t.len() && matches!(self.t[j].tok, Tok::Newline) {
                line = self.t[j].line;
                j += 1;
            }
            if j >= self.t.len() {
                return None;
            }
        }
        let op = match &self.t[j].tok {
            Tok::Or => BinOp::Or,
            Tok::Xor => BinOp::Xor,
            Tok::And => BinOp::And,
            Tok::Eq => BinOp::Eq,
            Tok::Ne => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            Tok::Plus => BinOp::Add,
            Tok::Minus => BinOp::Sub,
            Tok::Star => BinOp::Mul,
            Tok::Slash => BinOp::Div,
            Tok::KwDiv => BinOp::IntDiv,
            Tok::KwMod => BinOp::Mod,
            _ => return None,
        };
        let prec = binop_prec(&op);
        if matches!(self.t[self.i].tok, Tok::Newline) {
            self.i = j; // consume the newlines, the op is next
        }
        Some((op, prec, line))
    }

    fn parse_bin(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            let (op, prec, line) = match self.peek_binop() {
                Some(x) => x,
                None => break,
            };
            if prec < min_prec {
                break;
            }
            self.bump(); // operator
            let rhs = self.parse_bin(prec + 1)?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs), line);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.skip_newlines();
        let line = self.line();
        match self.peek() {
            Tok::Not => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(Expr::Un(UnOp::Not, Box::new(e), line))
            }
            Tok::Minus => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(Expr::Un(UnOp::Neg, Box::new(e), line))
            }
            Tok::Plus => {
                self.bump();
                self.parse_unary()
            }
            Tok::Inc => {
                self.bump();
                let e = self.parse_unary()?;
                let lv =
                    to_lvalue(&e).ok_or_else(|| format!("line {}: invalid ++ target", line))?;
                Ok(Expr::PreInc(Box::new(lv), line))
            }
            Tok::Dec => {
                self.bump();
                let e = self.parse_unary()?;
                let lv =
                    to_lvalue(&e).ok_or_else(|| format!("line {}: invalid -- target", line))?;
                Ok(Expr::PreDec(Box::new(lv), line))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let line = self.line();
        let mut e = self.parse_primary()?;
        loop {
            match self.peek() {
                Tok::Dot => {
                    self.bump();
                    match self.bump() {
                        Tok::Id(name) => e = Expr::Member(Box::new(e), name, line),
                        t => {
                            return Err(format!(
                                "line {}: expected variable name after '.', found {:?}",
                                line, t
                            ))
                        }
                    }
                }
                Tok::LBrack => {
                    self.bump();
                    let mut idx = vec![self.parse_expr()?];
                    if self.eat(&Tok::Comma) {
                        idx.push(self.parse_expr()?);
                    }
                    self.expect(Tok::RBrack, "']'")?;
                    e = Expr::Index(Box::new(e), idx, line);
                }
                Tok::Inc => {
                    self.bump();
                    let lv =
                        to_lvalue(&e).ok_or_else(|| format!("line {}: invalid ++ target", line))?;
                    e = Expr::PostInc(Box::new(lv), line);
                }
                Tok::Dec => {
                    self.bump();
                    let lv =
                        to_lvalue(&e).ok_or_else(|| format!("line {}: invalid -- target", line))?;
                    e = Expr::PostDec(Box::new(lv), line);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        // Line continuation: after an operator at end of line, skip the newline.
        self.skip_newlines();
        let line = self.line();
        match self.bump() {
            Tok::Num(x) => Ok(Expr::Num(x)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Id(name) => {
                // function call — only if '(' follows on the same line
                if matches!(self.peek(), Tok::LParen) {
                    self.bump();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Tok::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.eat(&Tok::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(Tok::RParen, "')'")?;
                    Ok(Expr::Call(name, args, line))
                } else {
                    Ok(Expr::Ident(name, line))
                }
            }
            Tok::LParen => {
                let e = self.parse_expr()?;
                self.expect(Tok::RParen, "')'")?;
                Ok(e)
            }
            t => Err(format!(
                "line {}: expected an expression, found {:?}",
                line, t
            )),
        }
    }
}

fn binop_prec(op: &BinOp) -> u8 {
    match op {
        BinOp::Or | BinOp::Xor => 1,
        BinOp::And => 2,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 3,
        BinOp::Add | BinOp::Sub => 4,
        BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod => 5,
    }
}

fn to_lvalue(e: &Expr) -> Option<LValue> {
    match e {
        Expr::Ident(s, l) => Some(LValue::Ident(s.clone(), *l)),
        Expr::Member(b, name, l) => Some(LValue::Member(b.clone(), name.clone(), *l)),
        Expr::Index(b, idx, l) => Some(LValue::Index(b.clone(), idx.clone(), *l)),
        _ => None,
    }
}

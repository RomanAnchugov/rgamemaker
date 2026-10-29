pub type Line = u32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv, // div
    Mod,    // mod
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Num(f64),
    Str(String),
    Ident(String, Line),
    Bin(BinOp, Box<Expr>, Box<Expr>, Line),
    Un(UnOp, Box<Expr>, Line),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>, Line),
    Assign(Box<LValue>, AssignOp, Box<Expr>, Line),
    PreInc(Box<LValue>, Line),
    PreDec(Box<LValue>, Line),
    PostInc(Box<LValue>, Line),
    PostDec(Box<LValue>, Line),
    Call(String, Vec<Expr>, Line),
    Member(Box<Expr>, String, Line),
    /// base[expr] or base[expr, expr]
    Index(Box<Expr>, Vec<Expr>, Line),
}

#[derive(Clone, Debug)]
pub enum LValue {
    Ident(String, Line),
    Member(Box<Expr>, String, Line),
    Index(Box<Expr>, Vec<Expr>, Line),
}

#[derive(Clone, Debug)]
pub struct SwitchArm {
    /// None = the `default` arm.
    pub value: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(Expr, Line),
    VarDecl(Vec<String>, Line),
    GlobalVarDecl(Vec<String>, Line),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>, Line),
    While(Expr, Box<Stmt>, Line),
    DoUntil(Box<Stmt>, Expr, Line),
    Repeat(Expr, Box<Stmt>, Line),
    /// for (init; cond; step) body
    For(Box<Stmt>, Option<Expr>, Option<Box<Stmt>>, Box<Stmt>, Line),
    Switch(Expr, Vec<SwitchArm>, Line),
    Break(Line),
    Exit(Line),
    With(Expr, Box<Stmt>, Line),
    Block(Vec<Stmt>, Line),
    Empty,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

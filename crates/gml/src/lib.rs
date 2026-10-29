mod ast;
mod interp;
mod lexer;
mod parser;
mod value;

pub use ast::*;
pub use interp::{Control, Interp, Runtime, ALL, NOONE, OTHER, SELF};
pub use lexer::{tokenize, Tok, TokItem, KEYWORDS};
pub use parser::parse_tokens;
pub use value::*;

/// Compile GML source into an executable program (AST).
pub fn compile(src: &str) -> Result<Program, String> {
    let toks = tokenize(src)?;
    parse_tokens(&toks)
}

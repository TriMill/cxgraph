mod scanner;
mod parser;
mod compiler;
mod optimizer;

pub use parser::ParseError;
pub use compiler::CompileError;

use self::optimizer::optimize;

#[derive(Debug, Clone, Copy)]
pub struct Position {
	pub pos: u32,
	pub line: u32,
	pub col: u32,
}

#[derive(Clone, Debug)]
pub enum Error<'s> {
	Parse(ParseError<'s>),
	Compile(CompileError<'s>),
}

impl <'s> From<CompileError<'s>> for Error<'s> {
    fn from(value: CompileError<'s>) -> Self {
        Self::Compile(value)
    }
}

impl <'s> From<ParseError<'s>> for Error<'s> {
    fn from(value: ParseError<'s>) -> Self {
        Self::Parse(value)
    }
}

pub fn compile(src: &str) -> Result<String, Error> {
	let mut buf = String::new();
	let stmts = parser::Parser::new(src).parse()?;
	let stmts = optimize(stmts);
	compiler::compile(&mut buf, &stmts)?;
	Ok(buf)
}

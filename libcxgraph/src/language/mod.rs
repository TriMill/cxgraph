use lalrpop_util::lalrpop_mod;

use crate::language::token::Lexer;

use self::compiler::Compiler;

mod token;
mod ast;
mod compiler;
mod builtins;

lalrpop_mod!(pub syntax, "/language/syntax.rs");

pub fn compile(src: &str) -> Result<String, Box<dyn std::error::Error>> {
	let lexer = Lexer::new(src);
	let result = syntax::ProgramParser::new()
		.parse(src, lexer)
		.map_err(|e| e.to_string())?;
	let mut wgsl = String::new();
	let mut cmp = Compiler::new(&mut wgsl);
	for defn in result {
		cmp.compile_defn(&defn)?;
	}
	cmp.ensure_plot_defined()?;
	Ok(wgsl)
}

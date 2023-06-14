use std::{collections::{HashSet, HashMap}, fmt};

use super::{ast::{Definition, Expression, ExpressionType, BinaryOp, UnaryOp}, builtins::{BUILTIN_CONSTS, BUILTIN_FUNCS}};

#[derive(Clone, Debug)]
pub struct CompileError(String);

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompileError {}

impl From<String> for CompileError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<fmt::Error> for CompileError {
    fn from(value: fmt::Error) -> Self {
        Self(value.to_string())
    }
}

fn format_char(buf: &mut String, c: char) {
	match c {
		'_' => buf.push_str("u_"),
		'\'' => buf.push_str("p_"),
		c => buf.push(c),
	}
}

fn format_name(prefix: &str, name: &str) -> String {
	let mut result = prefix.to_owned();
	result.reserve(name.len());
	for c in name.chars() {
		format_char(&mut result, c);
	}
	result
}

fn format_func(name: &str) -> String { format_name("func_", name) }
fn format_const(name: &str) -> String { format_name("const_", name) }
fn format_arg(name: &str) -> String { format_name("arg_", name) }
fn format_local(name: &str) -> String { format_name("local_", name) }
fn format_tmp(idx: usize) -> String { format!("tmp_{}", idx) }

pub struct Compiler<'w, 'i, W: fmt::Write> {
	buf: &'w mut W,
	vars: &'w HashMap<String, usize>,
	global_funcs: HashMap<&'i str, usize>,
	global_consts: HashSet<&'i str>,
}

#[derive(Clone)]
struct LocalState<'i> {
	local_vars: HashSet<&'i str>,
	next_tmp: usize,
}

impl<'i> LocalState<'i> {
	pub fn new() -> Self {
		Self {
			local_vars: HashSet::new(),
			next_tmp: 0,
		}
	}

	pub fn next_tmp(&mut self) -> String {
		let n = self.next_tmp;
		self.next_tmp += 1;
		format_tmp(n)
	}
}

impl<'w, 'i, W: fmt::Write> Compiler<'w, 'i, W> {
	pub fn new(buf: &'w mut W, vars: &'w HashMap<String, usize>) -> Self {
		Self {
			buf,
			vars,
			global_consts: HashSet::new(),
			global_funcs: HashMap::new(),
		}
	}

	pub fn compile_defn(&mut self, defn: &Definition<'i>) -> Result<(), CompileError> {
		match defn {
			Definition::Function { name, args, value } => {
				if self.global_consts.contains(name) || self.global_funcs.contains_key(name) {
					return Err(format!("name {name} is already declared in global scope").into())
				}
				write!(self.buf, "fn {}(", format_func(name))?;
				for arg in args {
					write!(self.buf, "{}: vec2f, ", format_arg(arg))?;
				}
				writeln!(self.buf, ") -> vec2f {{")?;

				let mut local = LocalState::new();
				for arg in args {
					writeln!(self.buf, "var {} = {};", format_local(arg), format_arg(arg))?;
					local.local_vars.insert(arg);
				}

				let mut last = String::with_capacity(0);
				for expr in value {
					last = self.compile_expr(&mut local, expr)?;
				}
				writeln!(self.buf, "return {last};\n}}")?;

				self.global_funcs.insert(name, args.len());
				Ok(())
			}
			Definition::Constant { name, value } => {
				if self.global_consts.contains(name) || self.global_funcs.contains_key(name) {
					return Err(format!("name {name} is already declared in global scope").into())
				}

				writeln!(self.buf, "fn {}() -> vec2f {{", format_const(name))?;
				let mut local = LocalState::new();

				let mut last = String::with_capacity(0);
				for expr in value {
					last = self.compile_expr(&mut local, expr)?;
				}
				writeln!(self.buf, "return {last};\n}}")?;

				self.global_consts.insert(name);
				Ok(())
			}
		}
	}

	pub fn ensure_plot_defined(&self) -> Result<(), CompileError> {
		if let Some(n) = self.global_funcs.get("plot") {
			if *n == 1 {
                Ok(())
			} else {
                Err("Plot function has wrong number of arguments".to_owned().into())
			}
		} else {
			Err("No plot function defined".to_owned().into())
		}
	}

	fn compile_expr(&mut self, local: &mut LocalState<'i>, expr: &Expression<'i>)
	-> Result<String, CompileError> {
		match expr.ty {
			ExpressionType::Name(v) => self.resolve_var(local, v),
			ExpressionType::Store(var) => {
				let a = self.compile_expr(local, &expr.children[0])?;
				let name = format_local(var);

				if !local.local_vars.contains(var) {
					write!(self.buf, "var ")?;
					local.local_vars.insert(var);
				}

				writeln!(self.buf, "{name} = {a};")?;
				Ok(name)
			},
			ExpressionType::Number(n) => {
				let name = local.next_tmp();
				writeln!(self.buf, "var {name} = vec2f({:?}, {:?});", n.re, n.im)?;
				Ok(name)
			},
			ExpressionType::Binary(op) => {
				let a = self.compile_expr(local, &expr.children[0])?;
				let b = self.compile_expr(local, &expr.children[1])?;
				let name = local.next_tmp();

				match op {
					BinaryOp::Add => writeln!(self.buf, "var {name} = {a} + {b};")?,
					BinaryOp::Sub => writeln!(self.buf, "var {name} = {a} - {b};")?,
					BinaryOp::Mul => writeln!(self.buf, "var {name} = c_mul({a}, {b});")?,
					BinaryOp::Div => writeln!(self.buf, "var {name} = c_div({a}, {b});")?,
					BinaryOp::Pow => writeln!(self.buf, "var {name} = c_pow({a}, {b});")?,
				}

				Ok(name)
			},
			ExpressionType::Unary(op) => {
				let a = self.compile_expr(local, &expr.children[0])?;
				let name = local.next_tmp();

				match op {
					UnaryOp::Pos => writeln!(self.buf, "var {name} = {a};")?,
					UnaryOp::Neg => writeln!(self.buf, "var {name} = -{a};")?,
					UnaryOp::Conj => writeln!(self.buf, "var {name} = c_conj({a});")?,
				}

				Ok(name)
			},
			ExpressionType::FnCall(f) => {
				let (fname, argc) = self.resolve_func(f)?;
				if argc != expr.children.len() {
					return Err(format!("function {f} expected {argc} args, got {}", expr.children.len()).into())
				}

				let mut args = Vec::with_capacity(expr.children.len());
				for child in &expr.children {
					args.push(self.compile_expr(local, child)?);
				}

				let name = local.next_tmp();
				write!(self.buf, "var {name} = {fname}(", )?;
				for arg in args {
					write!(self.buf, "{arg}, ")?;
				}
				writeln!(self.buf, ");")?;

				Ok(name)
			},
			ExpressionType::Sum { countvar, min, max }
			| ExpressionType::Prod { countvar, min, max } => {
				let acc = local.next_tmp();
				let ivar = local.next_tmp();
				if matches!(expr.ty, ExpressionType::Sum { .. }) {
					writeln!(self.buf, "var {acc} = vec2f(0.0, 0.0);")?;
				} else {
					writeln!(self.buf, "var {acc} = vec2f(1.0, 0.0);")?;
				}
				writeln!(self.buf, "for(var {ivar}: i32 = {min}; {ivar} <= {max}; {ivar}++) {{")?;
				writeln!(self.buf, "var {} = vec2f(f32({ivar}), 0.0);", format_local(countvar))?;
				let mut loop_local = local.clone();
				loop_local.local_vars.insert(countvar);
				let mut last = String::new();
				for child in &expr.children {
					last = self.compile_expr(&mut loop_local, child)?;
				}
				if matches!(expr.ty, ExpressionType::Sum { .. }) {
					writeln!(self.buf, "{acc} = {acc} + {last};\n}}")?;
				} else {
					writeln!(self.buf, "{acc} = c_mul({acc}, {last});\n}}")?;
				}
				Ok(acc)
			},
			ExpressionType::Iter { itervar, count } => {
				let init = expr.children.last().unwrap();
				let itervar_fmt = format_local(itervar);
				let v = self.compile_expr(local, init)?;
				writeln!(self.buf, "var {itervar_fmt} = {v};")?;
				let ivar = local.next_tmp();
				writeln!(self.buf, "for(var {ivar}: i32 = 0; {ivar} < {count}; {ivar}++) {{")?;
				let mut loop_local = local.clone();
				loop_local.local_vars.insert(itervar);
				let mut last = String::new();
				for child in &expr.children[..expr.children.len() - 1] {
					last = self.compile_expr(&mut loop_local, child)?;
				}
				writeln!(self.buf, "{itervar_fmt} = {last};\n}}")?;
				Ok(itervar_fmt)
			}
		}
	}

	fn resolve_func(&self, name: &str) -> Result<(String, usize), CompileError> {
		if let Some(argc) = self.global_funcs.get(name) {
			Ok((format_func(name), *argc))
		} else if let Some((var, argc)) = BUILTIN_FUNCS.with(|c| c.get(name).copied()) {
			Ok(((*var).to_owned(), argc))
		} else {
			Err(format!("use of undeclared function {name}").into())
		}
	}

	fn resolve_var(&self, local: &LocalState, name: &str) -> Result<String, CompileError> {
		if local.local_vars.contains(name) {
			Ok(format_local(name))
		} else if self.global_consts.contains(name) {
			Ok(format_const(name) + "()")
		} else if let Some(var) = self.vars.get(name) {
			if var % 2 == 0 {
				Ok(format!("uniforms.variables[{}].xy", var/2))
			} else {
				Ok(format!("uniforms.variables[{}].zw", var/2))
			}
		} else if let Some(var) = BUILTIN_CONSTS.with(|c| Some(c.get(name)?.0)) {
			Ok(var.to_owned())
		} else {
			Err(format!("use of undeclared variable {name}").into())
		}
	}
}

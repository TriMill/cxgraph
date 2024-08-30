use std::fmt;

use num_complex::Complex64 as Complex;

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
	Add, Sub, Mul, Div, Pow,
	Gt, Lt, Ge, Le, Eq, Ne,
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
	Pos, Neg, Conj
}

#[derive(Clone, Copy, Debug)]
pub enum ExpressionType<'a> {
	Block,
	Number(Complex),
	Name(&'a str),
	Binary(BinaryOp),
	Unary(UnaryOp),
	FnCall(&'a str),
	Store(&'a str),
	If,
	Sum { countvar: &'a str, min: i32, max: i32 },
	Prod { countvar: &'a str, min: i32, max: i32 },
	Iter { itervar: &'a str, count: i32 },
}

#[derive(Clone, Debug)]
pub struct Expression<'a> {
	pub ty: ExpressionType<'a>,
	pub children: Vec<Expression<'a>>,
}

impl<'a> Expression<'a> {
	pub fn new_block(exs: Vec<Expression<'a>>) -> Self {
		Self { ty: ExpressionType::Block, children: exs }
	}

	pub fn new_number(x: f64) -> Self {
		Self { ty: ExpressionType::Number(Complex::new(x, 0.0)), children: Vec::with_capacity(0) }
	}

	pub fn new_name(n: &'a str) -> Self {
		Self { ty: ExpressionType::Name(n), children: Vec::with_capacity(0) }
	}

	pub fn new_unary(op: UnaryOp, arg: Self) -> Self {
		Self { ty: ExpressionType::Unary(op), children: vec![arg] }
	}

	pub fn new_binary(op: BinaryOp, arg0: Self, arg1: Self) -> Self {
		Self { ty: ExpressionType::Binary(op), children: vec![arg0, arg1] }
	}

	pub fn new_fncall(name: &'a str, args: Vec<Self>) -> Self {
		Self { ty: ExpressionType::FnCall(name), children: args }
	}

	pub fn new_store(expr: Self, name: &'a str) -> Self {
		Self { ty: ExpressionType::Store(name), children: vec![expr] }
	}

	pub fn new_if(cond: Self, t: Self, f: Self) -> Self {
		Self {
			ty: ExpressionType::If,
			children: vec![cond, t, f],
		}
	}

	pub fn new_sum(countvar: &'a str, min: i32, max: i32, body: Self) -> Self {
		Self {
			ty: ExpressionType::Sum { countvar, min, max },
			children: vec![body],
		}
	}

	pub fn new_prod(accvar: &'a str, min: i32, max: i32, body: Self) -> Self {
		Self {
			ty: ExpressionType::Prod { countvar: accvar, min, max },
			children: vec![body],
		}
	}

	pub fn new_iter(itervar: &'a str, count: i32, init: Self, body: Self) -> Self {
		Self {
			ty: ExpressionType::Iter { itervar, count },
			children: vec![init, body],
		}
	}
}

pub enum Definition<'a> {
	Constant { name: &'a str, value: Vec<Expression<'a>> },
	Function { name: &'a str, args: Vec<&'a str>, value: Vec<Expression<'a>> },
}

fn display_expr(w: &mut impl fmt::Write, expr: &Expression, depth: usize) -> fmt::Result {
	let indent = depth*2;
	match expr.ty {
		ExpressionType::Block => write!(w, "{:indent$}BLOCK", "", indent=indent)?,
		ExpressionType::Number(n) => write!(w, "{:indent$}NUMBER {n:?}", "", indent=indent)?,
		ExpressionType::Name(n) => write!(w, "{:indent$}NAME {n}", "", indent=indent)?,
		ExpressionType::Binary(op) => write!(w, "{:indent$}OP {op:?}", "", indent=indent)?,
		ExpressionType::Unary(op) => write!(w, "{:indent$}OP {op:?}", "", indent=indent)?,
		ExpressionType::FnCall(f) => write!(w, "{:indent$}CALL {f}", "", indent=indent)?,
		ExpressionType::Store(n) => write!(w, "{:indent$}STORE {n}", "", indent=indent)?,
		ExpressionType::If => write!(w, "{:indent$}IF", "", indent=indent)?,
		ExpressionType::Sum { countvar, min, max } => write!(w, "{:indent$}SUM {countvar} {min} {max}", "", indent=indent)?,
		ExpressionType::Prod { countvar, min, max } => write!(w, "{:indent$}PROD {countvar} {min} {max}", "", indent=indent)?,
		ExpressionType::Iter { itervar, count } => write!(w, "{:indent$}ITER {itervar} {count}", "", indent=indent)?,
	}
	writeln!(w)?;
	for child in &expr.children {
		display_expr(w, child, depth + 1)?;
	}
	Ok(())
}

pub fn display_def(w: &mut impl fmt::Write, def: &Definition) -> fmt::Result {
	match def {
		Definition::Constant { name, value } => {
			writeln!(w, "CONSTANT {name}")?;
			for expr in value {
				display_expr(w, expr, 1)?;
			}
		},
		Definition::Function { name, args, value } => {
			writeln!(w, "FUNCTION {name}")?;
			for arg in args {
				writeln!(w, "  ARG {arg}")?;
			}
			for expr in value {
				display_expr(w, expr, 1)?;
			}
		}
	}
	Ok(())
}

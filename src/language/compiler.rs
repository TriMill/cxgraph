use std::{collections::{HashMap, HashSet}, fmt::Write};

use crate::complex::{Complex, cxfn};

use super::parser::{Expr, Stmt, UnaryOp, BinaryOp};

#[derive(Clone, Debug)]
pub enum CompileError<'s> {
	FmtError,
	TypeError(&'s str),
	UndefinedVar(&'s str),
	Reassignment(&'s str),
}

impl <'s> From<std::fmt::Error> for CompileError<'s> {
    fn from(_: std::fmt::Error) -> Self {
        Self::FmtError
    }
}

thread_local! {
	pub static BUILTINS: HashMap<&'static str, (&'static str, Type, Option<cxfn::Function>)> = {
		let mut m: HashMap<&'static str, (&'static str, Type, Option<cxfn::Function>)> = HashMap::new();
		m.insert("i",      ("CONST_I",  Type::Number,      Some(|_| Complex::new(0.0, 1.0))));
		m.insert("e",      ("CONST_E",  Type::Number,      Some(|_| Complex::new(std::f64::consts::E, 0.0))));
		m.insert("tau",    ("CONST_TAU",Type::Number,      Some(|_| Complex::new(std::f64::consts::TAU, 0.0))));
		m.insert("re",     ("c_re",     Type::Function(1), Some(cxfn::re)));
		m.insert("im",     ("c_im",     Type::Function(1), Some(cxfn::im)));
		m.insert("conj",   ("c_conj",   Type::Function(1), Some(cxfn::conj)));
		m.insert("abs_sq", ("c_abs_sq", Type::Function(1), Some(cxfn::abs_sq)));
		m.insert("abs",    ("c_abs",    Type::Function(1), Some(cxfn::abs)));
		m.insert("arg",    ("c_arg",    Type::Function(1), Some(cxfn::arg)));
		m.insert("pos",    ("c_pos",    Type::Function(1), Some(cxfn::pos)));
		m.insert("neg",    ("c_neg",    Type::Function(1), Some(cxfn::neg)));
		m.insert("recip",  ("c_recip",  Type::Function(1), Some(cxfn::recip)));
		m.insert("add",    ("c_add",    Type::Function(2), Some(cxfn::add)));
		m.insert("sub",    ("c_sub",    Type::Function(2), Some(cxfn::sub)));
		m.insert("mul",    ("c_mul",    Type::Function(2), Some(cxfn::mul)));
		m.insert("div",    ("c_div",    Type::Function(2), Some(cxfn::div)));
		m.insert("recip",  ("c_recip",  Type::Function(1), Some(cxfn::recip)));
		m.insert("exp",    ("c_exp",    Type::Function(1), Some(cxfn::exp)));
		m.insert("log",    ("c_log",    Type::Function(1), Some(cxfn::log)));
		m.insert("sqrt",   ("c_sqrt",   Type::Function(1), Some(cxfn::sqrt)));
		m.insert("sin",    ("c_sin",    Type::Function(1), Some(cxfn::sin)));
		m.insert("cos",    ("c_cos",    Type::Function(1), Some(cxfn::cos)));
		m.insert("tan",    ("c_tan",    Type::Function(1), Some(cxfn::tan)));
		m.insert("sinh",   ("c_sinh",   Type::Function(1), Some(cxfn::sinh)));
		m.insert("cosh",   ("c_cosh",   Type::Function(1), Some(cxfn::cosh)));
		m.insert("tanh",   ("c_tanh",   Type::Function(1), Some(cxfn::tanh)));
		m.insert("gamma",  ("c_gamma",  Type::Function(1), None));
		m
	};
}

#[derive(Clone, Copy)]
pub enum Type {
	Number,
	Function(u32),
}

#[derive(Clone, Copy)]
enum NameScope {
	Local, Global, Builtin
}

struct NameInfo<'s> {
	scope: NameScope,
	name: &'s str,
	ty: Type
}

impl <'s> NameInfo<'s> {
	pub fn get_cname(&self) -> String {
		let name = self.name;
		match (self.scope, self.ty) {
			(NameScope::Local, _) => format!("arg_{name}"),
			(NameScope::Global, Type::Number) => format!("VAR_{name}"),
			(NameScope::Global, Type::Function(_)) => format!("func_{name}"),
			(NameScope::Builtin, _) => name.to_owned(),
		}
	}
}

type LocalTable<'s> = HashSet<&'s str>;

type CompileResult<'s,T=()> = Result<T, CompileError<'s>>;

pub fn compile<'s>(buf: &mut impl Write, stmts: &[Stmt<'s>]) -> CompileResult<'s> {
	let mut compiler = Compiler::new(buf);
	for stmt in stmts {
		compiler.compile_stmt(stmt)?;
	}
	Ok(())
}

struct Compiler<'s, 'w, W> where W: Write {
	buf: &'w mut W,
	globals: HashMap<&'s str, Type>,
}

impl <'s, 'w, W: Write> Compiler<'s, 'w, W> {
	fn new(buf: &'w mut W) -> Self {
		Self {
			buf,
			globals: HashMap::new(),
		}
	}

	//////////////////
	//  Statements  //
	//////////////////

    fn compile_stmt(&mut self, stmt: &Stmt<'s>) -> CompileResult<'s> {
        let res = match stmt {
            Stmt::Const { name, body } => self.stmt_const(name, body),
            Stmt::Func { name, args, body } => self.stmt_func(name, args, body),
            Stmt::Deriv { name, func } => self.stmt_deriv(name, func),
            Stmt::Iter { name, func, count } => self.stmt_iter(name, func, *count),
        };
		writeln!(self.buf)?;
		res
    }

	fn stmt_const(&mut self, name: &'s str, body: &Expr<'s>) -> CompileResult<'s> {
		if self.name_info(name, None).is_some() {
			return Err(CompileError::Reassignment(name))
		}

		self.globals.insert(name, Type::Number);
		write!(self.buf, "const VAR_{name} = ")?;

		let locals = LocalTable::with_capacity(0);
		self.compile_expr(&locals, body)?;

		write!(self.buf, ";")?;
		Ok(())
	}

	fn stmt_func(&mut self, name: &'s str, args: &[&'s str], body: &Expr<'s>) -> CompileResult<'s> {
		if self.name_info(name, None).is_some() {
			return Err(CompileError::Reassignment(name))
		}

		self.globals.insert(name, Type::Function(args.len() as u32));
		write!(self.buf, "fn func_{name}(")?;

		let mut locals = LocalTable::with_capacity(args.len());
		for arg in args {
			write!(self.buf, "arg_{arg}:vec2f,")?;
			locals.insert(arg);
		}
		write!(self.buf, ")->vec2f{{return ")?;
		self.compile_expr(&locals, body)?;
		write!(self.buf, ";}}")?;
		Ok(())
	}

	fn stmt_deriv(&mut self, name: &'s str, func: &'s str) -> CompileResult<'s> {
		if self.name_info(name, None).is_some() {
			return Err(CompileError::Reassignment(name))
		}

		let Some(name_info) = self.name_info(func, None) else {
			return Err(CompileError::UndefinedVar(name))
		};
		let Type::Function(argc) = name_info.ty else {
			return Err(CompileError::TypeError(name))
		};

		let func_name = name_info.get_cname();

		self.globals.insert(name, Type::Function(argc));

		write!(self.buf, "fn func_{name}(")?;

		for i in 0..argc {
			write!(self.buf, "arg_{i}:vec2f,")?;
		}
		let args: String = (1..argc).map(|i| format!(",arg_{i}")).collect();

		write!(self.buf, ")->vec2f{{\
			let a = c_mul({func_name}(arg_0 + vec2( D_EPS, 0.0){args}), vec2( 0.25/D_EPS, 0.0));\
			let b = c_mul({func_name}(arg_0 + vec2(-D_EPS, 0.0){args}), vec2(-0.25/D_EPS, 0.0));\
			let c = c_mul({func_name}(arg_0 + vec2(0.0,  D_EPS){args}), vec2(0.0, -0.25/D_EPS));\
			let d = c_mul({func_name}(arg_0 + vec2(0.0, -D_EPS){args}), vec2(0.0,  0.25/D_EPS));\
			return a + b + c + d;}}\
		")?;
		Ok(())
	}

	fn stmt_iter(&mut self, name: &'s str, func: &'s str, count: u32) -> CompileResult<'s> {
		if self.name_info(name, None).is_some() {
			return Err(CompileError::Reassignment(name))
		}

		let Some(name_info) = self.name_info(func, None) else {
			return Err(CompileError::UndefinedVar(name))
		};
		let Type::Function(argc) = name_info.ty else {
			return Err(CompileError::TypeError(name))
		};

		let func_name = name_info.get_cname();

		self.globals.insert(name, Type::Function(argc));

		write!(self.buf, "fn func_{name}(")?;

		for i in 0..argc {
			write!(self.buf, "arg_{i}:vec2f,")?;
		}
		let args: String = (1..argc).map(|i| format!(",arg_{i}")).collect();

		write!(self.buf, ")->vec2f{{\
			var r=arg_0;\
			for(var i=0;i<{count};i++){{\
				r={func_name}(r{args});\
			}}\
			return r;}}\
		")?;
		Ok(())
	}

	///////////////////
	//  Expressions  //
	///////////////////

    fn compile_expr(&mut self, locals: &LocalTable<'s>, expr: &Expr<'s>) -> CompileResult<'s> {
        match expr {
            Expr::Number(z) => self.expr_number(*z),
            Expr::Name(name) => self.expr_var(locals, name),
            Expr::Unary(op, arg) => self.expr_unary(locals, *op, arg),
            Expr::Binary(op, lhs, rhs) => self.expr_binary(locals, *op, lhs, rhs),
            Expr::FnCall(name, args) => self.expr_fncall(locals, name, args),
        }
    }

	fn expr_number(&mut self, z: Complex) -> CompileResult<'s> {
        write!(self.buf, "vec2f({:?},{:?})", z.re, z.im)?;
		Ok(())
	}

	fn expr_unary(&mut self, locals: &LocalTable<'s>, op: UnaryOp, arg: &Expr<'s>) -> CompileResult<'s> {
        let strings = unop_strings(op);
        write!(self.buf, "{}", strings[0])?;
        self.compile_expr(locals, arg)?;
        write!(self.buf, "{}", strings[1])?;
        Ok(())
	}

	fn expr_binary(&mut self, locals: &LocalTable<'s>, op: BinaryOp, lhs: &Expr<'s>, rhs: &Expr<'s>) -> CompileResult<'s> {
        let strings = binop_strings(op);
        write!(self.buf, "{}", strings[0])?;
        self.compile_expr(locals, lhs)?;
        write!(self.buf, "{}", strings[1])?;
        self.compile_expr(locals, rhs)?;
        write!(self.buf, "{}", strings[2])?;
        Ok(())
	}

	fn expr_var(&mut self, locals: &LocalTable<'s>, name: &'s str) -> CompileResult<'s> {
		let Some(name_info) = self.name_info(name, Some(locals)) else {
			return Err(CompileError::UndefinedVar(name))
		};
		if !matches!(name_info.ty, Type::Number) {
			return Err(CompileError::TypeError(name))
		}
		write!(self.buf, "{}", name_info.get_cname())?;
		Ok(())
	}

	fn expr_fncall(&mut self, locals: &LocalTable<'s>, name: &'s str, args: &Vec<Expr<'s>>) -> CompileResult<'s> {
		let Some(name_info) = self.name_info(name, Some(locals)) else {
			return Err(CompileError::UndefinedVar(name))
		};
		if !matches!(name_info.ty, Type::Function(n) if n as usize == args.len()) {
			return Err(CompileError::TypeError(name))
		}
		write!(self.buf, "{}", name_info.get_cname())?;
		write!(self.buf, "(")?;
        for arg in args {
            self.compile_expr(locals, arg)?;
			write!(self.buf, ",")?;
        }
        write!(self.buf, ")")?;
        Ok(())
	}

	/////////////
	//  Names  //
	/////////////

	fn name_info(&self, name: &'s str, locals: Option<&LocalTable<'s>>) -> Option<NameInfo> {
		if let Some(locals) = locals {
			if locals.contains(name) {
				return Some(NameInfo { scope: NameScope::Local, name, ty: Type::Number });
			}
		}
		if let Some(ty) = self.globals.get(name).copied() {
			return Some(NameInfo { scope: NameScope::Global, name, ty })
		}
		if let Some((bname, ty, _)) = BUILTINS.with(|m| m.get(name).copied()) {
			return Some(NameInfo { scope: NameScope::Builtin, name: bname, ty })
		}
		None
	}

	// fn generate_iter(&mut self, argc: u32) -> CompileResult<'s> {
	// 	if !self.generate.contains_key(&format!("invoke{argc}")) {
	// 		self.generate.insert(format!("invoke{argc}"), Generate::Iter { argc });
	// 		self.generate_invoke(argc)?;
	// 		writeln!(self.buf)?;
	// 	}
	// 	write!(self.buf, "fn iter{argc}(func:vec2f,n:vec2f,")?;
	// 	for i in 0..argc {
	// 		write!(self.buf, "arg_{i}:vec2f")?;
	// 		write!(self.buf, ",")?;
	// 	}
	// 	write!(self.buf, ")->vec2f{{var result=arg_0;")?;
	// 	write!(self.buf, "for(var i=0;i<i32(n.x);i++){{result=invoke{argc}(func,result,")?;
	// 	for i in 1..argc {
	// 		write!(self.buf, "arg_{i},")?;
	// 	}
	// 	write!(self.buf, ");}}return result;}}")?;
	// 	Ok(())
	// }
}


const fn unop_strings(op: UnaryOp) -> [&'static str; 2] {
	match op {
		UnaryOp::Pos => ["+(", ")"],
		UnaryOp::Neg => ["-(", ")"],
		UnaryOp::Recip => ["c_recip(", ")"],
	}
}

const fn binop_strings(op: BinaryOp) -> [&'static str; 3] {
	match op {
		BinaryOp::Add => ["(", ")+(", ")"],
		BinaryOp::Sub => ["(", ")-(", ")"],
		BinaryOp::Mul => ["c_mul(", ",", ")"],
		BinaryOp::Div => ["c_div(", ",", ")"],
		BinaryOp::Pow => ["c_pow(", ",", ")"],
	}
}

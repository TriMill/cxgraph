use std::{collections::HashMap, fmt::Write};

use regex::Regex;

use crate::complex::{Complex, cxfn};

use super::parser::{Expr, Stmt, UnaryOp, BinaryOp};

#[derive(Clone, Debug)]
pub enum CompileError<'s> {
	FmtError,
	TypeError(&'s str),
	ArgCount(&'s str),
	UndefinedVar(&'s str),
	GlobalReassignment(&'s str),
	BuiltinReassignment(&'s str),
	StandaloneDerivative(&'s str),
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

	static RE_INVOKE: Regex = Regex::new(r"^invoke(\d+)$").unwrap();
	static RE_ITER: Regex = Regex::new(r"^iter(\d+)$").unwrap();
}

#[derive(Clone, Copy)]
pub enum Type {
	Number,
	Function(u32),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Generate {
	FunctionId { argc: u32, id: u32 },
	Invoke { argc: u32 },
	Iter { argc: u32 },
	Derivative { func: String, nderiv: u32, argc: u32 },
}

enum NameInfo {
	Local { id: u32 },
	Global { ty: Type, cname: String },
	Builtin { ty: Type, bname: &'static str },
	Generated { ty: Type, gname: String, gen: Generate }
}

impl NameInfo {
	pub fn get_compiled_name(&self) -> String {
		match self {
			NameInfo::Local { id } => format!("arg_{id}"),
			NameInfo::Global { cname, .. } => cname.to_owned(),
			NameInfo::Builtin { bname, .. } => (*bname).to_owned(),
			NameInfo::Generated { gname, .. } => gname.to_owned(),
		}
	}
}

type LocalTable<'s> = HashMap<&'s str, u32>;

type CompileResult<'s,T=()> = Result<T, CompileError<'s>>;

pub fn compile<'s>(buf: &mut impl Write, stmts: &[Stmt<'s>]) -> CompileResult<'s> {
	let mut compiler = Compiler::new(buf);
	for stmt in stmts {
		compiler.compile_stmt(stmt)?;
	}
	compiler.generate()?;
	Ok(())
}


struct Compiler<'s, 'w, W> where W: Write {
	buf: &'w mut W,
	globals: HashMap<&'s str, Type>,
	generate: HashMap<String, Generate>,
	next_fn_id: u32,
}

impl <'s, 'w, W: Write> Compiler<'s, 'w, W> {
	fn new(buf: &'w mut W) -> Self {
		Self {
			buf,
			globals: HashMap::new(),
			generate: HashMap::new(),
			next_fn_id: 0,
		}
	}

	//////////////////
	//  Statements  //
	//////////////////

    fn compile_stmt(&mut self, stmt: &Stmt<'s>) -> CompileResult<'s> {
        match stmt {
            Stmt::AssignConst(name, expr) => {
				if BUILTINS.with(|m| m.contains_key(name)) {
                    return Err(CompileError::BuiltinReassignment(name))
				}
                if self.globals.contains_key(name) {
                    return Err(CompileError::GlobalReassignment(name))
                }
                self.globals.insert(name, Type::Number);
                write!(self.buf, "const VAR_{name} = ")?;

                let locals = LocalTable::with_capacity(0);
                self.compile_expr(&locals, expr)?;

                write!(self.buf, ";")?;
            }
            Stmt::AssignFunc(name, args, expr) => {
				if BUILTINS.with(|m| m.contains_key(name)) {
                    return Err(CompileError::BuiltinReassignment(name))
				}
                if self.globals.contains_key(name) {
                    return Err(CompileError::GlobalReassignment(name))
                }
                self.globals.insert(name, Type::Function(args.len() as u32));
                write!(self.buf, "fn func_{name}(")?;

                let mut locals = LocalTable::with_capacity(args.len());
                for (i, arg) in args.iter().enumerate() {
                    write!(self.buf, "arg_{}:vec2f,", i)?;
                    locals.insert(arg, i as u32);
                }
                write!(self.buf, ")->vec2f{{return ")?;
                self.compile_expr(&locals, expr)?;
                write!(self.buf, ";}}")?;
            }
        }
		writeln!(self.buf)?;
        Ok(())
    }

	///////////////////
	//  Expressions  //
	///////////////////

    fn compile_expr(&mut self, locals: &LocalTable<'s>, expr: &Expr<'s>) -> CompileResult<'s> {
        match expr {
            Expr::Number(z) => self.compile_number(*z),
            Expr::Name(name) => self.compile_var(locals, name),
            Expr::NameDeriv(name, _) => return Err(CompileError::StandaloneDerivative(name)),
            Expr::Unary(op, arg) => self.compile_unary(locals, *op, arg),
            Expr::Binary(op, lhs, rhs) => self.compile_binary(locals, *op, lhs, rhs),
            Expr::FnCall { name, args, nderiv } => self.compile_fncall(locals, name, args, *nderiv),
        }
    }

	fn compile_number(&mut self, z: Complex) -> CompileResult<'s> {
        write!(self.buf, "vec2f({:?},{:?})", z.re, z.im)?;
		Ok(())
	}

	fn compile_unary(&mut self, locals: &LocalTable<'s>, op: UnaryOp, arg: &Expr<'s>) -> CompileResult<'s> {
        let strings = unop_strings(op);
        write!(self.buf, "{}", strings[0])?;
        self.compile_expr(locals, arg)?;
        write!(self.buf, "{}", strings[1])?;
        Ok(())
	}

	fn compile_binary(&mut self, locals: &LocalTable<'s>, op: BinaryOp, lhs: &Expr<'s>, rhs: &Expr<'s>) -> CompileResult<'s> {
        let strings = binop_strings(op);
        write!(self.buf, "{}", strings[0])?;
        self.compile_expr(locals, lhs)?;
        write!(self.buf, "{}", strings[1])?;
        self.compile_expr(locals, rhs)?;
        write!(self.buf, "{}", strings[2])?;
        Ok(())
	}

	fn compile_var(&mut self, locals: &LocalTable<'s>, name: &'s str) -> CompileResult<'s> {
		let Some(name_info) = self.name_info(name, Some(locals)) else {
			return Err(CompileError::UndefinedVar(name))
		};
		self.compile_name(name, name_info)?;
		Ok(())
	}

	fn compile_fncall(&mut self, locals: &LocalTable<'s>, name: &'s str, args: &Vec<Expr<'s>>, nderiv: u32) -> CompileResult<'s> {
		let Some(name_info) = self.name_info(name, Some(locals)) else {
			return Err(CompileError::UndefinedVar(name))
		};
		let compname = &name_info.get_compiled_name();
		for i in 0..nderiv {
			self.add_gen_deriv(&compname, i+1, args.len() as u32);
			write!(self.buf, "deriv_")?;
		}
		self.compile_name_fncall(name, name_info, args.len() as u32)?;
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


	fn compile_name_fncall(&mut self, name: &'s str, name_info: NameInfo, argc: u32) -> CompileResult<'s> {
		match name_info {
			NameInfo::Local { .. } => return Err(CompileError::TypeError(name)),
			NameInfo::Global { ty: Type::Function(c), cname } if c == argc => {
				write!(self.buf, "{cname}")?;
			}
			NameInfo::Global { ty: Type::Function(_), .. } => return Err(CompileError::ArgCount(name)),
			NameInfo::Global { .. } => return Err(CompileError::TypeError(name)),
			NameInfo::Builtin { ty: Type::Function(c), bname } if c == argc => {
				write!(self.buf, "{bname}")?;
			}
			NameInfo::Builtin { ty: Type::Function(_), .. } => return Err(CompileError::ArgCount(name)),
			NameInfo::Builtin { .. } => return Err(CompileError::TypeError(name)),
			NameInfo::Generated { ty: Type::Function(c), gname, gen } if c == argc => {
				write!(self.buf, "{gname}")?;
				self.generate.insert(gname, gen);
			}
			NameInfo::Generated { ty: Type::Function(_), .. } => return Err(CompileError::ArgCount(name)),
			NameInfo::Generated { .. } => return Err(CompileError::TypeError(name)),
		}
		Ok(())
	}

	fn compile_name(&mut self, name: &str, name_info: NameInfo) -> CompileResult<'s> {
		match name_info {
			NameInfo::Local { id } => {
				write!(self.buf, "arg_{id}")?;
			},
			NameInfo::Global { ty: Type::Function(c), cname } => {
				self.add_gen_fid(format!("{cname}"), c);
				write!(self.buf, "FID_{cname}")?;
			},
			NameInfo::Global { ty: Type::Number, .. } => {
				write!(self.buf, "VAR_{name}")?;
			},
			NameInfo::Builtin { ty: Type::Function(c), bname } => {
				self.add_gen_fid(bname.to_owned(), c);
				write!(self.buf, "FID_{bname}")?;
			},
			NameInfo::Builtin { ty: Type::Number, bname } => {
				write!(self.buf, "{bname}")?;
			},
			NameInfo::Generated { ty: Type::Function(c), gname, gen } => {
				write!(self.buf, "FID_{gname}")?;
				self.add_gen_fid(gname.to_owned(), c);
				self.generate.insert(gname, gen);
			},
			NameInfo::Generated { ty: Type::Number, gname, gen } => {
				write!(self.buf, "{gname}")?;
				self.generate.insert(gname, gen);
			},
		}
		Ok(())
	}

	fn name_info(&self, name: &'s str, locals: Option<&LocalTable<'s>>) -> Option<NameInfo> {
		if let Some(locals) = locals {
			if let Some(id) = locals.get(name) {
				return Some(NameInfo::Local { id: *id })
			}
		}
		if let Some(ty) = self.globals.get(name) {
			return Some(NameInfo::Global { ty: *ty, cname: format!("func_{name}") })
		}
		if let Some((bname, ty, _)) = BUILTINS.with(|m| m.get(name).copied()) {
			return Some(NameInfo::Builtin { ty, bname })
		}
		if let Some(caps) = RE_INVOKE.with(|re| re.captures(name)) {
			if let Ok(n) = caps[1].parse() {
				return Some(NameInfo::Generated {
					ty: Type::Function(n + 1),
					gname: format!("invoke{n}"),
					gen: Generate::Invoke { argc: n },
				})
			}
		}
		if let Some(caps) = RE_ITER.with(|re| re.captures(name)) {
			if let Ok(n) = caps[1].parse() {
				return Some(NameInfo::Generated {
					ty: Type::Function(n + 2),
					gname: format!("iter{n}"),
					gen: Generate::Iter { argc: n }
				})
			}
		}
		None
	}

	//////////////////
	//  Generation  //
	//////////////////

	fn add_gen_fid(&mut self, name: String, argc: u32) {
		self.generate.insert(name, Generate::FunctionId { argc, id: self.next_fn_id });
		self.next_fn_id += 1;
	}

	fn add_gen_deriv(&mut self, name: &str, nderiv: u32, argc: u32) {
		let func_name = "deriv_".repeat(nderiv as usize - 1) + name;
		let deriv_name = "deriv_".repeat(nderiv as usize) + name;
		self.generate.insert(deriv_name, Generate::Derivative { func: func_name, nderiv, argc });
	}

	fn generate(&mut self) -> CompileResult<'s> {
		for (name, g) in self.generate.clone() {
			match g {
                Generate::FunctionId { id, .. } => {
					write!(self.buf, "const FID_{name}=vec2f({id}.0,0.0);")?;
				}
                Generate::Invoke { argc } => self.generate_invoke(argc)?,
                Generate::Iter { argc } => self.generate_iter(argc)?,
				Generate::Derivative { func, nderiv, argc } => self.generate_derivative(func, nderiv, argc)?,
            }
			writeln!(self.buf)?;
		}
		Ok(())
	}

	fn generate_invoke(&mut self, argc: u32) -> CompileResult<'s> {
		write!(self.buf, "fn invoke{argc}(func:vec2f,")?;
		for i in 0..argc {
			write!(self.buf, "arg_{i}:vec2f")?;
			write!(self.buf, ",")?;
		}
		write!(self.buf, ")->vec2f{{switch i32(func.x){{")?;
		for (name, g) in &self.generate {
			if let Generate::FunctionId { argc: fargc, id } = g {
				write!(self.buf, "case {id}")?;
				if *id == 0 {
					write!(self.buf, ",default")?;
				}
				write!(self.buf, "{{return {name}(")?;
				let a = argc.min(*fargc);
				for i in 0..a {
					write!(self.buf, "arg_{i},")?;
				}
				for _ in a..*fargc {
					write!(self.buf, "vec2f(0.0),")?;
				}
				write!(self.buf, ");}}")?;
			}
		}
		write!(self.buf, "}};}}")?;
		Ok(())
	}

	fn generate_iter(&mut self, argc: u32) -> CompileResult<'s> {
		if !self.generate.contains_key(&format!("invoke{argc}")) {
			self.generate.insert(format!("invoke{argc}"), Generate::Iter { argc });
			self.generate_invoke(argc)?;
			writeln!(self.buf)?;
		}
		write!(self.buf, "fn iter{argc}(func:vec2f,n:vec2f,")?;
		for i in 0..argc {
			write!(self.buf, "arg_{i}:vec2f")?;
			write!(self.buf, ",")?;
		}
		write!(self.buf, ")->vec2f{{var result=arg_0;")?;
		write!(self.buf, "for(var i=0;i<i32(n.x);i++){{result=invoke{argc}(func,result,")?;
		for i in 1..argc {
			write!(self.buf, "arg_{i},")?;
		}
		write!(self.buf, ");}}return result;}}")?;
		Ok(())
	}

	fn generate_derivative(&mut self, func: String, nderiv: u32, argc: u32) -> CompileResult<'s> {
		if let Some(f) = func.strip_suffix("deriv_") {
			if !self.generate.contains_key(f) {
				self.generate.insert(func.clone(), Generate::Derivative { func: f.to_owned(), nderiv: nderiv - 1, argc });
				self.generate_derivative(f.to_owned(), nderiv - 1, argc)?;
			}
		}
		write!(self.buf, "fn deriv_{func}(z:vec2f")?;
		let mut args = String::new();
		for i in 1..argc {
			write!(self.buf, ",arg{i}:vec2f")?;
			args += &format!(",arg{i}");
		}
		write!(self.buf, ")->vec2f{{")?;
		write!(self.buf, "\
			let a = c_mul({func}(z + vec2( D_EPS, 0.0){args}), vec2( 0.25/D_EPS, 0.0));\
			let b = c_mul({func}(z + vec2(-D_EPS, 0.0){args}), vec2(-0.25/D_EPS, 0.0));\
			let c = c_mul({func}(z + vec2(0.0,  D_EPS){args}), vec2(0.0, -0.25/D_EPS));\
			let d = c_mul({func}(z + vec2(0.0, -D_EPS){args}), vec2(0.0,  0.25/D_EPS));\
			return a + b + c + d;}}\
		")?;
		Ok(())
	}
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

use crate::complex::{Complex, cxfn};

use super::{parser::{Expr, UnaryOp, BinaryOp, Stmt}, compiler::{BUILTINS, Type}};

fn apply_unary(op: UnaryOp, arg: Complex) -> Complex {
	match op {
		UnaryOp::Pos => cxfn::pos(&[arg]),
		UnaryOp::Neg => cxfn::neg(&[arg]),
		UnaryOp::Recip => cxfn::recip(&[arg]),
	}
}

fn apply_binary(op: BinaryOp, u: Complex, v: Complex) -> Complex {
	match op {
		BinaryOp::Add => cxfn::add(&[u, v]),
		BinaryOp::Sub => cxfn::sub(&[u, v]),
		BinaryOp::Mul => cxfn::mul(&[u, v]),
		BinaryOp::Div => cxfn::div(&[u, v]),
		BinaryOp::Pow => cxfn::pow(&[u, v]),
	}
}

pub fn optimize<'s>(stmts: Vec<Stmt<'s>>) -> Vec<Stmt<'s>> {
	stmts.into_iter().map(|s| match s {
		Stmt::AssignConst(a, expr) => Stmt::AssignConst(a, optimize_expr(expr)),
		Stmt::AssignFunc(a, b, expr) => Stmt::AssignFunc(a, b, optimize_expr(expr)),
	}).collect()
}

fn optimize_expr<'s>(e: Expr<'s>) -> Expr<'s> {
	match e {
		Expr::Unary(op, arg) => {
			match optimize_expr(*arg) {
				Expr::Number(z) => Expr::Number(apply_unary(op, z)),
				e => Expr::Unary(op, Box::new(e)),
			}
		},
		Expr::Binary(op, lhs, rhs) => {
			match (optimize_expr(*lhs), optimize_expr(*rhs)) {
				(Expr::Number(u), Expr::Number(v)) => Expr::Number(apply_binary(op, u, v)),
				(u, v) => Expr::Binary(op, Box::new(u), Box::new(v))
			}
		},
		Expr::FnCall { name, args, nderiv } => {
			let args: Vec<Expr<'s>> = args.into_iter().map(|a| optimize_expr(a)).collect();
			if let Some((_, Type::Function(argc), Some(func))) = BUILTINS.with(|m| m.get(name).copied()) {
				if argc as usize == args.len() && nderiv == 0 {
					if args.iter().all(|e| matches!(e, Expr::Number(_))) {
						let ns: Vec<Complex> = args.into_iter().map(|a| match a { Expr::Number(n) => n, _ => unreachable!() }).collect();
						return Expr::Number(func(&ns))
					}
				}
			}
			Expr::FnCall { name, args, nderiv }
		}
		Expr::Name(name) => {
			if let Some((_, Type::Number, Some(func))) = BUILTINS.with(|m| m.get(name).copied()) {
				Expr::Number(func(&[]))
			} else {
				e
			}
		}
		_ => e,
	}
}
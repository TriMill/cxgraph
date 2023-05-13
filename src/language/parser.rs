use std::iter::Peekable;

use crate::complex::Complex;

use super::{scanner::{Scanner, Token}, Position};

#[derive(Clone, Debug)]
pub enum ParseError<'s> {
	UnexpectedTokenPrefix(Position, Token<'s>),
	Expected(Position, Token<'s>),
	InvalidLValue(Position),
	InvalidFunction(Expr<'s>),
	UnexpectedEof,
}
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
	Add, Sub, Mul, Div, Pow,
}

impl BinaryOp {
	pub const fn from_token(tok: Token) -> Option<Self> {
		match tok {
			Token::Plus => Some(Self::Add),
			Token::Minus => Some(Self::Sub),
			Token::Star => Some(Self::Mul),
			Token::Slash => Some(Self::Div),
			Token::Caret => Some(Self::Pow),
			_ => None,
		}
	}

	pub const fn precedence(self) -> (u32, u32) {
		match self {
			BinaryOp::Add => (10, 11),
			BinaryOp::Sub => (10, 11),
			BinaryOp::Mul => (20, 21),
			BinaryOp::Div => (20, 21),
			BinaryOp::Pow => (31, 30),
		}
	}

}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
	Pos, Neg, Recip,
}

impl UnaryOp {
	pub const fn from_token(tok: Token) -> Option<Self> {
		match tok {
			Token::Plus => Some(Self::Pos),
			Token::Minus => Some(Self::Neg),
			Token::Slash => Some(Self::Recip),
			_ => None,
		}
	}

	pub const fn precedence(self) -> u32 {
		40
	}
}

#[derive(Clone, Debug)]
pub enum Expr<'s> {
	Number(Complex),
	Name(&'s str),
	NameDeriv(&'s str, u32),
	Unary(UnaryOp, Box<Expr<'s>>),
	Binary(BinaryOp, Box<Expr<'s>>, Box<Expr<'s>>),
	FnCall { name: &'s str, args: Vec<Expr<'s>>, nderiv: u32 },
}

#[derive(Debug)]
pub enum Stmt<'s> {
	AssignConst(&'s str, Expr<'s>),
	AssignFunc(&'s str, Vec<&'s str>, Expr<'s>),
}

pub struct Parser<'s> {
	scanner: Peekable<Scanner<'s>>,
}

impl <'s> Parser<'s> {
	pub fn new(src: &'s str) -> Self {
		Self {
			scanner: Scanner::new(src).peekable()
		}
	}

	fn expect(&mut self, tok: Token<'s>) -> Result<Position, ParseError<'s>> {
		match self.scanner.peek() {
			Some((p, t)) if *t == tok => {
				let p = *p;
				self.scanner.next();
				Ok(p)
			},
			Some((p, _)) => Err(ParseError::Expected(*p, tok)),
			None => Err(ParseError::UnexpectedEof),
		}
	}

	fn expr(&mut self, min_prec: u32) -> Result<Expr<'s>, ParseError<'s>> {
		let (pos, tok) = self.scanner.next().unwrap();
		let mut expr = match tok {
			Token::Number(n) => Expr::Number(Complex::from(n)),
			Token::Name(n) => Expr::Name(n),
			Token::LParen => {
				let e = self.expr(0)?;
				self.expect(Token::RParen)?;
				e
			}
			tok => if let Some(op) = UnaryOp::from_token(tok) {
				Expr::Unary(op, Box::new(self.expr(op.precedence())?))
			} else {
				return Err(ParseError::UnexpectedTokenPrefix(pos, tok))
			}
		};

		while let Some((_, tok)) = self.scanner.peek() {
			expr = match tok {
				Token::Equal | Token::RParen | Token::Newline | Token::Comma => break,
				Token::Quote => {
					self.scanner.next();
					match expr {
						Expr::Name(name) => Expr::NameDeriv(name, 1),
						Expr::NameDeriv(name, nderiv) => Expr::NameDeriv(name, nderiv+1),
						_ => return Err(ParseError::InvalidFunction(expr))
					}
				},
				Token::LParen => {
					self.scanner.next();
					let mut args = Vec::new();
					while !matches!(self.scanner.peek(), None | Some((_, Token::RParen))) {
						args.push(self.expr(0)?);
						match self.scanner.peek() {
							Some((_, Token::Comma)) => { self.scanner.next(); },
							Some((_, Token::RParen)) => break,
							Some((pos, _)) => return Err(ParseError::Expected(*pos, Token::RParen)),
							None => return Err(ParseError::UnexpectedEof),
						}
					}
					self.expect(Token::RParen)?;
					match expr {
						Expr::Name(name) => Expr::FnCall { name, args, nderiv: 0 },
						Expr::NameDeriv(name, nderiv) => Expr::FnCall { name, args, nderiv },
						_ => return Err(ParseError::InvalidFunction(expr)),
					}
				},
				tok => if let Some(op) = BinaryOp::from_token(*tok) {
					let (lp, rp) = op.precedence();
					if lp < min_prec {
						break;
					}
					self.scanner.next();
					let rhs = self.expr(rp)?;
					Expr::Binary(op, Box::new(expr), Box::new(rhs))
				} else {
					let (lp, rp) = BinaryOp::Mul.precedence();
					if lp < min_prec {
						break;
					}
					let rhs = self.expr(rp)?;
					Expr::Binary(BinaryOp::Mul, Box::new(expr), Box::new(rhs))
				}
			}
		}

		Ok(expr)
	}

	pub fn parse(&mut self) -> Result<Vec<Stmt<'s>>, ParseError<'s>> {
		let mut stmts = Vec::new();
		while self.scanner.peek().is_some() {
			while matches!(self.scanner.peek(), Some((_, Token::Newline))) {
				self.scanner.next();
			}

			if self.scanner.peek().is_none() {
				break;
			}

			let lhs_pos = self.scanner.peek().unwrap().0;
			let lhs = self.expr(0)?;

			self.expect(Token::Equal)?;

			if self.scanner.peek().is_none() {
				return Err(ParseError::UnexpectedEof)
			}

			let rhs = self.expr(0)?;

			if self.scanner.peek().is_some() {
				self.expect(Token::Newline)?;
			}

			let stmt = match lhs {
				Expr::Name(name) => Stmt::AssignConst(name, rhs),
				Expr::FnCall { name, args, nderiv: 0 } => {
					let mut arg_names = Vec::with_capacity(args.len());
					for arg in args {
						if let Expr::Name(name) = arg {
							arg_names.push(name)
						}
					}
					Stmt::AssignFunc(name, arg_names, rhs)
				}
				_ => return Err(ParseError::InvalidLValue(lhs_pos))
			};

			stmts.push(stmt);
		}
		Ok(stmts)
	}
}

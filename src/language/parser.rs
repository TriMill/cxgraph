use std::iter::Peekable;

use crate::complex::Complex;

use super::{scanner::{Scanner, Token}, Position};

#[derive(Clone, Debug)]
pub struct ParseError {
	msg: String,
	pos: Option<Position>,
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
			BinaryOp::Add
			| BinaryOp::Sub => (10, 11),
			BinaryOp::Mul
			| BinaryOp::Div => (20, 21),
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
	Unary(UnaryOp, Box<Expr<'s>>),
	Binary(BinaryOp, Box<Expr<'s>>, Box<Expr<'s>>),
	FnCall(&'s str, Vec<Expr<'s>>),
}

#[derive(Debug)]
pub enum Stmt<'s> {
	Expr(Expr<'s>),
	Store(&'s str, Expr<'s>),
	Iter(&'s str, u32, u32, Vec<Stmt<'s>>),
}

pub enum Defn<'s> {
	Const { name: &'s str, body: Expr<'s> },
	Func { name: &'s str, args: Vec<&'s str>, body: (Vec<Stmt<'s>>, Expr<'s>) },
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

	fn next(&mut self) -> Result<(Position, Token<'s>), ParseError> {
		match self.scanner.next() {
			Some(r) => Ok(r),
			None => Err(self.err_here("Unexpected EOF")),
		}
	}

	fn peek(&mut self) -> Result<(Position, Token<'s>), ParseError> {
		match self.scanner.peek() {
			Some(r) => Ok(*r),
			None => Err(self.err_here("Unexpected EOF")),
		}
	}

	fn at_end(&mut self) -> bool {
		self.scanner.peek().is_none()
	}

	fn expect(&mut self, tok: Token<'s>) -> Result<Position, ParseError> {
		match self.peek()? {
			(p, t) if t == tok => {
				self.next()?;
				Ok(p)
			},
			(p, t) => Err(self.err_at(format!("Unexpected token {t:?}, expected {tok:?}"), p)),
		}
	}

	fn err_here<S>(&mut self, msg: S) -> ParseError
	where S: Into<String> {
		ParseError { msg: msg.into(), pos: self.peek().map(|(p,_)| p).ok() }
	}

	fn err_at<S>(&mut self, msg: S, pos: Position) -> ParseError
	where S: Into<String> {
		ParseError { msg: msg.into(), pos: Some(pos) }
	}

	fn expr(&mut self, min_prec: u32) -> Result<Expr<'s>, ParseError> {
		let (pos, tok) = self.next()?;
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
				return Err(self.err_at(format!("Unexpected token {:?}", tok), pos))
			}
		};

		while let Ok((_, tok)) = self.peek() {
			if is_closing(&tok) {
				break;
			}
			expr = match tok {
				Token::LParen => {
					self.next()?;
					let mut args = Vec::new();
					while !matches!(self.peek(), Err(_) | Ok((_, Token::RParen))) {
						args.push(self.expr(0)?);
						match self.peek()?.1 {
							Token::Comma => { self.next()?; },
							Token::RParen => break,
							_ => return Err(self.err_here(format!("Unexpected token {:?}, expected ',' or ')'", tok)))
						}
					}
					self.expect(Token::RParen)?;
					match expr {
						Expr::Name(name) => Expr::FnCall(name, args),
						_ => return Err(self.err_here("Cannot call this expression"))
					}
				},
				tok => if let Some(op) = BinaryOp::from_token(tok) {
					let (lp, rp) = op.precedence();
					if lp < min_prec {
						break;
					}
					self.next()?;
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

	fn stmt(&mut self) -> Result<Stmt<'s>, ParseError> {
		let expr = self.expr(0)?;
		match self.peek()?.1 {
			Token::Arrow => {
				self.next()?;
				let name = match self.next()? {
					(_, Token::Name(name)) => name,
					(p, t) => return Err(self.err_at(format!("Unexpected token {t:?}, expected a name"), p))
				};
				Ok(Stmt::Store(name, expr))
			}
			_ => Ok(Stmt::Expr(expr))
		}
	}

	fn stmts(&mut self) -> Result<Vec<Stmt<'s>>, ParseError> {
		let mut stmts = Vec::new();
		loop {
			stmts.push(self.stmt()?);
			if !matches!(self.peek(), Ok((_, Token::Comma))) {
				break
			}
			self.next()?;
		}
		Ok(stmts)
	}

	pub fn parse(&mut self) -> Result<Vec<Defn<'s>>, ParseError> {
		println!("parse");
		let mut defns = Vec::new();
		while self.peek().is_ok() {
			println!("parse loop");
			while matches!(self.peek(), Ok((_, Token::Newline))) {
				self.next()?;
			}

			if self.peek().is_err() {
				break;
			}

			let lhspos = self.peek()?.0;
			let lhs = self.expr(0)?;

			self.expect(Token::Equal)?;

			let defn = match lhs {
				Expr::Name(name) => {
					let rhs = self.expr(0)?;
					Defn::Const { name, body: rhs }
				},
				Expr::FnCall(name, args) => {
					let mut rhs = self.stmts()?;
					let last = rhs.pop().ok_or(self.err_here("Empty function body"))?;
					let Stmt::Expr(last) = last else {
						return Err(self.err_here("Last statement in function body must be a plain expression"))
					};
					let args = args.iter()
						.map(|a| match a {
							Expr::Name(n) => Ok(*n),
							_ => Err(self.err_at("Invalid function declaration", lhspos))
						}).collect::<Result<Vec<&str>, ParseError>>()?;
					Defn::Func { name, args, body: (rhs, last) }
				},
				_ => return Err(self.err_at("Invalid lvalue, expected a name or function call", lhspos)),
			};

			defns.push(defn);

			if self.at_end() {
				break
			}
			self.expect(Token::Newline)?;
		}
		Ok(defns)
	}
}

fn is_closing(tok: &Token) -> bool {
	matches!(tok, Token::Equal | Token::RParen | Token::Newline | Token::Comma | Token::Arrow)
}

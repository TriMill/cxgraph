use std::{iter::Peekable, str::Chars};

use super::Position;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token<'s> {
	Error,
	Name(&'s str),
	Number(f64),
	Plus,
	Minus,
	Star,
	Slash,
	Caret,
	Equal,
	Comma,
	Colon,
	LParen,
	RParen,
	Newline,
}

pub struct Scanner<'s> {
	pub src: &'s str,
	pub chars: Peekable<Chars<'s>>,
	pub pos: Position,
}

impl <'s> Scanner<'s> {
	pub fn new(src: &'s str) -> Self {
		Self {
			src,
			chars: src.chars().peekable(),
			pos: Position { pos: 0, line: 1, col: 1 }
		}
	}

	fn next(&mut self) -> Option<char> {
		match self.chars.next() {
			Some('\n') => {
				self.pos.pos += 1;
				self.pos.line += 1;
				self.pos.col = 1;
				Some('\n')
			},
			Some(c) => {
				self.pos.pos += 1;
				self.pos.col += 1;
				Some(c)
			},
			None => None,
		}
	}

	fn peek(&mut self) -> Option<char> {
		self.chars.peek().copied()
	}

	fn next_number(&mut self, pos: u32) -> Token<'s> {
		while matches!(self.peek(), Some('0'..='9')) {
			self.next();
		}
		if matches!(self.peek(), Some('.')) {
			self.next();
			while matches!(self.peek(), Some('0'..='9')) {
				self.next();
			}
		}
		let s = &self.src[pos as usize..self.pos.pos as usize];
		match s.parse() {
			Ok(x) => Token::Number(x),
			Err(_) => Token::Error,
		}
	}

	fn next_name(&mut self, pos: u32) -> Token<'s> {
		while matches!(self.peek(), Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
			self.next();
		}
		let s = &self.src[pos as usize..self.pos.pos as usize];
		Token::Name(s)
	}

	pub fn next_token(&mut self) -> Option<(Position, Token<'s>)> {
		while matches!(self.peek(), Some(' ' | '\t')) {
			self.next();
		}
		self.peek()?;
		let pos = self.pos;
		let tok = match self.next().unwrap() {
			'\n' => Token::Newline,
			'+' => Token::Plus,
			'-' => Token::Minus,
			'*' => Token::Star,
			'/' => Token::Slash,
			'^' => Token::Caret,
			':' => Token::Colon,
			'=' => Token::Equal,
			',' => Token::Comma,
			'(' => Token::LParen,
			')' => Token::RParen,
			'0'..='9' => self.next_number(pos.pos),
			'a'..='z' | 'A'..='Z' => self.next_name(pos.pos),
			_ => Token::Error,
		};
		Some((pos, tok))
	}
}

impl <'s> Iterator for Scanner<'s> {
    type Item = (Position, Token<'s>);

    fn next(&mut self) -> Option<Self::Item> {
		self.next_token()
    }
}

use std::{str::CharIndices, iter::Peekable, fmt};

use unicode_xid::UnicodeXID;


#[derive(Clone, Copy, Debug)]
pub enum Token<'i> {
	Number(f64),
	Name(&'i str),
	Sum, Prod, Iter, If, While,
	LParen, RParen,
	LBrace, RBrace,
	Plus, Minus, Star, Slash, Caret,
	Greater, Less, GreaterEqual, LessEqual,
	EqualEqual, BangEqual,
	Comma, Arrow, Equal, Colon,
	Newline,
}

impl<'i> fmt::Display for Token<'i> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Token::Number(n) => write!(f, "{n}"),
			Token::Name(n) => write!(f, "{n}"),
			Token::Sum     => f.write_str("sum"),
			Token::Prod    => f.write_str("prod"),
			Token::Iter    => f.write_str("iter"),
			Token::If      => f.write_str("if"),
			Token::While   => f.write_str("while"),
			Token::LParen  => f.write_str("("),
			Token::RParen  => f.write_str(")"),
			Token::LBrace  => f.write_str("{"),
			Token::RBrace  => f.write_str("}"),
			Token::Plus    => f.write_str("+"),
			Token::Minus   => f.write_str("-"),
			Token::Star    => f.write_str("*"),
			Token::Slash   => f.write_str("/"),
			Token::Caret   => f.write_str("^"),
			Token::Comma   => f.write_str(","),
			Token::Arrow   => f.write_str("->"),
			Token::Equal   => f.write_str("="),
			Token::Colon   => f.write_str(":"),
			Token::Greater      => f.write_str(">"),
			Token::Less         => f.write_str("<"),
			Token::GreaterEqual => f.write_str(">="),
			Token::LessEqual    => f.write_str("<="),
			Token::EqualEqual   => f.write_str("=="),
			Token::BangEqual    => f.write_str("!="),
			Token::Newline => f.write_str("newline")
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum LexerError {
	Unexpected(usize, char),
	UnexpectedEof,
	InvalidNumber(usize, usize),
}

impl fmt::Display for LexerError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LexerError::UnexpectedEof => write!(f, "Unexpected EOF during lexing"),
			LexerError::Unexpected(i, c) => write!(f, "Unexpected character {c:?} at {i}"),
			LexerError::InvalidNumber(i, j) => write!(f, "Invalid number at {i}:{j}"),
		}
	}
}

pub type Spanned<T, L, E> = Result<(L, T, L), E>;

pub struct Lexer<'i> {
	src: &'i str,
	chars: Peekable<CharIndices<'i>>,
	bracket_depth: isize,
}

fn is_ident_begin(c: char) -> bool {
	c.is_xid_start()
}

fn is_ident_middle(c: char) -> bool {
	c.is_xid_continue() || matches!(c, '\'' | '\u{2080}'..='\u{2089}')
}

impl<'i> Lexer<'i> {
	pub fn new(src: &'i str) -> Self {
		Self {
			src,
			chars: src.char_indices().peekable(),
			bracket_depth: 0,
	 }
	}

	fn next_number(&mut self, i: usize, has_dot: bool) -> Spanned<Token<'i>, usize, LexerError> {
		let mut j = i;

		while self.chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
			j = self.chars.next().unwrap().0;
		}

		if !has_dot && matches!(self.chars.peek(), Some((_, '.'))) {
			j = self.chars.next().unwrap().0;
			while self.chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
				j = self.chars.next().unwrap().0;
			}
		}

		let s = &self.src[i..j+1];
		match s.parse::<f64>() {
			Ok(n) => Ok((i, Token::Number(n), j+1)),
			Err(_) => Err(LexerError::InvalidNumber(i, j+1)),
		}
	}

	fn next_word(&mut self, i: usize, mut j: usize) -> Spanned<Token<'i>, usize, LexerError> {
		while self.chars.peek().is_some_and(|(_, c)| is_ident_middle(*c)) {
			j += self.chars.next().unwrap().1.len_utf8();
		}

		let s = &self.src[i..j];
		match s {
			"sum"     => Ok((i, Token::Sum,     j)),
			"prod"    => Ok((i, Token::Prod,    j)),
			"iter"    => Ok((i, Token::Iter,    j)),
			"if"      => Ok((i, Token::If,      j)),
			"while"   => Ok((i, Token::While,   j)),
			_ => Ok((i, Token::Name(s), j)),
		}
	}

	fn skip_whitespace(&mut self) {
		while matches!(self.chars.peek(), Some((_, ' ' | '\t' | '\n' | '\r'))) {
			if self.bracket_depth == 0 && matches!(self.chars.peek(), Some((_, '\n'))) {
				break
			}
			self.chars.next();
		}
	}

	fn next_token(&mut self) -> Option<Spanned<Token<'i>, usize, LexerError>> {
		self.skip_whitespace();

		Some(match self.chars.next()? {
			(_, '#') => {
				while !matches!(self.chars.peek(), Some((_, '\n')) | None) {
					self.chars.next();
				}
				self.next_token()?
			}

			(i, '\n') => Ok((i, Token::Newline, i + 1)),

			(i, '(') => { self.bracket_depth += 1; Ok((i, Token::LParen, i + 1)) },
			(i, ')') => { self.bracket_depth -= 1; Ok((i, Token::RParen, i + 1)) },
			(i, '{') => { self.bracket_depth += 1; Ok((i, Token::LBrace, i + 1)) },
			(i, '}') => { self.bracket_depth -= 1; Ok((i, Token::RBrace, i + 1)) },

			(i, '+') => Ok((i, Token::Plus, i + 1)),
			(i, '-') => match self.chars.next_if(|(_, c)| *c == '>') {
				Some(_) => Ok((i, Token::Arrow, i + 2)),
				_ => Ok((i, Token::Minus, i + 1)),
			},
			(i, '*') => Ok((i, Token::Star, i + 1)),
			(i, '\u{22C5}') => Ok((i, Token::Star, i + '\u{22C5}'.len_utf8())),
			(i, '/') => Ok((i, Token::Slash, i + 1)),
			(i, '^') => Ok((i, Token::Caret, i + 1)),

			(i, '<') => match self.chars.next_if(|(_, c)| *c == '=') {
				Some(_) => Ok((i, Token::LessEqual, i + 2)),
				_ => Ok((i, Token::Less, i + 1)),
			},
			(i, '\u{2264}') => Ok((i, Token::LessEqual, i + '\u{2264}'.len_utf8())),
			(i, '>') => match self.chars.next_if(|(_, c)| *c == '=') {
				Some(_) => Ok((i, Token::GreaterEqual, i + 2)),
				_ => Ok((i, Token::Greater, i + 1)),
			},
			(i, '\u{2265}') => Ok((i, Token::GreaterEqual, i + '\u{2265}'.len_utf8())),
			(i, '=') => match self.chars.next_if(|(_, c)| *c == '=') {
				Some(_) => Ok((i, Token::EqualEqual, i + 2)),
				_ => Ok((i, Token::Equal, i + 1)),
			}
			(i, '!') => match self.chars.next() {
				Some((_, '=')) => Ok((i, Token::BangEqual, i + 2)),
				Some((_, c)) => Err(LexerError::Unexpected(i+1, c)),
				None => Err(LexerError::UnexpectedEof),
			}
			(i, '\u{2260}') => Ok((i, Token::BangEqual, i + '\u{2260}'.len_utf8())),

			(i, ',') => Ok((i, Token::Comma, i + 1)),
			(i, ':') => Ok((i, Token::Colon, i + 1)),

			(i, '0'..='9') => self.next_number(i, false),
			(i, '.') => self.next_number(i, true),
			(i, c) if is_ident_begin(c) => self.next_word(i, i + c.len_utf8()),
			(i, c) => Err(LexerError::Unexpected(i, c)),
		})
	}
}

impl<'i> Iterator for Lexer<'i> {
	type Item = Spanned<Token<'i>, usize, LexerError>;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_token()
	}
}

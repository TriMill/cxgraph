use std::{str::CharIndices, iter::Peekable, fmt};


#[derive(Clone, Copy, Debug)]
pub enum Token<'i> {
	Float(f64),
	Int(i32),
	Name(&'i str),
	Sum, Prod, Iter,
    LParen, RParen,
	LBrace, RBrace,
	Plus, Minus, Star, Slash, Caret,
	Comma, Arrow, Equal, Colon,
	Newline,
}

impl<'i> fmt::Display for Token<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Float(n) => write!(f, "{n}"),
            Token::Int(n) => write!(f, "{n}"),
            Token::Name(n) => write!(f, "{n}"),
			Token::Sum    => f.write_str("sum"),
			Token::Prod   => f.write_str("prod"),
			Token::Iter   => f.write_str("iter"),
            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::LBrace => f.write_str("{{"),
            Token::RBrace => f.write_str("}}"),
            Token::Plus   => f.write_str("+"),
            Token::Minus  => f.write_str("-"),
            Token::Star   => f.write_str("*"),
            Token::Slash  => f.write_str("/"),
            Token::Caret  => f.write_str("^"),
            Token::Comma  => f.write_str(","),
            Token::Arrow  => f.write_str("->"),
            Token::Equal  => f.write_str("="),
            Token::Colon  => f.write_str(":"),
            Token::Newline => f.write_str("newline")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LexerError {
	Unexpected(usize, char),
	InvalidNumber(usize, usize),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::Unexpected(i, c) => write!(f, "Unexpected character {c:?} at {i}"),
            LexerError::InvalidNumber(i, j) => write!(f, "Invalid number at {i}:{j}"),
        }
    }
}

pub type Spanned<T, L, E> = Result<(L, T, L), E>;

pub struct Lexer<'i> {
	src: &'i str,
	chars: Peekable<CharIndices<'i>>,
}

fn is_ident_begin(c: char) -> bool {
	c.is_alphabetic()
}

fn is_ident_middle(c: char) -> bool {
	c.is_alphanumeric() || c == '_' || c == '\''
}

impl<'i> Lexer<'i> {
	pub fn new(src: &'i str) -> Self {
		Self { src, chars: src.char_indices().peekable() }
	}

	fn next_number(&mut self, i: usize, mut has_dot: bool) -> Spanned<Token<'i>, usize, LexerError> {
		let mut j = i;

		while self.chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
			j = self.chars.next().unwrap().0;
		}

		if !has_dot && matches!(self.chars.peek(), Some((_, '.'))) {
			j = self.chars.next().unwrap().0;
			has_dot = true;
			while self.chars.peek().is_some_and(|(_, c)| c.is_ascii_digit()) {
				j = self.chars.next().unwrap().0;
			}
		}

		let s = &self.src[i..j+1];
		if !has_dot {
			if let Ok(n) = s.parse::<i32>() {
				return Ok((i, Token::Int(n), j+1))
			}
		}
		match s.parse::<f64>() {
			Ok(n) => Ok((i, Token::Float(n), j+1)),
			Err(_) => Err(LexerError::InvalidNumber(i, j+1)),
		}
	}

	fn next_word(&mut self, i: usize, mut j: usize) -> Spanned<Token<'i>, usize, LexerError> {
		while self.chars.peek().is_some_and(|(_, c)| is_ident_middle(*c)) {
			j += self.chars.next().unwrap().1.len_utf8();
		}

		let s = &self.src[i..j];
		match s {
			"sum"   => Ok((i, Token::Sum, j)),
			"prod"  => Ok((i, Token::Prod, j)),
			"iter"  => Ok((i, Token::Iter, j)),
			_ => Ok((i, Token::Name(s), j)),
		}
	}

	fn next_token(&mut self) -> Option<Spanned<Token<'i>, usize, LexerError>> {
		while matches!(self.chars.peek(), Some((_, ' ' | '\t' | '\r'))) {
			self.chars.next();
		}

		Some(match self.chars.next()? {
			(i, '(') => Ok((i, Token::LParen, i + 1)),
			(i, ')') => Ok((i, Token::RParen, i + 1)),
			(i, '{') => Ok((i, Token::LBrace, i + 1)),
			(i, '}') => Ok((i, Token::RBrace, i + 1)),
			(i, '+') => Ok((i, Token::Plus, i + 1)),
			(i, '-') => match self.chars.peek() {
				Some((_, '>')) => {
					self.chars.next();
					Ok((i, Token::Arrow, i + 2))
				},
				_ => Ok((i, Token::Minus, i + 1)),
			}
			(i, '*') => Ok((i, Token::Star, i + 1)),
			(i, '/') => Ok((i, Token::Slash, i + 1)),
			(i, '^') => Ok((i, Token::Caret, i + 1)),
			(i, ',') => Ok((i, Token::Comma, i + 1)),
			(i, '=') => Ok((i, Token::Equal, i + 1)),
			(i, ':') => Ok((i, Token::Colon, i + 1)),
			(i, '\n') => Ok((i, Token::Newline, i + 1)),
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

//! Bounded lexer. Every byte is visited once and charged to the admission meter
//! by the parser.

use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionOccurrence, ExpressionResult, SyntaxDenial,
};

use super::literal::DecimalText;
use super::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Identifier(Box<str>),
    Integer(u128),
    Float(DecimalText),
    String(Box<str>),
    True,
    False,
    Let,
    None,
    Punct(Punct),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Punct {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    PathSeparator,
    Semicolon,
    Dot,
    Bang,
    Minus,
    Plus,
    Star,
    Slash,
    Percent,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Assign,
    AndAnd,
    OrOr,
    Coalesce,
    Question,
}

pub(crate) struct Lexer<'source> {
    source: &'source str,
    bytes: &'source [u8],
    position: usize,
}

fn denial(detail: SyntaxDenial, start: usize, end: usize) -> ExpressionDenial {
    ExpressionDenial::at(
        ExpressionDenialDetail::Syntax(detail),
        ExpressionOccurrence::Source(SourceSpan::new(start as u32, end as u32)),
    )
}

impl<'source> Lexer<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            position: 0,
        }
    }

    pub(crate) fn next_token(&mut self) -> ExpressionResult<(Token, SourceSpan)> {
        self.skip_trivia();
        let start = self.position;
        let Some(&byte) = self.bytes.get(start) else {
            let end = SourceSpan::new(start as u32, start as u32);
            return Ok((Token::End, end));
        };
        let token = match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),
            b'0'..=b'9' => self.number()?,
            b'"' => self.string()?,
            _ => Token::Punct(self.punct()?),
        };
        Ok((token, SourceSpan::new(start as u32, self.position as u32)))
    }

    fn skip_trivia(&mut self) {
        while let Some(&byte) = self.bytes.get(self.position) {
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => self.position += 1,
                b'/' if self.bytes.get(self.position + 1) == Some(&b'/') => {
                    while let Some(&byte) = self.bytes.get(self.position) {
                        if byte == b'\n' {
                            break;
                        }
                        self.position += 1;
                    }
                }
                _ => return,
            }
        }
    }

    fn identifier(&mut self) -> Token {
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            self.position += 1;
        }
        match &self.source[start..self.position] {
            "true" => Token::True,
            "false" => Token::False,
            "let" => Token::Let,
            "none" => Token::None,
            text => Token::Identifier(text.into()),
        }
    }

    fn number(&mut self) -> ExpressionResult<Token> {
        let start = self.position;
        if self.bytes[start] == b'0' && matches!(self.bytes.get(start + 1), Some(b'x' | b'X')) {
            self.position += 2;
            let digits_start = self.position;
            self.take_while(|byte| byte.is_ascii_hexdigit());
            return self.integer(digits_start, 16, start);
        }
        self.take_while(|byte| byte.is_ascii_digit());
        let mut is_float = false;
        if self.bytes.get(self.position) == Some(&b'.')
            && self
                .bytes
                .get(self.position + 1)
                .is_some_and(u8::is_ascii_digit)
        {
            is_float = true;
            self.position += 1;
            self.take_while(|byte| byte.is_ascii_digit());
        }
        if matches!(self.bytes.get(self.position), Some(b'e' | b'E')) {
            is_float = true;
            self.position += 1;
            if matches!(self.bytes.get(self.position), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let exponent_start = self.position;
            self.take_while(|byte| byte.is_ascii_digit());
            if exponent_start == self.position {
                return Err(denial(SyntaxDenial::MalformedNumber, start, self.position));
            }
        }
        if self
            .bytes
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            return Err(denial(
                SyntaxDenial::MalformedNumber,
                start,
                self.position + 1,
            ));
        }
        if !is_float {
            return self.integer(start, 10, start);
        }
        DecimalText::parse(&self.source[start..self.position])
            .map(Token::Float)
            .ok_or_else(|| {
                ExpressionDenial::at(
                    ExpressionDenialDetail::InvalidValue("float literal exponent out of range"),
                    ExpressionOccurrence::Source(SourceSpan::new(
                        start as u32,
                        self.position as u32,
                    )),
                )
            })
    }

    fn integer(
        &mut self,
        digits_start: usize,
        radix: u32,
        start: usize,
    ) -> ExpressionResult<Token> {
        let digits = &self.source[digits_start..self.position];
        let malformed = || denial(SyntaxDenial::MalformedNumber, start, self.position);
        if digits.is_empty()
            || self
                .bytes
                .get(self.position)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            return Err(malformed());
        }
        let mut value: u128 = 0;
        for digit in digits.bytes() {
            let digit = char::from(digit).to_digit(radix).ok_or_else(malformed)?;
            value = value
                .checked_mul(u128::from(radix))
                .and_then(|value| value.checked_add(u128::from(digit)))
                .ok_or_else(|| {
                    ExpressionDenial::at(
                        ExpressionDenialDetail::InvalidValue("integer literal out of range"),
                        ExpressionOccurrence::Source(SourceSpan::new(
                            start as u32,
                            self.position as u32,
                        )),
                    )
                })?;
        }
        Ok(Token::Integer(value))
    }

    fn take_while(&mut self, accept: impl Fn(&u8) -> bool) {
        while self.bytes.get(self.position).is_some_and(&accept) {
            self.position += 1;
        }
    }

    fn string(&mut self) -> ExpressionResult<Token> {
        let start = self.position;
        self.position += 1;
        let mut text = String::new();
        loop {
            let rest = &self.source[self.position..];
            let Some(character) = rest.chars().next() else {
                return Err(denial(
                    SyntaxDenial::UnterminatedString,
                    start,
                    self.position,
                ));
            };
            self.position += character.len_utf8();
            match character {
                '"' => return Ok(Token::String(text.into_boxed_str())),
                '\\' => text.push(self.escape(start)?),
                other => text.push(other),
            }
        }
    }

    fn escape(&mut self, start: usize) -> ExpressionResult<char> {
        let escape_start = self.position - 1;
        let invalid = |end| denial(SyntaxDenial::InvalidEscape, escape_start, end);
        let Some(&byte) = self.bytes.get(self.position) else {
            return Err(denial(
                SyntaxDenial::UnterminatedString,
                start,
                self.position,
            ));
        };
        self.position += 1;
        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => {
                if self.bytes.get(self.position) != Some(&b'{') {
                    return Err(invalid(self.position));
                }
                self.position += 1;
                let digits_start = self.position;
                self.take_while(|byte| byte.is_ascii_hexdigit());
                let digits = &self.source[digits_start..self.position];
                if digits.is_empty()
                    || digits.len() > 6
                    || self.bytes.get(self.position) != Some(&b'}')
                {
                    return Err(invalid(self.position));
                }
                self.position += 1;
                let scalar = u32::from_str_radix(digits, 16).map_err(|_| invalid(self.position))?;
                char::from_u32(scalar).ok_or_else(|| {
                    denial(
                        SyntaxDenial::InvalidUnicodeScalar(scalar),
                        escape_start,
                        self.position,
                    )
                })
            }
            _ => Err(invalid(self.position)),
        }
    }

    fn punct(&mut self) -> ExpressionResult<Punct> {
        let start = self.position;
        let first = self.bytes[start];
        let second = self.bytes.get(start + 1).copied();
        let (punct, width) = match (first, second) {
            (b':', Some(b':')) => (Punct::PathSeparator, 2),
            (b'<', Some(b'=')) => (Punct::LessEqual, 2),
            (b'>', Some(b'=')) => (Punct::GreaterEqual, 2),
            (b'=', Some(b'=')) => (Punct::EqualEqual, 2),
            (b'!', Some(b'=')) => (Punct::NotEqual, 2),
            (b'&', Some(b'&')) => (Punct::AndAnd, 2),
            (b'|', Some(b'|')) => (Punct::OrOr, 2),
            (b'?', Some(b'?')) => (Punct::Coalesce, 2),
            (b'(', _) => (Punct::LeftParen, 1),
            (b')', _) => (Punct::RightParen, 1),
            (b'[', _) => (Punct::LeftBracket, 1),
            (b']', _) => (Punct::RightBracket, 1),
            (b'{', _) => (Punct::LeftBrace, 1),
            (b'}', _) => (Punct::RightBrace, 1),
            (b',', _) => (Punct::Comma, 1),
            (b':', _) => (Punct::Colon, 1),
            (b';', _) => (Punct::Semicolon, 1),
            (b'.', _) => (Punct::Dot, 1),
            (b'!', _) => (Punct::Bang, 1),
            (b'-', _) => (Punct::Minus, 1),
            (b'+', _) => (Punct::Plus, 1),
            (b'*', _) => (Punct::Star, 1),
            (b'/', _) => (Punct::Slash, 1),
            (b'%', _) => (Punct::Percent, 1),
            (b'<', _) => (Punct::Less, 1),
            (b'>', _) => (Punct::Greater, 1),
            (b'=', _) => (Punct::Assign, 1),
            (b'?', _) => (Punct::Question, 1),
            _ => {
                let character = self.source[start..].chars().next().unwrap_or('\u{fffd}');
                let end = start + character.len_utf8();
                return Err(denial(
                    SyntaxDenial::UnexpectedCharacter(character),
                    start,
                    end,
                ));
            }
        };
        self.position += width;
        Ok(punct)
    }
}

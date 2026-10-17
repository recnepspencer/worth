use super::objects::{Object, Objects};
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};
impl Objects {
    pub(super) fn invoke(
        &mut self,
        op: u32,
        a: u32,
        b: u32,
        _c: u32,
    ) -> Result<u32, super::Failure> {
        // Charge before native recursive formatting/traversal or construction.
        if matches!(op, 2 | 3 | 36 | 37 | 40 | 42) {
            self.charge(if op == 40 { b } else { a })?;
        }
        let carried = match op {
            6 | 33 | 35 => Some(self.structure(a)?),
            36 | 37 => Some(self.structure(a)?.concatenate(self.structure(b)?)?),
            40 => Some(self.structure(b)?.group()?),
            42 => Some(self.structure(a)?.children()),
            _ => None,
        };
        let object = match op {
            1 => {
                return Ok(self
                    .text(a)?
                    .len()
                    .try_into()
                    .map_err(|_| "host text exceeds protocol")?)
            }
            2 | 3 => Object::Text(match self.get(a)? {
                Object::Stream(v) => {
                    if op == 2 {
                        v.to_string()
                    } else {
                        format!("{v:?}")
                    }
                }
                Object::Token(v) => {
                    if op == 2 {
                        v.to_string()
                    } else {
                        format!("{v:?}")
                    }
                }
                Object::LexError(v) => {
                    if op == 2 {
                        v.to_string()
                    } else {
                        format!("{v:?}")
                    }
                }
                _ => return Err("display requires a token object".into()),
            }),
            4 => {
                return Ok(match self.token(a)? {
                    TokenTree::Group(_) => 1,
                    TokenTree::Ident(_) => 2,
                    TokenTree::Punct(_) => 3,
                    TokenTree::Literal(_) => 4,
                })
            }
            5 => Object::Span(self.token(a)?.span()),
            6 => {
                let mut token = self.token(a)?;
                token.set_span(self.span(b)?);
                Object::Token(token)
            }
            10..=22 => return self.span_operation(op, a, b),
            30 => Object::Stream(TokenStream::new()),
            31 => return Ok(u32::from(self.stream(a)?.is_empty())),
            32 => match self.parsed_text(a)?.parse() {
                Ok(stream) => Object::Stream(stream),
                Err(error) => Object::LexError(error),
            },
            33 => Object::Iterator(self.stream(a)?.into_iter()),
            34 => {
                let Object::Iterator(iter) = self.get_mut(a)? else {
                    return Err("macro handle is not an iterator".into());
                };
                let Some(token) = iter.next() else {
                    return Ok(0);
                };
                Object::Token(token)
            }
            35 => {
                let Object::Iterator(iter) = self.get(a)? else {
                    return Err("macro handle is not an iterator".into());
                };
                Object::Iterator(iter.clone())
            }
            36 => {
                let mut stream = self.stream(a)?;
                stream.extend([self.token(b)?]);
                Object::Stream(stream)
            }
            37 => {
                let mut stream = self.stream(a)?;
                stream.extend([self.stream(b)?]);
                Object::Stream(stream)
            }
            38 => {
                let Object::Iterator(iter) = self.get(a)? else {
                    return Err("macro handle is not an iterator".into());
                };
                let (lower, upper) = iter.size_hint();
                return match b {
                    0 => lower
                        .try_into()
                        .map_err(|_| "iterator exceeds protocol".into()),
                    1 => upper
                        .ok_or("iterator has no upper bound")?
                        .try_into()
                        .map_err(|_| "iterator exceeds protocol".into()),
                    2 => Ok(u32::from(upper.is_some())),
                    _ => Err("unknown iterator size operation".into()),
                };
            }
            40 => Object::Token(TokenTree::Group(Group::new(
                match a {
                    0 => Delimiter::Parenthesis,
                    1 => Delimiter::Brace,
                    2 => Delimiter::Bracket,
                    3 => Delimiter::None,
                    _ => return Err("invalid delimiter".into()),
                },
                self.stream(b)?,
            ))),
            41 => {
                let TokenTree::Group(group) = self.token(a)? else {
                    return Err("macro handle is not a group".into());
                };
                return Ok(match group.delimiter() {
                    Delimiter::Parenthesis => 0,
                    Delimiter::Brace => 1,
                    Delimiter::Bracket => 2,
                    Delimiter::None => 3,
                });
            }
            42..=44 => {
                let TokenTree::Group(group) = self.token(a)? else {
                    return Err("macro handle is not a group".into());
                };
                match op {
                    42 => Object::Stream(group.stream()),
                    43 => Object::Span(group.span_open()),
                    _ => Object::Span(group.span_close()),
                }
            }
            45 => Object::Token(TokenTree::Ident(Ident::new(&self.text(a)?, self.span(b)?))),
            46 => Object::Token(TokenTree::Ident(Ident::new_raw(
                &self.text(a)?,
                self.span(b)?,
            ))),
            47 => Object::Token(TokenTree::Punct(Punct::new(
                char::from_u32(a).ok_or("invalid punctuation scalar")?,
                match b {
                    0 => Spacing::Alone,
                    1 => Spacing::Joint,
                    _ => return Err("invalid spacing".into()),
                },
            ))),
            48..=49 => {
                let TokenTree::Punct(p) = self.token(a)? else {
                    return Err("macro handle is not punctuation".into());
                };
                return Ok(if op == 48 {
                    p.as_char() as u32
                } else {
                    match p.spacing() {
                        Spacing::Alone => 0,
                        Spacing::Joint => 1,
                    }
                });
            }
            50..=56 => return self.literal_operation(op, a, b),
            58 => return Ok(u32::from(matches!(self.get(a)?, Object::LexError(_)))),
            _ => return Err(format!("unsupported macro operation {op}").into()),
        };
        if let Some(structure) = carried {
            self.push_carried(object, structure)
        } else {
            self.push(object)
        }
    }
}

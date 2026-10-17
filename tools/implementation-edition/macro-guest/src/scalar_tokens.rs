impl Literal {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
impl Punct {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
impl Ident {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
impl Group {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
use crate::{call, text, text_handle, Span, TokenStream};
use std::{fmt, str::FromStr};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    Alone,
    Joint,
}
#[derive(Clone)]
pub struct Group(pub(crate) u32, std::marker::PhantomData<std::rc::Rc<()>>);
#[derive(Clone)]
pub struct Ident(pub(crate) u32, std::marker::PhantomData<std::rc::Rc<()>>);
#[derive(Clone)]
pub struct Punct(pub(crate) u32, std::marker::PhantomData<std::rc::Rc<()>>);
#[derive(Clone)]
pub struct Literal(pub(crate) u32, std::marker::PhantomData<std::rc::Rc<()>>);
impl Group {
    pub fn new(delimiter: Delimiter, stream: TokenStream) -> Self {
        Self::from_handle(call(40, delimiter as u32, stream.0, 0))
    }
    pub fn delimiter(&self) -> Delimiter {
        match call(41, self.0, 0, 0) {
            0 => Delimiter::Parenthesis,
            1 => Delimiter::Brace,
            2 => Delimiter::Bracket,
            3 => Delimiter::None,
            _ => panic!("invalid delimiter"),
        }
    }
    pub fn stream(&self) -> TokenStream {
        TokenStream::from_handle(call(42, self.0, 0, 0))
    }
    pub fn span_open(&self) -> Span {
        Span::from_handle(call(43, self.0, 0, 0))
    }
    pub fn span_close(&self) -> Span {
        Span::from_handle(call(44, self.0, 0, 0))
    }
}
impl Ident {
    pub fn new(s: &str, span: Span) -> Self {
        Self::from_handle(call(45, text_handle(s), span.0, 0))
    }
    pub fn new_raw(s: &str, span: Span) -> Self {
        Self::from_handle(call(46, text_handle(s), span.0, 0))
    }
}
impl Punct {
    pub fn new(c: char, spacing: Spacing) -> Self {
        Self::from_handle(call(47, c as u32, spacing as u32, 0))
    }
    pub fn as_char(&self) -> char {
        char::from_u32(call(48, self.0, 0, 0)).unwrap()
    }
    pub fn spacing(&self) -> Spacing {
        match call(49, self.0, 0, 0) {
            0 => Spacing::Alone,
            1 => Spacing::Joint,
            _ => panic!("invalid spacing"),
        }
    }
}
macro_rules! common {
    ($ty:ident) => {
        impl $ty {
            pub fn span(&self) -> Span {
                Span::from_handle(call(5, self.0, 0, 0))
            }
            pub fn set_span(&mut self, span: Span) {
                self.0 = call(6, self.0, span.0, 0);
            }
        }
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&text(call(2, self.0, 0, 0)))
            }
        }
        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&text(call(3, self.0, 0, 0)))
            }
        }
    };
}
common!(Group);
common!(Ident);
common!(Punct);
common!(Literal);
impl FromStr for Literal {
    type Err = crate::LexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let h = call(50, text_handle(s), 0, 0);
        if call(58, h, 0, 0) != 0 {
            Err(crate::LexError(h))
        } else {
            Ok(Self::from_handle(h))
        }
    }
}
impl Literal {
    pub fn string(s: &str) -> Self {
        Self::from_handle(call(51, text_handle(s), 0, 0))
    }
    pub fn character(c: char) -> Self {
        Self::from_handle(call(52, c as u32, 0, 0))
    }
    pub fn byte_string(s: &[u8]) -> Self {
        let hex: String = s.iter().map(|b| format!("{b:02x}")).collect();
        Self::from_handle(call(53, text_handle(&hex), 0, 0))
    }
    pub fn c_string(s: &std::ffi::CStr) -> Self {
        let hex: String = s
            .to_bytes_with_nul()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Self::from_handle(call(54, text_handle(&hex), 0, 0))
    }
    pub fn byte_character(c: u8) -> Self {
        Self::from_handle(call(55, u32::from(c), 0, 0))
    }
}
macro_rules! numeric {
    ($name:ident,$ty:ty,$kind:expr) => {
        impl Literal {
            pub fn $name(value: $ty) -> Self {
                Self::from_handle(call(56, text_handle(&value.to_string()), $kind, 0))
            }
        }
    };
}
numeric!(u8_suffixed, u8, 0);
numeric!(u16_suffixed, u16, 1);
numeric!(u32_suffixed, u32, 2);
numeric!(u64_suffixed, u64, 3);
numeric!(u128_suffixed, u128, 4);
numeric!(usize_suffixed, usize, 5);
numeric!(i8_suffixed, i8, 6);
numeric!(i16_suffixed, i16, 7);
numeric!(i32_suffixed, i32, 8);
numeric!(i64_suffixed, i64, 9);
numeric!(i128_suffixed, i128, 10);
numeric!(isize_suffixed, isize, 11);
numeric!(f32_suffixed, f32, 12);
numeric!(f64_suffixed, f64, 13);
numeric!(u8_unsuffixed, u8, 14);
numeric!(u16_unsuffixed, u16, 15);
numeric!(u32_unsuffixed, u32, 16);
numeric!(u64_unsuffixed, u64, 17);
numeric!(u128_unsuffixed, u128, 18);
numeric!(usize_unsuffixed, usize, 19);
numeric!(i8_unsuffixed, i8, 20);
numeric!(i16_unsuffixed, i16, 21);
numeric!(i32_unsuffixed, i32, 22);
numeric!(i64_unsuffixed, i64, 23);
numeric!(i128_unsuffixed, i128, 24);
numeric!(isize_unsuffixed, isize, 25);
numeric!(f32_unsuffixed, f32, 26);
numeric!(f64_unsuffixed, f64, 27);

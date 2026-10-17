impl TokenStream {
    pub(crate) fn from_handle(handle: u32) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}
use crate::{call, text, text_handle, Group, Ident, Literal, Punct};
use std::{fmt, str::FromStr};

#[derive(Clone)]
pub struct TokenStream(pub u32, std::marker::PhantomData<std::rc::Rc<()>>);
#[derive(Clone)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}
pub struct LexError(pub(crate) u32);
impl fmt::Debug for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(3, self.0, 0, 0)))
    }
}
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(2, self.0, 0, 0)))
    }
}
impl std::error::Error for LexError {}
impl TokenStream {
    pub fn from_host(handle: u32) -> Self {
        Self::from_handle(handle)
    }
    pub fn new() -> Self {
        Self::from_handle(call(30, 0, 0, 0))
    }
    pub fn is_empty(&self) -> bool {
        call(31, self.0, 0, 0) != 0
    }
}
impl Default for TokenStream {
    fn default() -> Self {
        Self::new()
    }
}
impl FromStr for TokenStream {
    type Err = LexError;
    fn from_str(s: &str) -> Result<Self, LexError> {
        let h = call(32, text_handle(s), 0, 0);
        if call(58, h, 0, 0) != 0 {
            Err(LexError(h))
        } else {
            Ok(Self::from_handle(h))
        }
    }
}
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(2, self.0, 0, 0)))
    }
}
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(3, self.0, 0, 0)))
    }
}
impl TokenTree {
    pub fn span(&self) -> crate::Span {
        crate::Span::from_handle(call(5, self.handle(), 0, 0))
    }
    pub fn set_span(&mut self, span: crate::Span) {
        let h = call(6, self.handle(), span.0, 0);
        *self = Self::from_handle(h);
    }
    pub(crate) fn handle(&self) -> u32 {
        match self {
            Self::Group(v) => v.0,
            Self::Ident(v) => v.0,
            Self::Punct(v) => v.0,
            Self::Literal(v) => v.0,
        }
    }
    fn from_handle(h: u32) -> Self {
        match call(4, h, 0, 0) {
            1 => Self::Group(Group::from_handle(h)),
            2 => Self::Ident(Ident::from_handle(h)),
            3 => Self::Punct(Punct::from_handle(h)),
            4 => Self::Literal(Literal::from_handle(h)),
            _ => panic!("invalid token kind"),
        }
    }
}
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(2, self.handle(), 0, 0)))
    }
}
impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&text(call(3, self.handle(), 0, 0)))
    }
}
impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = token_stream::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        token_stream::IntoIter(call(33, self.0, 0, 0), std::marker::PhantomData)
    }
}
pub mod token_stream {
    pub struct IntoIter(
        pub(crate) u32,
        pub(crate) std::marker::PhantomData<std::rc::Rc<()>>,
    );
    impl Iterator for IntoIter {
        type Item = super::TokenTree;
        fn next(&mut self) -> Option<Self::Item> {
            let h = crate::call(34, self.0, 0, 0);
            (h != 0).then(|| super::TokenTree::from_handle(h))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            let lower = crate::call(38, self.0, 0, 0) as usize;
            let upper = (crate::call(38, self.0, 2, 0) != 0)
                .then(|| crate::call(38, self.0, 1, 0) as usize);
            (lower, upper)
        }
    }
    impl Clone for IntoIter {
        fn clone(&self) -> Self {
            Self(crate::call(35, self.0, 0, 0), std::marker::PhantomData)
        }
    }
}
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, iter: I) {
        for token in iter {
            self.0 = call(36, self.0, token.handle(), 0);
        }
    }
}
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, iter: I) {
        for stream in iter {
            self.0 = call(37, self.0, stream.0, 0);
        }
    }
}
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(i: I) -> Self {
        let mut s = Self::new();
        s.extend(i);
        s
    }
}
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(i: I) -> Self {
        let mut s = Self::new();
        s.extend(i);
        s
    }
}
impl From<TokenTree> for TokenStream {
    fn from(t: TokenTree) -> Self {
        std::iter::once(t).collect()
    }
}
macro_rules! from_token {
    ($ty:ident) => {
        impl From<$ty> for TokenTree {
            fn from(v: $ty) -> Self {
                Self::$ty(v)
            }
        }
    };
}
from_token!(Group);
from_token!(Ident);
from_token!(Punct);
from_token!(Literal);

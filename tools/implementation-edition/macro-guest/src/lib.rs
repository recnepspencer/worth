//! Confined guest proc_macro API. All compiler objects remain in the host.
//! This is a distinct effective compilation, not the native sysroot library.
mod scalar_tokens;
mod span;
#[path = "token_stream.rs"]
mod streams;
mod transport;
pub use scalar_tokens::{Delimiter, Group, Ident, Literal, Punct, Spacing};
pub use span::Span;
pub use streams::{token_stream, LexError, TokenStream, TokenTree};
pub fn is_available() -> bool {
    true
}

pub(crate) use transport::{call, text, text_handle};

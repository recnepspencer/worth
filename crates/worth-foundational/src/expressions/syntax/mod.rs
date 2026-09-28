//! Bounded lexer, parser, and arena syntax tree for expression drafts.

pub(crate) mod ast;
pub(crate) mod literal;
mod parser;
mod primary;
mod source_map;
mod tokens;

pub(crate) use parser::{parse_source, SyntaxLimits};
pub(crate) use primary::GENERIC_INTRINSICS;
pub use source_map::{ExpressionSourceMap, SourceOrigin, SourceSpan};

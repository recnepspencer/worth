//! The closed builtin vocabulary and installed expression functions.

mod builtins;
mod catalog;

pub(crate) use builtins::{is_reserved_callable, Builtin};
pub use catalog::{
    ExpressionFunctionCatalog, ExpressionFunctionCatalogBuilder, ExpressionFunctionDeclaration,
    InstalledExpressionFunction,
};

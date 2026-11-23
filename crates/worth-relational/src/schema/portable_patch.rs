//! Request-funded portable patch grammar shared by mutation and retained comparison.
mod comparison;
pub(crate) mod endpoints;
pub(crate) mod quote;

pub(crate) use comparison::append_portable_in_scope;

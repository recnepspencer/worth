//! Canonical expression meaning.

mod basis;
mod identity;

pub use identity::ExpressionProgramIdentity;
pub(crate) use identity::{derive_identity, IdentityInputs};

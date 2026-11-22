//! Deterministic identity namespaces for application declarations.
//!
//! One canonical root (for example `daymark.tasks.add.v1`) derives every
//! sub-identity an action, query or feature needs: `root.input`,
//! `root.binding`, `root.slot.title` and so on. An item whose persisted wire
//! identity predates its namespace keeps it through an explicit per-role
//! override, so adopting a namespace never churns stored identities.
//!
//! Every constructor validates and refuses with a typed
//! [`ApplicationIdentityNamespaceDenial`]; none panics. The const macros
//! `worth_query_identity_namespace!` and `worth_query_identity!` evaluate the
//! same constructors at compile time, where a refusal is a compile error.
//!
//! A namespace is naming only. It grants nothing; the installed schema still
//! owns member uniqueness and every admission decision.

mod denial;
mod derived;
mod grammar;
mod namespace;
mod role;
#[cfg(test)]
mod tests;

pub use denial::ApplicationIdentityNamespaceDenial;
pub use derived::ApplicationDerivedIdentity;
pub use grammar::MAXIMUM_APPLICATION_IDENTITY_BYTES;
pub use namespace::ApplicationIdentityNamespace;
pub use role::ApplicationIdentityRole;

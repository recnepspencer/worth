//! Application aggregate names; every read still requires fresh view admission.
mod identity;
pub use identity::WorthQueryAggregateIdentity;

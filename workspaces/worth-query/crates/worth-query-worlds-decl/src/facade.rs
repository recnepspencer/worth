//! The decl world-family facade.
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts use concrete definitions through these scopes."
)]
#[path = "decl/mod.rs"]
pub mod decl;

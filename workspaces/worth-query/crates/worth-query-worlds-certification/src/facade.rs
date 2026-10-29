//! The certification world-family facade.
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts use concrete definitions through these scopes."
)]
#[path = "certification/mod.rs"]
pub mod certification;
pub use worth_query_worlds_host::facade::host;

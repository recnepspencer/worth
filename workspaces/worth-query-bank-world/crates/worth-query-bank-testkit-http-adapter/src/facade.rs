#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Authored fixture modules retain lexical imports at their named facade."
)]
#[path = "mod.rs"]
mod fixture;
pub use fixture::*;

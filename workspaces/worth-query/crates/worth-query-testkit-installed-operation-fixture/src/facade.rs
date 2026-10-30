#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Preserve authored fixture scopes while concrete uses retain compiler checking."
)]
#[path = "query/mod.rs"]
pub mod query;

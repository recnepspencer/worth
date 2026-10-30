#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Preserve authored fixture scopes while concrete uses retain compiler checking."
)]
#[path = "package_archive/mod.rs"]
pub mod package_archive;

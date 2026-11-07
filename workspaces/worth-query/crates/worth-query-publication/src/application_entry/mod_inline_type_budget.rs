//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    let _cleanup=census.owner::<super::WorthQueryOperationCleanup>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/cleanup.rs","WorthQueryOperationCleanup",2*1024);
    super::correction::inline_type_budget::collect(census);
    super::demand::inline_type_budget::collect(census);
    super::denial::inline_type_budget::collect(census);
    super::history::inline_type_budget::collect(census);
    super::mutation::inline_type_budget::collect(census);
    super::operation::inline_type_budget::collect(census);
    super::request::inline_type_budget::collect(census);
    super::retained_read::inline_type_budget::collect(census);
    super::workflow::inline_type_budget::collect(census);
}

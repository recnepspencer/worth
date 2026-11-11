//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::context::inline_type_budget::collect(census);
    super::correction_source::inline_type_budget::collect(census);
    super::document_history::inline_type_budget::collect(census);
    super::generated_materialization::inline_type_budget::collect(census);
}

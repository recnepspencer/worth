//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::artifact_execution::inline_type_budget::collect(census);
    super::draft_execution::inline_type_budget::collect(census);
    super::mutation_execution::inline_type_budget::collect(census);
    super::publication_execution::inline_type_budget::collect(census);
}

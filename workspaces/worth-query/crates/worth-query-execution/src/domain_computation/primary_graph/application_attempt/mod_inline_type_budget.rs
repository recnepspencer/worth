//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::compare_and_commit::inline_type_budget::collect(census);
    super::denial::inline_type_budget::collect(census);
    super::effect_program::inline_type_budget::collect(census);
    super::idempotency::inline_type_budget::collect(census);
    super::idempotency_resolution::inline_type_budget::collect(census);
    super::provider_execution::inline_type_budget::collect(census);
}

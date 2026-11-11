//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::execution::inline_type_budget::collect(census);
    super::installation_extension::inline_type_budget::collect(census);
    super::temporal::inline_type_budget::collect(census);
}

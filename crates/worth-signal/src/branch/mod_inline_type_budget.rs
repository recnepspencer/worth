//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::advance_completion::inline_type_budget::collect(census);
    super::advancement::inline_type_budget::collect(census);
    super::basis_registry::inline_type_budget::collect(census);
    super::owner_services::inline_type_budget::collect(census);
}

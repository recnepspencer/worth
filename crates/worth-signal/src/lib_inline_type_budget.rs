//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::branch::inline_type_budget::collect(census);
    super::data::inline_type_budget::collect(census);
    super::logic::inline_type_budget::collect(census);
    super::observation::inline_type_budget::collect(census);
    super::runtime_policy::inline_type_budget::collect(census);
    super::state::inline_type_budget::collect(census);
}

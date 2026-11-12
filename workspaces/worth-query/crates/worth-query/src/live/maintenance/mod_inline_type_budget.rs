//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::admission::inline_type_budget::collect(census);
    super::consumer_policy::inline_type_budget::collect(census);
}

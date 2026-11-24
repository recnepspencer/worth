//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::census::Census) {
    super::preparation::inline_type_budget::collect(census);
}

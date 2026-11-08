//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::creation::inline_type_budget::collect(census);
    super::read_identity::inline_type_budget::collect(census);
    super::selection::inline_type_budget::collect(census);
}

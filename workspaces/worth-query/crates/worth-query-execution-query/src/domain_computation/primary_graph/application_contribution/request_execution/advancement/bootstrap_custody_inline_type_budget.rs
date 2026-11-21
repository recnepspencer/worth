//! Unit-only scalar observations are recorded at the defining owner.
pub fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::collect_owned_inline_type_sizes(census);
}

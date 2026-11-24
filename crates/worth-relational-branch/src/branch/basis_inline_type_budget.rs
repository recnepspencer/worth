//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub fn collect(census: &mut crate::inline_type_sizes::census::Census) {
    super::retirement::inline_type_budget::collect(census);
}

//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub fn collect(census: &mut crate::inline_type_sizes::census::Census) {
    super::required_output_origin::inline_type_budget::collect(census);
}

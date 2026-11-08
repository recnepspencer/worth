//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::authority::inline_type_budget::collect(census);
    super::counters::inline_type_budget::collect(census);
    super::denial::inline_type_budget::collect(census);
    super::posture::inline_type_budget::collect(census);
    super::request::inline_type_budget::collect(census);
    super::step_contract::inline_type_budget::collect(census);
}

//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::application_attempt_affinity::inline_type_budget::collect(census);
    super::elevation_currentness::inline_type_budget::collect(census);
    super::phase::inline_type_budget::collect(census);
}

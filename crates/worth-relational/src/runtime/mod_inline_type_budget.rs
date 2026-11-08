//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::home_program_transition::inline_type_budget::collect(census);
    super::state::inline_type_budget::collect(census);
}

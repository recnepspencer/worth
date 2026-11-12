//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::outcome::inline_type_budget::collect(census);
    super::performed::inline_type_budget::collect(census);
    super::program_output_settlement::inline_type_budget::collect(census);
    super::recovery::inline_type_budget::collect(census);
    super::retained::inline_type_budget::collect(census);
}

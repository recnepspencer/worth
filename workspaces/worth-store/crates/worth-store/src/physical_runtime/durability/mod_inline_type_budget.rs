//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::closeout::inline_type_budget::collect(census);
    super::lifecycle::inline_type_budget::collect(census);
    super::mutation::inline_type_budget::collect(census);
    super::publication::inline_type_budget::collect(census);
    super::settlement::inline_type_budget::collect(census);
    super::wal::inline_type_budget::collect(census);
}

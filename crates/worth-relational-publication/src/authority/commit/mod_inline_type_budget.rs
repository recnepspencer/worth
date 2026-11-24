//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(
    census: &mut worth_relational_contracts::internal_port::inline_type_sizes::census::Census,
) {
    super::pipeline::inline_type_budget::collect(census);
}

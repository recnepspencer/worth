//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::application_aftermath::inline_type_budget::collect(census);
    super::authorization::inline_type_budget::collect(census);
    super::execution_runtime::inline_type_budget::collect(census);
    super::managed_run::inline_type_budget::collect(census);
    super::primary_graph::inline_type_budget::collect(census);
    super::product_publication::inline_type_budget::collect(census);
    super::execution_observation::inline_type_budget::collect(census);
}

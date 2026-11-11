//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::backend::inline_type_budget::collect(census);
    super::builder::inline_type_budget::collect(census);
    super::computed::inline_type_budget::collect(census);
    super::conditional_resources::inline_type_budget::collect(census);
    super::installed_product::inline_type_budget::collect(census);
    super::primary_graph_source::inline_type_budget::collect(census);
    super::runtime_root_state::inline_type_budget::collect(census);
    super::effect::inline_type_budget::collect(census);
    super::error::inline_type_budget::collect(census);
    super::granular_source_read_basis::inline_type_budget::collect(census);
    super::graph_read_access::inline_type_budget::collect(census);
    super::journal_replay::inline_type_budget::collect(census);
    super::managed_live_resource::inline_type_budget::collect(census);
    super::published_artifacts::inline_type_budget::collect(census);
    super::owned_async_source::inline_type_budget::collect(census);
    super::shared_read_pins::inline_type_budget::collect(census);
    super::support::inline_type_budget::collect(census);
    super::surface::inline_type_budget::collect(census);
    super::workspace::inline_type_budget::collect(census);
}

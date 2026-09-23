pub(super) mod allocation_frontier;
pub(super) mod batch_placement;
pub(super) mod extent_placement;
pub(super) mod free_space_projection;
pub(super) mod free_space_routing;
pub(super) mod inline_page_packing;
pub(super) mod inline_plan_failure;
pub(super) mod inline_segment_plan;
pub(super) mod placement_context;
pub(super) mod placement_policy;
pub(super) mod policy_units;
pub(super) mod prepared_payload;
mod prepared_root_projection;
pub(super) mod published_segment_reuse;
pub(super) mod published_tail_page;
pub(super) mod rebased_root;
pub(super) mod reusable_inline_tail;
mod settled_root_projection;

pub(in crate::physical_runtime) use prepared_root_projection::{
    sealed_publication_overhead, PreparedPhysicalRootProjection,
};
pub(in crate::physical_runtime::record_serving) use settled_root_projection::merge_settled_root_projections;
pub(in crate::physical_runtime) use settled_root_projection::RejectedSettledRootProjections;
pub use settled_root_projection::SettledRootProjectionMergeDenial;

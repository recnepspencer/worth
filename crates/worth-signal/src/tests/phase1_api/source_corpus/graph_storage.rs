pub(in crate::tests::phase1_api) const ENTRIES_SOURCE: &str = concat!(
    include_str!("../../../data/graph/storage/entries.rs"),
    include_str!("../../../data/graph/storage/entries/access.rs"),
    include_str!("../../../data/graph/storage/entries/allocation.rs"),
    include_str!("../../../data/graph/storage/entries/construction.rs"),
    include_str!("../../../data/graph/storage/entries/contracts.rs"),
    include_str!("../../../data/graph/storage/entries/diagnostic_artifacts.rs"),
    include_str!("../../../data/graph/storage/entries/iteration.rs"),
    include_str!("../../../data/graph/storage/entries/snapshots.rs"),
    include_str!("../../../data/graph/storage/entries/transitions.rs"),
);
pub(in crate::tests::phase1_api) const GRAPH_RUNTIME_SOURCE: &str = concat!(
    include_str!("../../../data/graph/runtime/graph.rs"),
    include_str!("../../../data/graph/runtime/graph/branch_mutations.rs"),
    include_str!("../../../data/graph/runtime/graph/capabilities.rs"),
    include_str!("../../../data/graph/runtime/graph/checkpoint.rs"),
    include_str!("../../../data/graph/runtime/graph/construction.rs"),
    include_str!("../../../data/graph/runtime/graph/counter_access.rs"),
    include_str!("../../../data/graph/runtime/graph/observation_state.rs"),
    include_str!("../../../data/graph/runtime/graph/reconstruction_counters.rs"),
    include_str!("../../../data/graph/runtime/graph/scratch_lease.rs"),
    include_str!("../../../data/graph/runtime/graph/topology_state.rs"),
    include_str!("../../../data/graph/runtime/graph/traversal_state.rs"),
);
pub(in crate::tests::phase1_api) const SLOT_SOURCE: &str =
    include_str!("../../../data/graph/storage/slot.rs");
pub(in crate::tests::phase1_api) const PERSISTENT_PAGED_VECTOR_SOURCE: &str =
    include_str!("../../../data/persistent_paged_vector.rs");
pub(in crate::tests::phase1_api) const PERSISTENT_VECTOR_SOURCE: &str =
    include_str!("../../../data/persistent_vector.rs");
pub(in crate::tests::phase1_api) const INVALIDATION_AUTHORITY_SOURCE: &str =
    include_str!("../../../data/graph/storage/entries/invalidation_authority.rs");
pub(in crate::tests::phase1_api) const ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE: &str = concat!(
    include_str!("../../../data/graph/storage/invalidation_causes/application.rs"),
    include_str!("../../../data/graph/topology/mutation/classification.rs"),
);
pub(in crate::tests::phase1_api) const ENTRY_TRANSITIONS_SOURCE: &str =
    include_str!("../../../data/graph/storage/entries/transitions.rs");

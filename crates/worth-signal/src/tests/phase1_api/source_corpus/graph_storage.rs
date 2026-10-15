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
pub(in crate::tests::phase1_api) const INVALIDATION_AUTHORITY_SOURCE: &str =
    include_str!("../../../data/graph/storage/entries/invalidation_authority.rs");
pub(in crate::tests::phase1_api) const ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE: &str = concat!(
    include_str!("../../../data/graph/storage/invalidation_causes/application.rs"),
    include_str!("../../../data/graph/topology/mutation/classification.rs"),
);
pub(in crate::tests::phase1_api) const ENTRY_TRANSITIONS_SOURCE: &str =
    include_str!("../../../data/graph/storage/entries/transitions.rs");

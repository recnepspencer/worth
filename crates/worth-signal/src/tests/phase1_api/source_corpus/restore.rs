pub(in crate::tests::phase1_api) const BRANCH_BASIS_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/state/branching/basis.rs");
pub(in crate::tests::phase1_api) const BRANCH_BASIS_RUNTIME_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/state/branching/basis_runtime.rs");
pub(in crate::tests::phase1_api) const BRANCH_FORK_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/state/branching/fork.rs");
pub(in crate::tests::phase1_api) const BRANCHES_SOURCE: &str = concat!(
    include_str!("../../../logic/transaction/runtime/state/branching/branches.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/authority.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/catalog.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/lifecycle.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/owner_snapshot.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/selection.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/snapshot_storage.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/branches/transfer.rs"),
);
pub(in crate::tests::phase1_api) const LIFECYCLE_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/state/branching/lifecycle.rs");
pub(in crate::tests::phase1_api) const RUNTIME_STATE_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/state/runtime_state/transfer_packets.rs");
pub(in crate::tests::phase1_api) const SNAPSHOT_RESTORE_SOURCE: &str = concat!(
    include_str!("../../../data/graph/diagnostics_access/artifacts.rs"),
    include_str!("../../../data/graph/diagnostics_access/artifacts/derived.rs"),
    include_str!("../../../data/graph/diagnostics_access/artifacts/historical.rs"),
    include_str!("../../../data/graph/diagnostics_access/artifacts/retained.rs"),
    include_str!("../../../data/graph/diagnostics_access/artifacts/summary.rs"),
    include_str!("../../../data/graph/diagnostics_access/artifacts/tier.rs"),
);
pub(in crate::tests::phase1_api) const RUNTIME_SNAPSHOTTING_SOURCE: &str = concat!(
    include_str!("../../../logic/transaction/runtime/state/branching/snapshotting.rs"),
    include_str!("../../../logic/transaction/runtime/state/branching/snapshotting/capture.rs"),
    include_str!(
        "../../../logic/transaction/runtime/state/branching/snapshotting/restore_selection.rs"
    ),
    include_str!("../../../logic/transaction/runtime/state/branching/snapshotting/validation.rs"),
);
pub(in crate::tests::phase1_api) const CHECKPOINT_IMAGE_SOURCE: &str =
    include_str!("../../../data/node/checkpoint_image.rs");
pub(in crate::tests::phase1_api) const STATE_SOURCE: &str = concat!(
    include_str!("../../../state/mod.rs"),
    include_str!("../../../state/restore.rs"),
);

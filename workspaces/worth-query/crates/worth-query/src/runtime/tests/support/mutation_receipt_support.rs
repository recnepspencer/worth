use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMutationDelta,
    WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
};
use crate::runtime::WorthQueryAspectTouch;
use worth_runtime_bridge::facade::BridgeMutationAuthorityBundle;

pub(in crate::runtime::tests) fn test_mutation_receipt(
    commit_identity: WorthQueryCommitIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    collection: impl Into<String>,
    entity_identity: WorthQueryEntityIdentity,
    kind: WorthQueryMutationKind,
    aspect_touches: Vec<WorthQueryAspectTouch>,
) -> WorthQueryMutationReceipt {
    WorthQueryMutationReceipt::from_authoritative_parts(
        commit_identity,
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            collection,
            entity_identity,
            kind,
            aspect_touches,
        )
        .expect("fixture digest must fit")],
    )
}

pub(in crate::runtime::tests) fn test_mutation_receipt_with_bridge_authority(
    commit_identity: WorthQueryCommitIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    collection: impl Into<String>,
    entity_identity: WorthQueryEntityIdentity,
    kind: WorthQueryMutationKind,
    aspect_touches: Vec<WorthQueryAspectTouch>,
    bridge_authority: BridgeMutationAuthorityBundle,
) -> WorthQueryMutationReceipt {
    WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        commit_identity,
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            collection,
            entity_identity,
            kind,
            aspect_touches,
        )
        .expect("fixture digest must fit")],
        bridge_authority,
    )
    .admit_runtime_write_authority()
}

use super::*;
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
};

pub(super) fn snapshot_identity_from_runtime(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> WorthQuerySnapshotIdentity {
    runtime
        .history()
        .historical_latest_commit()
        .map(|commit| {
            WorthQuerySnapshotIdentity::from_runtime_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(commit.commit_id.0, commit.version_id.0),
            )
        })
        .unwrap_or_else(WorthQuerySnapshotIdentity::empty_runtime_state)
}

pub(crate) fn snapshot_identity_from_branch(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &worth_relational::facade::history::BranchId,
) -> Result<
    Option<WorthQuerySnapshotIdentity>,
    worth_relational::facade::branch::RelationalBranchBasisDenial,
> {
    let identity = runtime.branch_identity(branch).map_err(|_| {
        worth_relational::facade::branch::RelationalBranchBasisDenial::UnknownBranch(branch.clone())
    })?;
    let (_, basis) = runtime.observe_branch(&identity)?;
    Ok(snapshot_identity_from_admitted_basis(&basis))
}

pub(crate) fn snapshot_identity_from_admitted_basis(
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Option<WorthQuerySnapshotIdentity> {
    let observation = basis.observation();
    observation.commit_id().map(|commit_id| {
        WorthQuerySnapshotIdentity::from_runtime_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(commit_id.0, observation.version_id().0),
        )
    })
}

pub(super) fn entity_identity(
    entity: worth_relational::facade::identity::EntityId,
) -> WorthQueryEntityIdentity {
    WorthQueryEntityIdentity::from_runtime_receipt_record(
        RelationalBridgeRecordIdentityParts::entity(
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
    )
}

pub(super) fn entity_id_from_identity(
    identity: WorthQueryEntityIdentity,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryWorkspaceError> {
    let Some(parts) = identity
        .has_current_authority()
        .then(|| identity.relational_record_parts())
        .flatten()
    else {
        return Err(WorthQueryWorkspaceError::new(
            "memory workspace mutations require current relational entity authority",
        ));
    };
    Ok(worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId(parts.partition_id()),
        parts.local_slot(),
        parts.generation(),
    ))
}

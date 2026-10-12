use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RuntimeBridge;

use crate::facade::foundation::{
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQueryMutationReceipt,
    WorthQueryWorkspaceError,
};
use crate::facade::runtime::{
    WorthQueryAspectTouch, WorthQueryBackendAdmissibleMutation,
    WorthQueryRuntimeWriteAuthorityAdapter, WriteAuthorityExecutionReceipt,
};
use crate::runtime::build_bridge_authority_bundle;
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

use super::certification_entity_identity;

pub(super) struct CertificationWriteAuthority;

impl WorthQueryRuntimeWriteAuthorityAdapter for CertificationWriteAuthority {
    fn write(
        &mut self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
        let (collection, aspect_touches) = write_receipt_shape(&mutation);
        let entity_identity = "certification-entity-1";
        let commit_identity =
            crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1);
        let snapshot_identity =
            crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(1, 1),
            );
        let entity_identity_handle = certification_entity_identity(entity_identity);
        let bridge_authority = build_bridge_authority_bundle(
            bridge,
            &snapshot_identity,
            &mutation,
            crate::runtime::WorthQueryBridgeMutationTarget::new(
                &collection,
                &entity_identity_handle,
                WorthQueryMutationKind::Updated,
            ),
        )?;
        let receipt = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
            commit_identity,
            snapshot_identity,
            vec![WorthQueryMutationDelta::from_touched_aspects(
                collection,
                entity_identity_handle,
                WorthQueryMutationKind::Updated,
                aspect_touches,
            )],
            bridge_authority,
        );
        Ok(self.build_write_authority_execution_receipt(&mutation, receipt))
    }
}

fn write_receipt_shape(
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> (String, Vec<WorthQueryAspectTouch>) {
    (
        mutation
            .declared_collection_identity()
            .map(|collection| collection.as_str().to_string())
            .unwrap_or("Task".to_string()),
        mutation.declared_aspect_touches(),
    )
}

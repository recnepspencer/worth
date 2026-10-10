//! Backend write authority receipt construction and batch admission.
use super::*;

pub trait WorthQueryRuntimeWriteAuthorityAdapter {
    fn build_bridge_mutation_authority_bundle(
        &self,
        bridge: &RuntimeBridge,
        snapshot_identity: &WorthQuerySnapshotIdentity,
        mutation: &WorthQueryBackendAdmissibleMutation,
        collection: &str,
        entity_identity: &WorthQueryEntityIdentity,
        mutation_kind: WorthQueryMutationKind,
    ) -> Result<BridgeMutationAuthorityBundle, WorthQueryWorkspaceError> {
        super::super::build_bridge_authority_bundle(
            bridge,
            snapshot_identity,
            mutation,
            super::super::WorthQueryBridgeMutationTarget::new(
                collection,
                entity_identity,
                mutation_kind,
            ),
        )
    }

    fn build_write_authority_execution_receipt(
        &self,
        mutation: &WorthQueryBackendAdmissibleMutation,
        receipt: WorthQueryMutationReceipt,
    ) -> WriteAuthorityExecutionReceipt {
        WriteAuthorityExecutionReceipt::from_backend_admissible_mutation(
            mutation,
            receipt.admit_runtime_write_authority(),
        )
    }

    fn write(
        &mut self,
        bridge: &RuntimeBridge,
        relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError>;

    fn write_batch(
        &mut self,
        bridge: &RuntimeBridge,
        mut relational_runtime: Option<&mut RelationalRuntime>,
        mutations: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WriteAuthorityExecutionReceipt>, WorthQueryWorkspaceError> {
        let mut receipts = Vec::with_capacity(mutations.len());
        for mutation in mutations {
            receipts.push(self.write(bridge, relational_runtime.as_deref_mut(), mutation)?);
        }
        Ok(receipts)
    }
}

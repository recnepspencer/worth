//! Closed preflight of an external native-only verb against actual current custody.
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
    transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    /// Select an actual owned business edge; callers supply no publication,
    /// relation roster or ownership proof. Preparation never publishes an effect.
    #[doc(hidden)]
    pub fn prepare_owned_edge_retarget_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        target: EntityId,
    ) -> Result<(), TransactionCommitError> {
        let current = super::select_current(execution, basis)?;
        let registry = basis.inner.root.schema_authority().registry();
        let edge = current
            .origins()
            .filter(|origin| origin.meaning.source_scope == owner)
            .flat_map(|origin| &origin.relations)
            .find(|edge| !registry.native_private_kinds.contains(&edge.kind))
            .ok_or_else(|| {
                crate::mvcc::origin_denial(
                    "owned edge retarget court requires a real business edge",
                )
            })?;
        let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id: edge.id,
                kind_id: edge.kind,
                source: EntityReference::Existing(edge.source),
                target: EntityReference::Existing(target),
            },
        ));
        let mut transaction = self
            .begin_branch_transaction(basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| {
                crate::mvcc::origin_denial(format!("actual endpoint probe admission: {cause:?}"))
            })?;
        transaction
            .push_batch(
                WorkerIntentBatch::new("external-owned-edge-retarget").push(intent),
                execution.allocation_policy(),
            )
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        let _unpublished = self.prepare_branch_transaction(execution, transaction)?;
        Ok(())
    }
}

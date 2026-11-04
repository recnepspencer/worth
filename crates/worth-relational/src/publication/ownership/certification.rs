//! Closed preflight of an external native-only verb against actual current custody.
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
    transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    /// Remove both current fact families from the real selected read image. The
    /// independently retained warm position image must still refuse the loss.
    #[doc(hidden)]
    pub fn probe_coherent_ownership_loss_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(), TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                let records = [RecordRef::Entity(owner)];
                let mut current = super::read(execution, basis, &records, context)?;
                assert_eq!(
                    current.publications.len(),
                    1,
                    "closed court selects one real publication"
                );
                current.publications.clear();
                current.codec_retired.clear();
                current.catalogs.clear();
                current.edges.clear();
                current.links.clear();
                current.positions.clear();
                super::read::anchors::validate(
                    &mut current,
                    &basis.inner.root.output_publication_positions,
                    &records,
                    context,
                )
            })
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?
            .0
    }

    /// Inspect the actual current aggregate memberships of this Query publication.
    /// Returned identifiers grant no mutation or publication standing.
    #[doc(hidden)]
    pub fn current_owned_aggregate_memberships_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<Vec<crate::identity::data::RelationId>, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
        let registry = basis.inner.root.schema_authority().registry();
        Ok(current
            .origins()
            .filter(|o| o.meaning.source_scope == owner)
            .flat_map(|o| &o.relations)
            .filter(|edge| {
                registry
                    .relation_kinds
                    .get(&edge.kind)
                    .is_some_and(|kind| kind.kind_name.starts_with("worth-aggregate-membership-"))
            })
            .map(|edge| edge.id)
            .collect())
    }

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
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
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

impl RelationalRuntime {
    /// Descriptive publication sources reached for one real Query output record.
    #[doc(hidden)]
    pub fn current_ownership_sources_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: EntityId,
    ) -> Result<Vec<EntityId>, TransactionCommitError> {
        let current = super::select_current(execution, basis, &[RecordRef::Entity(record)])?;
        Ok(current
            .origins()
            .map(|origin| origin.meaning.source_scope)
            .collect())
    }
}

//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn resolve_aftermath_causality_in_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        pending: &WorthQueryPendingAftermathCausality,
        outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        Self::resolve_aftermath_causality_in_snapshot(
            self,
            execution,
            runtime,
            basis,
            snapshot,
            pending,
            outcome_identity,
        )
    }

    /// The canonical correction fact `entity` holds at `basis`, or `None`
    /// when no live correction fact is there (tree-based undo's reversal
    /// reader, public #56 ruling (u)).
    #[doc(hidden)]
    pub fn correction_fact_at_basis_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        entity: worth_relational::facade::identity::EntityId,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        let index = self
            .graph
            .layout_ref()
            .provider_aftermath_causality()
            .lineage
            .reversed_source_index;
        self.with_correction_snapshot(execution, basis, index, |runtime, snapshot, layout| {
            match canonical_record::read(runtime, snapshot, layout, execution, entity, owner) {
                Ok(fact) => Ok(Some(fact)),
                // The read refuses a record that is not a live fact.
                Err(WorthQueryAftermathCausalityReadDenial::Unavailable) => Ok(None),
                Err(other) => Err(other),
            }
        })
    }

    /// Rewrites one canonical field of the correction fact `entity` in an
    /// ordinary commit on `basis`'s branch, no navigation, and reads the fact
    /// at that commit. Ruling (u) relaxes the canonical read only for a
    /// navigation that rematerialized the fact; this is its negative court.
    #[doc(hidden)]
    pub fn rewrite_and_read_correction_fact_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        entity: worth_relational::facade::identity::EntityId,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        use worth_relational::facade::transactions::{
            EntityMutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
        };
        let locator = self
            .graph
            .layout_ref()
            .provider_aftermath_causality()
            .outcome_identity_locator
            .clone();
        let rewritten = self.graph.with_runtime(execution, |runtime| {
            let mut transaction = runtime
                .begin_branch_transaction(
                    basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("an ordinary transaction on the fact's branch");
            transaction
                .push_batch(
                    WorkerIntentBatch::new("ruling-u-fact-rewrite").push(MutationIntent::Entity(
                        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                            entity_id: entity,
                            fields: AspectFieldPatch::from(BTreeMap::from([(
                                locator,
                                AspectValue::UInt64(u64::MAX),
                            )])),
                        }),
                    )),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("the rewrite stages");
            let committed = transaction.commit(execution, runtime).expect("the rewrite commits");
            crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
            let identity = runtime
                .branch_identity(&committed.commit.branch_id)
                .expect("the rewritten branch is owner-issued");
            runtime.admit_branch_basis(&identity).expect("the rewrite's basis admits")
        });
        self.correction_fact_at_basis_for_test(execution, &rewritten, entity, owner)
    }
}

//! A real Query publication, recaptured as damaged external current content.
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
    transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    #[doc(hidden)]
    pub fn probe_cold_current_ownership_pairing_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        unrelated: EntityId,
    ) -> Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError> {
        let fail =
            |detail| crate::mvcc::origin_denial(format!("actual cold ownership court: {detail:?}"));
        let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
        let publication = current
            .origins()
            .find(|origin| origin.meaning.source_scope == owner)
            .map(|origin| origin.commit.commit_id)
            .ok_or_else(|| fail("real Query publication absent".to_owned()))?;
        assert_eq!(
            owner.partition_id, unrelated.partition_id,
            "unrelated access must load the corrupted region"
        );
        let mut answers = Vec::new();
        for change in [3, 4] {
            let (native, source) = super::cold_checkpoint_source::Source::capture_corrupted(
                execution,
                self,
                publication,
                change,
            );
            let recovery = crate::durability::medium::RelationalDurableRecoverySource::new(
                Some(
                    crate::durability::medium::RelationalDurableCheckpointSource::new(
                        native.covered_position().expect("recorded position"),
                        native
                            .covered_branch_transitions()
                            .expect("recorded branches"),
                        native,
                    ),
                ),
                Vec::new(),
                Vec::new(),
                source,
            );
            let mut cold = self.fork().map_err(|e| fail(format!("{e:?}")))?;
            let plan = cold
                .durability()
                .recovery_plan_from_source(
                    execution,
                    recovery,
                    crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .map_err(|e| fail(format!("recovery plan: {e:?}")))?;
            cold.durability_recovery()
                .recover(execution, plan)
                .map_err(|e| fail(format!("recovery: {e:?}")))?;
            let identity = cold
                .branch_identity(basis.descriptor().branch_id())
                .map_err(|e| fail(format!("{e:?}")))?;
            let (_, basis) = cold
                .observe_branch(&identity)
                .map_err(|e| fail(format!("{e:?}")))?;
            assert!(
                matches!(
                    basis.inner.root.probe_head_region(owner.partition_id),
                    crate::storage::residency::PartitionHeadProbe::Cold
                ),
                "the pairing court must reach a genuinely cold current member after recovery"
            );
            // No owning scope or member is selected. Only the cold member's
            // independent presence/row validation can discover this damage.
            answers.push(
                super::select_current(execution, &basis, &[RecordRef::Entity(unrelated)])
                    .map(|_| ()),
            );
        }
        Ok(answers)
    }
}

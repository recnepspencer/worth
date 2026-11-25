//! Bind cold ownership courts to the installed checkpoint, recovery and preparation services.
use worth_relational_durability::internal_port::durability::data::native_checkpoint::roster_corruption::RelationalNativeCheckpointOwnerPort as _;
use super::RelationalRuntime;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::durability::data::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
use crate::durability::published_medium::RelationalDurableRecoverySource;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::EntityId;
use crate::publication::ownership::{
    cold_checkpoint_source::Source, cold_pairing_court, locality_court,
};
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::outcomes::transaction_commit_error::{
    origin_denial, TransactionCommitError,
};
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;

impl RelationalRuntime {
    #[doc(hidden)]
    pub fn probe_cold_current_ownership_pairing_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        unrelated: EntityId,
    ) -> Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError> {
        let fail = |detail| origin_denial(format!("actual cold ownership court: {detail:?}"));
        let publication = cold_pairing_court::selected_publication(execution, basis, owner)?;
        assert_eq!(
            owner.partition_id, unrelated.partition_id,
            "unrelated access must load the corrupted region"
        );
        let mut answers = Vec::new();
        for change in [3, 4] {
            let source = self.capture_ownership_probe(execution, Some((publication, change)));
            let (_cold, selected) = self.restore_ownership_probe(
                execution,
                source,
                basis.descriptor().branch_id(),
                fail,
            )?;
            answers.push(cold_pairing_court::observe_unrelated(
                execution, &selected, owner, unrelated,
            ));
        }
        Ok(answers)
    }

    #[doc(hidden)]
    pub fn probe_current_ownership_locality_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        seed_fields: impl Fn(u64) -> AspectFieldPatch,
        value_fields: impl Fn(u64) -> AspectFieldPatch,
    ) -> Result<
        crate::publication::ownership::CurrentOwnershipLocalityForTest,
        TransactionCommitError,
    > {
        let fail = |detail: String| locality_court::locality_error(detail);
        let kind = locality_court::owner_kind(basis, owner)?;
        let branch = BranchId("current-ownership-locality-court".into());
        let (_, source) = self
            .observe_fork_source(basis.descriptor().branch_id())
            .map_err(|cause| fail(format!("{cause:?}")))?;
        self.fork_branch(execution, branch.clone(), source)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let identity = self
            .branch_identity(&branch)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let (_, seed) = self
            .observe_branch(&identity)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let mut setup = self
            .begin_branch_transaction(&seed, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let (batch, first) = locality_court::seed_partitions(kind, seed_fields);
        setup
            .push_batch(batch, execution.allocation_policy())
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        let seeded = setup.commit(execution, self)?;
        let unrelated = seeded
            .created_entity(&first)
            .ok_or_else(|| fail("ordinary seed did not resolve".into()))?;
        drop(seeded);
        // Leave one ordinary source edit at the head so older unrelated regions stay cold.
        let (_, head) = self
            .observe_branch(&identity)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let mut ordinary = self
            .begin_branch_transaction(&head, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| fail(format!("{cause:?}")))?;
        ordinary
            .push_batch(
                locality_court::field_edit("ordinary-source-head", owner, value_fields(11)),
                execution.allocation_policy(),
            )
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        drop(ordinary.commit(execution, self)?);
        let recovery = self.capture_ownership_probe(execution, None);
        let (cold, selected) = self.restore_ownership_probe(execution, recovery, &branch, fail)?;
        locality_court::inspect_recovered_locality(
            execution,
            &selected,
            owner,
            unrelated,
            value_fields,
            |request, batch| {
                let mut transaction = cold
                    .begin_branch_transaction(
                        &selected,
                        crate::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .map_err(|cause| fail(format!("{cause:?}")))?;
                transaction
                    .push_batch(batch, request.allocation_policy())
                    .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
                Ok(cold
                    .prepare_branch_transaction(&request, transaction)
                    .map(|_| ()))
            },
        )
    }

    fn capture_ownership_probe(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        corruption: Option<(CommitId, u8)>,
    ) -> RelationalDurableRecoverySource {
        let checkpoint = self
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                },
            )
            .expect("the actual private branch records");
        let checkpoint = if let Some((publication, change)) = corruption {
            // Re-record against the original capture's own external members.
            checkpoint
                .corrupt_current_publication_roster_external_for_test(
                    execution,
                    Source::members_of(&checkpoint),
                    publication,
                    change,
                )
                .expect("the real recorder emits damaged external current members without admitting them")
        } else {
            checkpoint
        };
        Source::archive(checkpoint, || {
            self.durability().canonical_commit_artifacts(execution)
        })
    }

    fn restore_ownership_probe(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        recovery: RelationalDurableRecoverySource,
        branch: &BranchId,
        fail: impl Fn(String) -> TransactionCommitError,
    ) -> Result<(Self, AdmittedRelationalBranchBasis), TransactionCommitError> {
        let mut cold = self.fork().map_err(|cause| fail(format!("{cause:?}")))?;
        let plan = cold
            .durability()
            .recovery_plan_from_source(
                execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .map_err(|cause| fail(format!("{cause:?}")))?;
        cold.durability_recovery()
            .recover(execution, plan)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let identity = cold
            .branch_identity(branch)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        let (_, basis) = cold
            .observe_branch(&identity)
            .map_err(|cause| fail(format!("{cause:?}")))?;
        Ok((cold, basis))
    }
}

//! Locality court: real Query custody, ordinary private-branch edits and cold recovery.
use super::cold_checkpoint_source as source;
use crate::{
    branch::AdmittedRelationalBranchBasis, history::data::BranchId, identity::data::*,
    runtime::RelationalRuntime, storage::overlay::CurrentPartitionAccess, transactions::data::*,
};
use worth_execution::ExecutionRequest;
#[derive(Debug)]
pub struct CurrentOwnershipLocalityForTest {
    pub unrelated_roster_records: usize,
    pub owned_publications: usize,
    pub cold_before: usize,
    pub cold_after_unrelated: usize,
    pub cold_after_owned: usize,
    pub owned_refusal: crate::publication::data::PublicationError,
}
impl RelationalRuntime {
    #[doc(hidden)]
    pub fn probe_current_ownership_locality_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        seed_fields: impl Fn(u64) -> AspectFieldPatch,
        value_fields: impl Fn(u64) -> AspectFieldPatch,
    ) -> Result<CurrentOwnershipLocalityForTest, TransactionCommitError> {
        let fail = |detail| {
            crate::mvcc::origin_denial(format!("current ownership locality court: {detail}"))
        };
        let kind = basis
            .inner
            .root
            .get_current_partition(owner.partition_id)
            .and_then(|p| p.entity_arena().get(&owner))
            .and_then(|r| r.kind_id())
            .ok_or_else(|| fail("real Query owner absent".into()))?;
        let branch = BranchId("current-ownership-locality-court".into());
        let (_, source) = self
            .observe_fork_source(basis.descriptor().branch_id())
            .map_err(|e| fail(format!("{e:?}")))?;
        self.fork_branch(execution, branch.clone(), source)
            .map_err(|e| fail(format!("{e:?}")))?;
        let identity = self
            .branch_identity(&branch)
            .map_err(|e| fail(format!("{e:?}")))?;
        let (_, seed) = self
            .observe_branch(&identity)
            .map_err(|e| fail(format!("{e:?}")))?;
        let mut tx = self
            .begin_branch_transaction(&seed, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|e| fail(format!("{e:?}")))?;
        let mut batch = WorkerIntentBatch::new("unrelated-native-partitions");
        let mut first = None;
        for part in 1..=64 {
            let partition = PartitionId(1000 + part);
            let key = crate::symbols::data::ClientKey::raw(format!("unrelated-{part}"));
            let created = CreatedEntityRef {
                partition_id: partition,
                kind_id: kind,
                client_key: key.clone(),
            };
            if first.is_none() {
                first = Some(created);
            }
            batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: partition,
                kind_id: kind,
                client_key: key,
                fields: seed_fields(50_000 + u64::from(part)),
            })));
        }
        tx.push_batch(batch, execution.allocation_policy())
            .map_err(|e| TransactionCommitError::conflict(e.into_conflict()))?;
        let seeded = tx.commit(execution, self)?;
        let unrelated = seeded
            .created_entity(first.as_ref().unwrap())
            .ok_or_else(|| fail("ordinary seed did not resolve".into()))?;
        drop(seeded);
        // Leave one ordinary source edit at the head. Recovery may admit the
        // head envelope's own regions; the earlier unrelated regions must stay cold.
        let (_, head) = self
            .observe_branch(&identity)
            .map_err(|e| fail(format!("{e:?}")))?;
        let mut ordinary = self
            .begin_branch_transaction(&head, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|e| fail(format!("{e:?}")))?;
        ordinary
            .push_batch(
                WorkerIntentBatch::new("ordinary-source-head").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: owner,
                        fields: value_fields(11),
                    }),
                )),
                execution.allocation_policy(),
            )
            .map_err(|e| TransactionCommitError::conflict(e.into_conflict()))?;
        drop(ordinary.commit(execution, self)?);
        let (checkpoint, source) = source::Source::capture(execution, self);
        let recovery = crate::durability::medium::RelationalDurableRecoverySource::new(
            Some(
                crate::durability::medium::RelationalDurableCheckpointSource::new(
                    checkpoint.covered_position().expect("recorded position"),
                    checkpoint
                        .covered_branch_transitions()
                        .expect("recorded branch coverage"),
                    checkpoint,
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
            .map_err(|e| fail(format!("{e:?}")))?;
        cold.durability_recovery()
            .recover(execution, plan)
            .map_err(|e| fail(format!("{e:?}")))?;
        let identity = cold
            .branch_identity(&branch)
            .map_err(|e| fail(format!("{e:?}")))?;
        let (_, basis) = cold
            .observe_branch(&identity)
            .map_err(|e| fail(format!("{e:?}")))?;
        let root = &basis.inner.root;
        let count = || {
            (1002..=1064)
                .filter(|p| {
                    matches!(
                        root.probe_head_region(PartitionId(*p)),
                        crate::storage::residency::PartitionHeadProbe::Cold
                    )
                })
                .count()
        };
        let cold_before = count();
        // Bound this certification caller's entire recovered edit wave. Native's
        // fence and custody stages still consume the original request account.
        execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(Some(100_000)).run_with_checkpoint(
                    lease,
                    None,
                    |_| {
                        let bounded = *execution;
                        let selected = super::select_current(
                            &bounded,
                            &basis,
                            &[RecordRef::Entity(unrelated)],
                        )?;
                        let unrelated_roster_records = selected.publications.len()
                            + selected.catalogs.len()
                            + selected.positions.len()
                            + selected.edges.len()
                            + selected.links.len();
                        let mut tx = cold
                            .begin_branch_transaction(
                                &basis,
                                crate::mvcc::RelationalTransactionIntent::ordinary(),
                            )
                            .map_err(|e| fail(format!("{e:?}")))?;
                        tx.push_batch(
                            WorkerIntentBatch::new("unrelated-value-edit").push(
                                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                                    UpdateEntityFieldsIntent {
                                        entity_id: unrelated,
                                        fields: value_fields(7),
                                    },
                                )),
                            ),
                            bounded.allocation_policy(),
                        )
                        .map_err(|e| TransactionCommitError::conflict(e.into_conflict()))?;
                        let prepared = cold.prepare_branch_transaction(&bounded, tx)?;
                        drop(prepared);
                        let cold_after_unrelated = count();
                        let selected =
                            super::select_current(&bounded, &basis, &[RecordRef::Entity(owner)])?;
                        let owned_publications = selected.publications.len();
                        let member = selected
                            .origins()
                            .find(|o| o.meaning.source_scope == owner)
                            .and_then(|o| o.outputs.first())
                            .copied()
                            .ok_or_else(|| fail("real Query output absent".into()))?;
                        let mut tx = cold
                            .begin_branch_transaction(
                                &basis,
                                crate::mvcc::RelationalTransactionIntent::ordinary(),
                            )
                            .map_err(|e| fail(format!("{e:?}")))?;
                        tx.push_batch(
                            WorkerIntentBatch::new("external-owned-edit").push(
                                MutationIntent::Entity(EntityMutationIntent::Delete(
                                    DeleteEntityIntent { entity_id: member },
                                )),
                            ),
                            bounded.allocation_policy(),
                        )
                        .map_err(|e| TransactionCommitError::conflict(e.into_conflict()))?;
                        let refusal = cold
                            .prepare_branch_transaction(&bounded, tx)
                            .err()
                            .ok_or_else(|| fail("external delete unexpectedly prepared".into()))?;
                        let TransactionCommitError::Publication {
                            error: owned_refusal,
                            ..
                        } = refusal
                        else {
                            return Err(refusal);
                        };
                        Ok(CurrentOwnershipLocalityForTest {
                            unrelated_roster_records,
                            owned_publications,
                            cold_before,
                            cold_after_unrelated,
                            cold_after_owned: count(),
                            owned_refusal,
                        })
                    },
                )
            })
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?
            .0
    }
}

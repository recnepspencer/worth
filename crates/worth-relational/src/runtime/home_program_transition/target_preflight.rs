//! Native-owned exact pre-intent target cold-reconstruction budget admission
//! for Home program transitions.
//!
//! Stages any ordered `BeforeTarget` prefix steps under predecessor contracts
//! and then the `Target` step under the compiled target schema — strictly
//! against detached identity, allocator, symbol, and sequence state (never
//! writing to Store, consuming index generation IDs, or mutating live branch
//! or allocator state) — and executes Native's exact `ChangedRecords::cold` +
//! `cold_projection::project` + per-index publication work accounting against
//! the target index catalog.

use std::collections::{BTreeMap, BTreeSet};

use worth_relational_storage::internal_port::storage::overlay::PartitionAccess as _;

use super::*;
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};
use crate::transactions::data::{TransactionCommitError, WorkerIntentBatch};

struct PreflightOverlayAccumulator {
    staged_partitions:
        BTreeMap<crate::identity::data::PartitionId, crate::storage::overlay::PartitionState>,
    all_partitions: BTreeSet<crate::identity::data::PartitionId>,
    entity_slot_count: usize,
    relation_slot_count: usize,
}

impl PreflightOverlayAccumulator {
    fn from_predecessor(root: &crate::branch::RelationalBranchRoot) -> Self {
        Self {
            staged_partitions: BTreeMap::new(),
            all_partitions: root.partition_ids_iter().collect(),
            entity_slot_count: root.entity_slot_count(),
            relation_slot_count: root.relation_slot_count(),
        }
    }

    fn merge_step(
        &mut self,
        predecessor_root: &crate::branch::RelationalBranchRoot,
        committed_partitions: BTreeMap<
            crate::identity::data::PartitionId,
            (
                crate::storage::overlay::PartitionState,
                crate::storage::overlay::PartitionMutationJournal,
            ),
        >,
    ) {
        for (partition_id, (mut overlay, journal)) in committed_partitions {
            overlay.clear_runtime_pin_counters();
            let (old_e, old_r, merged) = if let Some(current) = self
                .staged_partitions
                .get(&partition_id)
                .or_else(|| predecessor_root.get_partition(partition_id))
            {
                let old_e = current.entity_arena.slot_count();
                let old_r = current.relation_arena.slot_count();
                let mut partition = current.clone();
                partition.merge_overlay_from_owned(&mut overlay, &journal);
                (old_e, old_r, partition)
            } else {
                (0, 0, overlay)
            };
            self.entity_slot_count = self
                .entity_slot_count
                .saturating_sub(old_e)
                .saturating_add(merged.entity_arena.slot_count());
            self.relation_slot_count = self
                .relation_slot_count
                .saturating_sub(old_r)
                .saturating_add(merged.relation_arena.slot_count());
            self.all_partitions.insert(partition_id);
            self.staged_partitions.insert(partition_id, merged);
        }
    }
}

impl RelationalRuntime {
    /// Admit `budget` against the exact staged successor cold-reconstruction
    /// work (`ChangedRecords::cold`, `cold_projection::project`, and per-index
    /// publication charges) before publishing any durable transition intent.
    ///
    /// Ordered `before_target_steps` are staged sequentially under predecessor
    /// contracts onto the detached overlay before `target_step_batches` are
    /// staged under `target` contracts.
    #[allow(clippy::too_many_arguments)]
    pub fn preflight_home_program_target_index_budget(
        &self,
        request: &ExecutionRequest<'_, '_>,
        predecessor: &AdmittedRelationalBranchBasis,
        target: &PreparedRelationalHomeProgramContracts,
        schema_transition: Option<(ProposedSchemaTransition, Option<SchemaReconciliationPolicy>)>,
        before_target_steps: &[&[WorkerIntentBatch]],
        target_step_batches: &[WorkerIntentBatch],
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        if target.meaning.runtime != self.runtime_instance_id()
            || predecessor.identity().runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::ForeignCandidate,
                work: DerivedIndexMaintenanceWork::default(),
            });
        }

        let predecessor_root = &predecessor.owner_inner().root;
        if (predecessor_root.commit_id().is_none() && before_target_steps.is_empty())
            || predecessor_root.schema_authority().authority_digest()
                == target.meaning.schema.authority_digest()
        {
            return Ok(DerivedIndexMaintenanceWork::default());
        }

        let fallback_catalog = self.indexes.definition_read_binding().snapshot();
        if !worth_relational_indexes::internal_port::indexes::authority::maintenance::cold_preflight::target_catalog_is_non_empty(
            target.index_definitions.as_deref(),
            fallback_catalog.as_ref(),
        ) {
            return Ok(DerivedIndexMaintenanceWork::default());
        }

        let epoch = self.configuration_binding().snapshot();
        if !target.receiver_epoch.same_epoch(&epoch) {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::BeforeRootMismatch,
                work: DerivedIndexMaintenanceWork::default(),
            });
        }

        let mut detached_prep = self.preparation_runtime_snapshot().detached_for_preflight();
        let mut accumulator = PreflightOverlayAccumulator::from_predecessor(predecessor_root);

        if !before_target_steps.is_empty() && !accumulator.all_partitions.is_empty() {
            predecessor_root
                .demand_regions_full(
                    request,
                    &detached_prep.services.symbols,
                    accumulator.all_partitions.clone(),
                )
                .map_err(|failure| {
                    map_staging_error(
                        crate::authority::commit::preparation::storage_denial::storage_read_denial(
                            predecessor,
                            failure,
                        ),
                    )
                })?;
        }

        for step_batches in before_target_steps {
            if step_batches.is_empty() {
                continue;
            }
            let (_version, next_symbols, committed_partitions) = self.stage_preflight_step(
                request,
                predecessor,
                &detached_prep,
                crate::mvcc::RelationalTransactionIntent::ordinary(),
                None,
                None,
                step_batches,
                &accumulator.staged_partitions,
            )?;
            detached_prep.services.symbols = next_symbols;
            accumulator.merge_step(predecessor_root, committed_partitions);
        }

        let mut target_intent = crate::mvcc::RelationalTransactionIntent::ordinary();
        if let Some((transition, policy)) = schema_transition {
            target_intent = target_intent.with_schema_transition(transition, policy);
        }
        let target_schema_input = Some(
            target.meaning.schema.continuity_input(
                epoch
                    .config
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
            ),
        );
        let target_admission = Some(AdmittedRelationalHomeProgramTarget::for_preflight(target));
        let (proposed_version, _next_symbols, committed_partitions) = self.stage_preflight_step(
            request,
            predecessor,
            &detached_prep,
            target_intent,
            target_schema_input,
            target_admission,
            target_step_batches,
            &accumulator.staged_partitions,
        )?;
        accumulator.merge_step(predecessor_root, committed_partitions);

        let staged = worth_relational_indexes::internal_port::indexes::authority::maintenance::cold_preflight::StagedHomeTargetSuccessor {
            predecessor_root: Arc::clone(predecessor_root),
            version_id: proposed_version,
            schema_authority: Arc::clone(&target.meaning.schema),
            staged_partitions: accumulator.staged_partitions,
            entity_slot_count: accumulator.entity_slot_count,
            relation_slot_count: accumulator.relation_slot_count,
            region_count: accumulator.all_partitions.len(),
        };
        worth_relational_indexes::internal_port::indexes::authority::maintenance::schema_transition::admit_home_target_cold_reconstruction_budget(
            request,
            staged,
            target.index_definitions.as_deref(),
            fallback_catalog.as_ref(),
            budget,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn stage_preflight_step(
        &self,
        request: &ExecutionRequest<'_, '_>,
        predecessor: &AdmittedRelationalBranchBasis,
        detached_prep: &crate::runtime::RelationalPreparationRuntime,
        intent: crate::mvcc::RelationalTransactionIntent,
        schema_authority_input: Option<crate::schema::SchemaContinuityAuthorityInput>,
        home_target: Option<Arc<AdmittedRelationalHomeProgramTarget>>,
        step_batches: &[WorkerIntentBatch],
        prior_staged_partitions: &BTreeMap<
            crate::identity::data::PartitionId,
            crate::storage::overlay::PartitionState,
        >,
    ) -> Result<
        (
            crate::identity::data::VersionId,
            crate::runtime::RuntimeSymbolTable,
            BTreeMap<
                crate::identity::data::PartitionId,
                (
                    crate::storage::overlay::PartitionState,
                    crate::storage::overlay::PartitionMutationJournal,
                ),
            >,
        ),
        DerivedIndexMaintenanceDenial,
    > {
        let mut transaction = self
            .begin_home_preflight_transaction(predecessor, intent, &detached_prep.services)
            .map_err(|_| DerivedIndexMaintenanceDenial {
                kind: Denial::BeforeRootMismatch,
                work: DerivedIndexMaintenanceWork::default(),
            })?;
        transaction.schema_authority_input = schema_authority_input;
        transaction.home_target = home_target;

        // Reserve exact cloned batch bytes under the request ceiling strictly
        // before any `batch.clone()` allocates.
        let _copy_reservation =
            worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::reserve_copy(
                request,
                batches_copy_bytes(step_batches),
            )
            .map_err(map_staging_error)?;
        for batch in step_batches {
            transaction
                .push_batch(batch.clone(), request.allocation_policy())
                .map_err(map_staging_denial)?;
        }

        let step_prep = detached_prep
            .select_home_contracts(
                &transaction.basis,
                transaction.home_target.as_ref(),
                transaction.recorded_result_schema(),
            )
            .map_err(map_staging_error)?;
        transaction
            .demand_preparation_regions(
                request,
                &step_prep.services.symbols,
                &step_prep.index_definitions,
                &step_prep.mirror_residency,
            )
            .map_err(map_staging_error)?;
        step_prep
            .stage_home_target_preflight_partitions(
                request,
                &mut transaction,
                prior_staged_partitions,
            )
            .map_err(map_staging_error)
    }
}

fn batches_copy_bytes(batches: &[WorkerIntentBatch]) -> Option<usize> {
    use worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::{
        intent_bytes, slots,
    };
    let mut total = slots::<WorkerIntentBatch>(batches.len())?;
    for batch in batches {
        total = total
            .checked_add(batch.name.len())?
            .checked_add(batch.partition_key.as_ref().map_or(0, String::len))?
            .checked_add(intent_bytes(&batch.intents)?)?;
    }
    Some(total)
}

fn map_staging_denial(
    denial: crate::mvcc::RelationalTransactionStagingDenial,
) -> DerivedIndexMaintenanceDenial {
    use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
    let is_physical_alloc = matches!(
        denial.allocation_denial().map(|d| d.classify()),
        Some(worth_execution::ExecutionAllocationDenialClass::Physical(_))
    );
    if is_physical_alloc {
        return DerivedIndexMaintenanceDenial {
            kind: Denial::Execution(
                crate::execution::RelationalExecutionDenialCause::AllocationFailed,
            ),
            work: DerivedIndexMaintenanceWork::default(),
        };
    }
    let error = crate::execution::allocation_commit_error(TransactionCommitError::conflict(
        denial.into_conflict(),
    ));
    map_staging_error(error)
}

fn map_staging_error(error: TransactionCommitError) -> DerivedIndexMaintenanceDenial {
    use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};
    let kind = match error {
        TransactionCommitError::Execution {
            denial:
                CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(cause),
                    ..
                },
            ..
        } => Denial::Execution(cause),
        TransactionCommitError::Publication { error, .. } => {
            if let Some(indexes) = error.index_maintenance_denial() {
                return indexes.clone();
            }
            Denial::BeforeRootMismatch
        }
        _ => Denial::BeforeRootMismatch,
    };
    DerivedIndexMaintenanceDenial {
        kind,
        work: DerivedIndexMaintenanceWork::default(),
    }
}

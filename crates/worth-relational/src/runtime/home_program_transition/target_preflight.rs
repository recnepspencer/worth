//! Native-owned exact pre-intent target cold-reconstruction budget admission
//! for Home program transitions.
//!
//! Stages the target step's mutation batches in memory under the compiled
//! target schema (without writing to Store, consuming index generation IDs,
//! or mutating live branch state) and executes Native's exact
//! `ChangedRecords::cold` + `cold_projection::project` + per-index publication
//! work accounting against the target index catalog.

use std::collections::BTreeMap;

use worth_relational_storage::internal_port::storage::overlay::PartitionAccess as _;

use super::*;
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};
use crate::transactions::data::{TransactionCommitError, WorkerIntentBatch};

impl RelationalRuntime {
    /// Admit `budget` against the exact staged successor cold-reconstruction
    /// work (`ChangedRecords::cold`, `cold_projection::project`, and per-index
    /// publication charges) before publishing any durable transition intent.
    pub fn preflight_home_program_target_index_budget(
        &self,
        request: &ExecutionRequest<'_, '_>,
        predecessor: &AdmittedRelationalBranchBasis,
        target: &PreparedRelationalHomeProgramContracts,
        schema_transition: Option<(ProposedSchemaTransition, Option<SchemaReconciliationPolicy>)>,
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
        if predecessor_root.commit_id().is_none()
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

        let mut intent = crate::mvcc::RelationalTransactionIntent::ordinary();
        if let Some((transition, policy)) = schema_transition {
            intent = intent.with_schema_transition(transition, policy);
        }
        let mut transaction = self
            .begin_home_preflight_transaction(predecessor, intent)
            .map_err(|_| DerivedIndexMaintenanceDenial {
                kind: Denial::BeforeRootMismatch,
                work: DerivedIndexMaintenanceWork::default(),
            })?;
        transaction.schema_authority_input = Some(
            target.meaning.schema.continuity_input(
                epoch
                    .config
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
            ),
        );
        transaction.home_target = Some(AdmittedRelationalHomeProgramTarget::for_preflight(target));
        for batch in target_step_batches {
            transaction
                .push_batch(batch.clone(), request.allocation_policy())
                .map_err(|_| DerivedIndexMaintenanceDenial {
                    kind: Denial::Execution(
                        crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                    ),
                    work: DerivedIndexMaintenanceWork::default(),
                })?;
        }

        let prep_runtime = self
            .preparation_runtime_snapshot()
            .select_home_contracts(
                &transaction.basis,
                transaction.home_target.as_ref(),
                transaction.recorded_result_schema(),
            )
            .map_err(map_staging_error)?;
        transaction
            .demand_preparation_regions(
                request,
                &prep_runtime.services.symbols,
                &prep_runtime.index_definitions,
                &prep_runtime.mirror_residency,
            )
            .map_err(map_staging_error)?;

        let (proposed_version, committed_partitions) = prep_runtime
            .stage_home_target_preflight_partitions(request, &mut transaction)
            .map_err(map_staging_error)?;

        let mut staged_partitions = BTreeMap::new();
        let mut entity_slot_count = predecessor_root.entity_slot_count();
        let mut relation_slot_count = predecessor_root.relation_slot_count();
        let mut all_partitions: std::collections::BTreeSet<crate::identity::data::PartitionId> =
            predecessor_root.partition_ids_iter().collect();

        for (partition_id, (mut overlay, journal)) in committed_partitions {
            overlay.clear_runtime_pin_counters();
            let (old_e, old_r, merged) =
                if let Some(current) = predecessor_root.get_partition(partition_id) {
                    let old_e = current.entity_arena.slot_count();
                    let old_r = current.relation_arena.slot_count();
                    let mut partition = current.clone();
                    partition.merge_overlay_from_owned(&mut overlay, &journal);
                    (old_e, old_r, partition)
                } else {
                    (0, 0, overlay)
                };
            entity_slot_count = entity_slot_count
                .saturating_sub(old_e)
                .saturating_add(merged.entity_arena.slot_count());
            relation_slot_count = relation_slot_count
                .saturating_sub(old_r)
                .saturating_add(merged.relation_arena.slot_count());
            all_partitions.insert(partition_id);
            staged_partitions.insert(partition_id, merged);
        }

        let staged = worth_relational_indexes::internal_port::indexes::authority::maintenance::cold_preflight::StagedHomeTargetSuccessor {
            predecessor_root: Arc::clone(predecessor_root),
            version_id: proposed_version,
            schema_authority: Arc::clone(&target.meaning.schema),
            staged_partitions,
            entity_slot_count,
            relation_slot_count,
            region_count: all_partitions.len(),
        };
        worth_relational_indexes::internal_port::indexes::authority::maintenance::schema_transition::admit_home_target_cold_reconstruction_budget(
            request,
            staged,
            target.index_definitions.as_deref(),
            fallback_catalog.as_ref(),
            budget,
        )
    }
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

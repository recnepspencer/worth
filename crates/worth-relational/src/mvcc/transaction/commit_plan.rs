use std::collections::BTreeMap;

use crate::authority::intent_merge::{
    canonical_intent_key, collect_created_entity_refs, collect_deleted_relation_ids,
    detect_conflicting_updates, validate_branch_locality, validate_intent,
};
use crate::capabilities::{InstrumentationSource, RuntimeConfigSource};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{CommitConflict, ConflictClass, MergedCommitPlan};
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MergedPlanPreparationTiming {
    pub(crate) validation_micros: u64,
    pub(crate) sort_micros: u64,
    pub(crate) conflict_detection_micros: u64,
}

impl crate::mvcc::BranchBoundRelationalTransaction {
    pub(crate) fn validate_staged_branch_locality(
        &self,
        current_state: &impl CurrentPartitionAccess,
        interner: &crate::symbols::data::StringInterner,
    ) -> Result<(), CommitConflict> {
        validate_branch_locality(current_state, self.batches(), interner)
    }

    pub(crate) fn build_merged_plan_for_state(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        current_state: &impl CurrentPartitionAccess,
        intents: super::OwnedNormalizedPlan,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::OwnedNormalizedPlan, crate::transactions::data::TransactionCommitError> {
        self.build_merged_plan_for_state_with_timing(runtime, current_state, intents, request)
            .map(|(plan, _)| plan)
    }

    pub(crate) fn build_merged_plan_for_state_with_timing(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        current_state: &impl CurrentPartitionAccess,
        mut intents: super::OwnedNormalizedPlan,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (super::OwnedNormalizedPlan, MergedPlanPreparationTiming),
        crate::transactions::data::TransactionCommitError,
    > {
        let created_entities = collect_created_entity_refs(&intents.plan().merged_intents);
        let deleted_relations = collect_deleted_relation_ids(&intents.plan().merged_intents);
        let validation_started = Instant::now();
        for intent in &intents.plan().merged_intents {
            validate_intent(
                current_state,
                self.mutation_schema_authority().as_ref(),
                runtime.runtime_config().storage.cross_context_policy,
                runtime.runtime_instrumentation(),
                &created_entities,
                &deleted_relations,
                intent,
            )?;
        }
        let validation_micros = validation_started.elapsed().as_micros() as u64;
        let sort_started = Instant::now();
        intents.intents_mut().sort_by_key(canonical_intent_key);
        let sort_micros = sort_started.elapsed().as_micros() as u64;
        let conflict_started = Instant::now();
        detect_conflicting_updates(&intents.plan().merged_intents)?;
        if self.authored_coupling.is_some() {
            let needs_copy = self
                .prepared_coupled_commands
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_none();
            if needs_copy {
                let prepared =
                    super::OwnedNormalizedPlan::shared_copy_for_request(intents.plan(), request)?;
                let mut slot = self
                    .prepared_coupled_commands
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if slot.is_none() {
                    *slot = Some(prepared);
                }
            }
        }
        Ok((
            intents,
            MergedPlanPreparationTiming {
                validation_micros,
                sort_micros,
                conflict_detection_micros: conflict_started.elapsed().as_micros() as u64,
            },
        ))
    }

    pub(crate) fn normalized_intents_for_merge(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::OwnedNormalizedPlan, crate::transactions::data::TransactionCommitError> {
        self.normalize_intents_for_merge(runtime, request)?;
        super::OwnedNormalizedPlan::copy_batches(self.transaction_id, self.batches(), request)
    }

    fn normalize_intents_for_merge(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        request
            .consult()
            .map_err(super::normalized_plan::request_denial)?;
        let policy = runtime.runtime_config().identity.client_key_symbol_policy;
        if !policy.interns_requested_strings() {
            return Ok(());
        }
        let mut result = Ok(());
        runtime.services.symbols.normalize_client_keys(|symbols| {
            let (entries, outcome) =
                self.overlay
                    .normalize_client_keys(&mut self.footprint, symbols, policy, request);
            result = outcome;
            entries
        });
        result
    }

    pub(crate) fn ensure_runtime_affinity(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_instance(runtime.runtime_instance_id())
    }

    pub(crate) fn ensure_runtime_affinity_for_runtime(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_instance(runtime.runtime_instance_id())
    }

    fn ensure_runtime_affinity_for_instance(
        &self,
        runtime_instance_id: u64,
    ) -> Result<(), CommitConflict> {
        if self.basis.identity().runtime_instance_id() == runtime_instance_id {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::ForeignRuntime {
                expected_runtime_instance_id: runtime_instance_id,
                actual_runtime_instance_id: self.basis.identity().runtime_instance_id(),
            }))
        }
    }

    pub(crate) fn ensure_current_basis(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity(runtime)?;
        if self.basis.is_current() {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::StaleValidationBasis {
                detail: "transaction basis is no longer current".to_owned(),
            }))
        }
    }

    pub(crate) fn ensure_current_basis_for_runtime(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_runtime(runtime)?;
        if self.basis.is_current() {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::StaleValidationBasis {
                detail: "transaction basis is no longer current".to_owned(),
            }))
        }
    }
}

pub(crate) fn bulk_reservations_for_plan(
    _state: &impl CurrentPartitionAccess,
    plan: &MergedCommitPlan,
) -> (usize, usize) {
    let mut entity_requests = BTreeMap::new();
    let mut relation_requests = BTreeMap::new();
    for intent in &plan.merged_intents {
        if let Some((partition_id, requested)) = intent.bulk_entity_reservation() {
            *entity_requests.entry(partition_id).or_insert(0usize) += requested;
        }
        if let Some((partition_id, requested)) = intent.bulk_relation_reservation() {
            *relation_requests.entry(partition_id).or_insert(0usize) += requested;
        }
    }

    let entity_reserved = entity_requests.into_values().sum();
    let relation_reserved = relation_requests.into_values().sum();
    (entity_reserved, relation_reserved)
}

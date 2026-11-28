//! Normalize staged commands and validate them against the selected immutable state.
use contracts::config::data::policies::CrossContextPolicy;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::transactions::data::canonical_intent_key::canonical_intent_key;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_commit_admission::internal_port::intent_admission::{
    collect_created_entity_refs, collect_deleted_relation_ids, detect_conflicting_updates,
    validate_intent,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

#[derive(Debug, Clone, Copy, Default)]
pub struct MergedPlanPreparationTiming {
    pub(crate) validation_micros: u64,
    pub(crate) sort_micros: u64,
    pub(crate) conflict_detection_micros: u64,
}

pub struct StagedPlanValidation<'a> {
    pub schema: &'a RelationalSchemaRegistry,
    pub cross_context: CrossContextPolicy,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub coupled_commands: Option<&'a Mutex<Option<Arc<OwnedNormalizedPlan>>>>,
}

impl StagedPlanValidation<'_> {
    pub fn validate(
        &self,
        current_state: &impl CurrentPartitionAccess,
        mut intents: OwnedNormalizedPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(OwnedNormalizedPlan, MergedPlanPreparationTiming), TransactionCommitError> {
        let created_entities = collect_created_entity_refs(&intents.plan().merged_intents);
        let deleted_relations = collect_deleted_relation_ids(&intents.plan().merged_intents);
        let validation_started = Instant::now();
        for intent in &intents.plan().merged_intents {
            let mut candidates_scanned = 0;
            let validation = validate_intent(
                current_state,
                self.schema,
                self.cross_context,
                &mut candidates_scanned,
                &created_entities,
                &deleted_relations,
                intent,
            );
            if candidates_scanned != 0 {
                self.instrumentation.count(|counters| {
                    counters.relation_identity_candidates_scanned += candidates_scanned;
                });
            }
            validation?;
        }
        let validation_micros = validation_started.elapsed().as_micros() as u64;
        let sort_started = Instant::now();
        intents.intents_mut().sort_by_key(canonical_intent_key);
        let sort_micros = sort_started.elapsed().as_micros() as u64;
        let conflict_started = Instant::now();
        detect_conflicting_updates(&intents.plan().merged_intents)?;
        if let Some(commands) = self.coupled_commands {
            let needs_copy = commands.lock().unwrap_or_else(|e| e.into_inner()).is_none();
            if needs_copy {
                let prepared =
                    OwnedNormalizedPlan::shared_copy_for_request(intents.plan(), request)?;
                let mut slot = commands.lock().unwrap_or_else(|e| e.into_inner());
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
}

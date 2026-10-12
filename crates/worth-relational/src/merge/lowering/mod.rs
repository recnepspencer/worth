mod aspect_outcomes;
mod decision_log;
mod denial_classification;
mod record_decision;
mod record_intents;
mod record_readiness;
mod resolution;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::merge::data::{
    LoweredMergePlan, LoweredMergePlanRecord, MergeExecutionReadiness, MergePlanningError,
    NormalizedRelationalMergeRequest, PolicyResolvedMergePlan,
};
use crate::merge::MergeAccess;
use crate::transactions::data::RecordRef;

use aspect_outcomes::lowered_aspect_outcomes_for_record;
use decision_log::{build_decision_log, build_decision_log_digest_basis};
use record_decision::record_decision_for_record;
use record_intents::{
    denial_bundle_for_record, execution_bundle_for_record, rejected_reason_for_record,
};
use record_readiness::{aggregate_record_readiness, summarize_lowered_records};
pub(crate) use resolution::{
    blocked_reason_for_deletion_class, blocked_reason_for_topology_resolution_class,
};
use resolution::{
    blocked_reason_for_record, executable_class_for_record, resolution_class_for_record,
};

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn lower_planning_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: NormalizedRelationalMergeRequest,
    ) -> Result<LoweredMergePlan, MergePlanningError> {
        let policy_plan = self.plan_policy_scope(execution, request)?;
        self.lower_policy_plan(policy_plan)
    }

    fn lower_policy_plan(
        &self,
        policy_plan: PolicyResolvedMergePlan,
    ) -> Result<LoweredMergePlan, MergePlanningError> {
        let causal_by_record = policy_plan
            .causal_annotations
            .iter()
            .map(|annotation| (annotation.record.clone(), annotation))
            .collect::<BTreeMap<RecordRef, _>>();
        let source_records_by_ref = policy_plan
            .source_records
            .iter()
            .map(|record| (record.record_ref.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let classifications_by_record = policy_plan
            .classifications
            .iter()
            .map(|classification| (classification.record.clone(), classification))
            .collect::<BTreeMap<_, _>>();
        let lowered_records = policy_plan
            .policy_records
            .iter()
            .map(|policy_record| {
                let causal = causal_by_record.get(&policy_record.record).ok_or_else(|| {
                    MergePlanningError::MissingCausalAnnotation {
                        record: policy_record.record.clone(),
                    }
                })?;
                let source_record = source_records_by_ref
                    .get(&policy_record.record)
                    .ok_or_else(|| MergePlanningError::MissingLoweringSourceRecord {
                        record: policy_record.record.clone(),
                    })?;
                let classification = classifications_by_record
                    .get(&policy_record.record)
                    .ok_or_else(
                        || MergePlanningError::MissingLoweringConflictClassification {
                            record: policy_record.record.clone(),
                        },
                    )?;
                let resolution_class = resolution_class_for_record(
                    policy_record.classification,
                    classification.relation_evidence.as_ref(),
                );
                let aspect_outcomes = lowered_aspect_outcomes_for_record(
                    self.runtime,
                    source_record,
                    policy_record,
                    resolution_class,
                )?;
                let policy_readiness = aspect_outcomes::readiness_for_policy_decision(
                    policy_record.proof_boundary.decision_boundary,
                );
                let aspect_readiness = if aspect_outcomes.is_empty() {
                    policy_readiness
                } else {
                    aggregate_record_readiness(aspect_outcomes.as_slice())
                };
                let readiness = match policy_readiness {
                    MergeExecutionReadiness::Rejected => MergeExecutionReadiness::Rejected,
                    MergeExecutionReadiness::Blocked => MergeExecutionReadiness::Blocked,
                    MergeExecutionReadiness::Admitted => aspect_readiness,
                };
                let lowered_action = record_intents::lowered_action_for_record(
                    policy_record.classification,
                    aspect_outcomes.as_slice(),
                    readiness,
                );
                let execution_bundle = execution_bundle_for_record(
                    policy_record.classification,
                    aspect_outcomes.as_slice(),
                    readiness,
                );
                let executable_class = executable_class_for_record(
                    resolution_class,
                    readiness,
                    execution_bundle.as_ref().map(|bundle| bundle.kind),
                );
                let denial_bundle = denial_bundle_for_record(
                    policy_record.classification,
                    resolution_class,
                    aspect_outcomes.as_slice(),
                    readiness,
                );
                let blocked_reason = blocked_reason_for_record(
                    policy_record.classification,
                    resolution_class,
                    aspect_outcomes.as_slice(),
                    readiness,
                );
                let rejected_reason =
                    rejected_reason_for_record(aspect_outcomes.as_slice(), readiness);
                let record_decision = record_decision_for_record(
                    readiness,
                    policy_record.classification,
                    resolution_class,
                    lowered_action,
                    blocked_reason,
                    rejected_reason,
                    execution_bundle.clone(),
                    denial_bundle.clone(),
                )?;
                Ok(LoweredMergePlanRecord {
                    record: policy_record.record.clone(),
                    target_record: policy_record.target_record.clone(),
                    classification: policy_record.classification,
                    resolution_class,
                    executable_class,
                    causal_disposition: causal.disposition,
                    applied_policies: policy_record.applied_policies.clone(),
                    policy_proof_boundary: policy_record.proof_boundary,
                    readiness,
                    record_decision,
                    lowered_action,
                    blocked_reason,
                    rejected_reason,
                    aspect_outcomes: Arc::from(aspect_outcomes),
                })
            })
            .collect::<Result<Vec<_>, MergePlanningError>>()?;
        let lowered_summary = summarize_lowered_records(Arc::from(lowered_records.clone()));
        let decision_log = build_decision_log(&lowered_records);
        let decision_log_digest_basis = build_decision_log_digest_basis(&decision_log);

        Ok(LoweredMergePlan {
            request: policy_plan.request,
            basis: policy_plan.basis,
            ancestry: policy_plan.ancestry,
            target_delta: policy_plan.target_delta,
            source_delta: policy_plan.source_delta,
            effective_identity_declarations: policy_plan.effective_identity_declarations,
            source_records: policy_plan.source_records,
            candidates: policy_plan.candidates,
            validated_schema_correspondences: policy_plan.validated_schema_correspondences,
            identity_summary: policy_plan.identity_summary,
            classifications: policy_plan.classifications,
            conflict_summary: policy_plan.conflict_summary,
            causal_annotations: policy_plan.causal_annotations,
            causal_summary: policy_plan.causal_summary,
            policy_records: policy_plan.policy_records,
            policy_summary: policy_plan.policy_summary,
            lowered_records: Arc::from(lowered_records),
            lowered_summary,
            decision_log,
            decision_log_digest_basis,
        })
    }
}

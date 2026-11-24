use std::sync::Arc;

use crate::merge::data::{
    LoweredMergePlan, LoweredMergePlanRecord, LoweredRecordDecision, LoweredRecordDecisionKind,
    LoweredRecordDenialKind, LoweredRecordExecutionIntentKind, MergeLoweredAspectDigestRow,
    MergeLoweredPlanDigestBasis,
};

pub(super) fn merge_lowered_plan_digest_basis(
    plan: &LoweredMergePlan,
) -> MergeLoweredPlanDigestBasis {
    MergeLoweredPlanDigestBasis {
        records: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.record.clone())
                .collect::<Vec<_>>(),
        ),
        readiness: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.readiness)
                .collect::<Vec<_>>(),
        ),
        resolution_classes: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.resolution_class)
                .collect::<Vec<_>>(),
        ),
        executable_classes: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.executable_class)
                .collect::<Vec<_>>(),
        ),
        record_decisions: Arc::from(
            plan.lowered_records
                .iter()
                .map(record_decision_kind)
                .collect::<Vec<_>>(),
        ),
        lowered_actions: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.lowered_action)
                .collect::<Vec<_>>(),
        ),
        blocked_reasons: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.blocked_reason)
                .collect::<Vec<_>>(),
        ),
        rejected_reasons: Arc::from(
            plan.lowered_records
                .iter()
                .map(|record| record.rejected_reason)
                .collect::<Vec<_>>(),
        ),
        execution_bundle_kinds: Arc::from(
            plan.lowered_records
                .iter()
                .map(execution_bundle_kind)
                .collect::<Vec<_>>(),
        ),
        denial_bundle_kinds: Arc::from(
            plan.lowered_records
                .iter()
                .map(denial_bundle_kind)
                .collect::<Vec<_>>(),
        ),
        aspect_rows: Arc::from(
            plan.lowered_records
                .iter()
                .map(lowered_aspect_rows_for_digest)
                .collect::<Vec<_>>(),
        ),
    }
}

fn record_decision_kind(record: &LoweredMergePlanRecord) -> LoweredRecordDecisionKind {
    match record.record_decision {
        LoweredRecordDecision::Execute(_) => LoweredRecordDecisionKind::Execute,
        LoweredRecordDecision::Block(_) => LoweredRecordDecisionKind::Block,
        LoweredRecordDecision::Reject(_) => LoweredRecordDecisionKind::Reject,
    }
}

fn execution_bundle_kind(
    record: &LoweredMergePlanRecord,
) -> Option<LoweredRecordExecutionIntentKind> {
    match &record.record_decision {
        LoweredRecordDecision::Execute(bundle) => Some(bundle.kind),
        LoweredRecordDecision::Block(_) | LoweredRecordDecision::Reject(_) => None,
    }
}

fn denial_bundle_kind(record: &LoweredMergePlanRecord) -> Option<LoweredRecordDenialKind> {
    match &record.record_decision {
        LoweredRecordDecision::Block(bundle) | LoweredRecordDecision::Reject(bundle) => {
            Some(bundle.kind)
        }
        LoweredRecordDecision::Execute(_) => None,
    }
}

fn lowered_aspect_rows_for_digest(
    record: &LoweredMergePlanRecord,
) -> Arc<[MergeLoweredAspectDigestRow]> {
    Arc::from(
        record
            .aspect_outcomes
            .iter()
            .map(|aspect| MergeLoweredAspectDigestRow {
                aspect_key: aspect.aspect_key.clone(),
                readiness: aspect.readiness,
                lowered_action: aspect.lowered_action,
                authorized_values: aspect.authorized_values,
                execution_intent: aspect.execution_intent,
                resolved_value_strategy: aspect.resolved_value_strategy.clone(),
                denial_intent: aspect.denial_intent,
                blocked_reason: aspect.blocked_reason,
                rejected_reason: aspect.rejected_reason,
            })
            .collect::<Vec<_>>(),
    )
}

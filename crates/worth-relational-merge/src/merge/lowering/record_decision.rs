use std::sync::Arc;

use crate::merge::data::{
    LoweredMergeAction, LoweredMergeBlockedReason, LoweredMergeRejectedReason,
    LoweredRecordDecision, LoweredRecordDenialAspectIntent, LoweredRecordDenialBundle,
    LoweredRecordDenialKind, LoweredRecordExecutionAspectIntent, LoweredRecordExecutionBundle,
    LoweredRecordExecutionIntentKind, MergeExecutionReadiness, MergePlanningError,
    MergeResolutionClass,
};

use super::denial_classification::{
    blocked_denial_kind_for_record, blocked_denial_kind_from_reason,
};

pub(super) fn record_decision_for_record(
    readiness: MergeExecutionReadiness,
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    lowered_action: Option<LoweredMergeAction>,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    rejected_reason: Option<LoweredMergeRejectedReason>,
    execution_bundle: Option<LoweredRecordExecutionBundle>,
    denial_bundle: Option<LoweredRecordDenialBundle>,
) -> Result<LoweredRecordDecision, MergePlanningError> {
    match readiness {
        MergeExecutionReadiness::Admitted => {
            if let Some(bundle) = execution_bundle {
                Ok(LoweredRecordDecision::Execute(bundle))
            } else {
                synthesized_execution_bundle(classification, lowered_action)
                    .map(LoweredRecordDecision::Execute)
                    .ok_or(MergePlanningError::MissingLoweredRecordExecutionBundle {
                        classification,
                        readiness,
                        lowered_action,
                    })
            }
        }
        MergeExecutionReadiness::Blocked => {
            if let Some(bundle) = denial_bundle {
                Ok(LoweredRecordDecision::Block(bundle))
            } else {
                synthesized_denial_bundle(
                    classification,
                    resolution_class,
                    blocked_reason,
                    readiness,
                )
                .map(LoweredRecordDecision::Block)
                .ok_or(MergePlanningError::MissingLoweredRecordDenialBundle)
            }
        }
        MergeExecutionReadiness::Rejected => {
            if let Some(bundle) = denial_bundle {
                Ok(LoweredRecordDecision::Reject(bundle))
            } else {
                let _ = rejected_reason;
                synthesized_denial_bundle(classification, resolution_class, None, readiness)
                    .map(LoweredRecordDecision::Reject)
                    .ok_or(MergePlanningError::MissingLoweredRecordDenialBundle)
            }
        }
    }
}

fn synthesized_execution_bundle(
    classification: crate::merge::data::MergeConflictClass,
    lowered_action: Option<LoweredMergeAction>,
) -> Option<LoweredRecordExecutionBundle> {
    let kind = match (classification, lowered_action) {
        (
            crate::merge::data::MergeConflictClass::SourceOnlyAddition,
            Some(LoweredMergeAction::KeepSourceAddition),
        ) => LoweredRecordExecutionIntentKind::AdoptSourceRecord,
        (
            crate::merge::data::MergeConflictClass::ExactSharedTruth,
            Some(LoweredMergeAction::KeepExactSharedTruth),
        ) => LoweredRecordExecutionIntentKind::PreserveSharedRecord,
        (
            crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence,
            Some(LoweredMergeAction::ReconcileSchemaCorrespondence),
        )
        | (
            crate::merge::data::MergeConflictClass::DivergentVisibleState,
            Some(LoweredMergeAction::ReconcileDivergentVisibleState),
        ) => LoweredRecordExecutionIntentKind::ReconcileRecord,
        (
            crate::merge::data::MergeConflictClass::Deletion(
                crate::merge::data::DeletionMergeClass::DeletedOnBothSides,
            ),
            Some(LoweredMergeAction::ConvergeDeletedOnBothSides),
        ) => LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides,
        _ => return None,
    };
    Some(LoweredRecordExecutionBundle {
        kind,
        aspects: Arc::from(Vec::<LoweredRecordExecutionAspectIntent>::new()),
    })
}

fn synthesized_denial_bundle(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredRecordDenialBundle> {
    let kind = match readiness {
        MergeExecutionReadiness::Admitted => return None,
        MergeExecutionReadiness::Blocked => blocked_reason
            .map(blocked_denial_kind_from_reason)
            .unwrap_or_else(|| {
                blocked_denial_kind_for_record(classification, resolution_class, &[])
            }),
        MergeExecutionReadiness::Rejected => LoweredRecordDenialKind::RejectedPolicy,
    };
    Some(LoweredRecordDenialBundle {
        kind,
        aspects: Arc::from(Vec::<LoweredRecordDenialAspectIntent>::new()),
    })
}

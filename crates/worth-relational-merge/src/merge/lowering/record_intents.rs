use std::sync::Arc;

use crate::merge::data::{
    AspectComparisonState, LoweredAspectAction, LoweredAspectOutcome, LoweredMergeAction,
    LoweredMergeRejectedReason, LoweredRecordDenialAspectIntent, LoweredRecordDenialBundle,
    LoweredRecordExecutionAspectIntent, LoweredRecordExecutionBundle,
    LoweredRecordExecutionIntentKind, MergeExecutionReadiness, MergeResolutionClass,
};

use super::denial_classification::{
    blocked_denial_kind_for_record, rejected_denial_kind_for_record,
};

pub(super) fn lowered_action_for_record(
    classification: crate::merge::data::MergeConflictClass,
    aspect_outcomes: &[LoweredAspectOutcome],
    readiness: MergeExecutionReadiness,
) -> Option<LoweredMergeAction> {
    if readiness != MergeExecutionReadiness::Admitted
        || (!aspect_outcomes.is_empty()
            && aspect_outcomes
                .iter()
                .any(|aspect| aspect.lowered_action.is_none()))
    {
        return None;
    }

    match classification {
        crate::merge::data::MergeConflictClass::SourceOnlyAddition => {
            Some(LoweredMergeAction::KeepSourceAddition)
        }
        crate::merge::data::MergeConflictClass::ExactSharedTruth => {
            Some(LoweredMergeAction::KeepExactSharedTruth)
        }
        crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence => {
            Some(LoweredMergeAction::ReconcileSchemaCorrespondence)
        }
        crate::merge::data::MergeConflictClass::DivergentVisibleState
        | crate::merge::data::MergeConflictClass::StrategyIntentConflict => {
            Some(LoweredMergeAction::ReconcileDivergentVisibleState)
        }
        crate::merge::data::MergeConflictClass::Deletion(
            crate::merge::data::DeletionMergeClass::DeletedOnBothSides,
        ) => Some(LoweredMergeAction::ConvergeDeletedOnBothSides),
        crate::merge::data::MergeConflictClass::Deletion(_)
        | crate::merge::data::MergeConflictClass::RelationEndpointDivergence => None,
    }
}

pub(super) fn execution_bundle_for_record(
    classification: crate::merge::data::MergeConflictClass,
    aspect_outcomes: &[LoweredAspectOutcome],
    readiness: MergeExecutionReadiness,
) -> Option<LoweredRecordExecutionBundle> {
    if readiness != MergeExecutionReadiness::Admitted {
        return None;
    }
    let aspect_intents = aspect_outcomes
        .iter()
        .filter_map(|outcome| {
            Some(LoweredRecordExecutionAspectIntent {
                aspect_key: outcome.aspect_key.clone(),
                intent: outcome.execution_intent?,
            })
        })
        .collect::<Vec<_>>();
    let kind = match classification {
        crate::merge::data::MergeConflictClass::SourceOnlyAddition => {
            LoweredRecordExecutionIntentKind::AdoptSourceRecord
        }
        crate::merge::data::MergeConflictClass::ExactSharedTruth => {
            LoweredRecordExecutionIntentKind::PreserveSharedRecord
        }
        crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence => {
            LoweredRecordExecutionIntentKind::ReconcileRecord
        }
        crate::merge::data::MergeConflictClass::DivergentVisibleState
        | crate::merge::data::MergeConflictClass::StrategyIntentConflict => {
            LoweredRecordExecutionIntentKind::ReconcileRecord
        }
        crate::merge::data::MergeConflictClass::Deletion(
            crate::merge::data::DeletionMergeClass::DeletedOnBothSides,
        ) if aspect_intents.is_empty() => {
            LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides
        }
        crate::merge::data::MergeConflictClass::Deletion(_)
        | crate::merge::data::MergeConflictClass::RelationEndpointDivergence => {
            return None;
        }
    };
    Some(LoweredRecordExecutionBundle {
        kind,
        aspects: Arc::from(aspect_intents),
    })
}

pub(super) fn rejected_reason_for_record(
    aspect_outcomes: &[LoweredAspectOutcome],
    readiness: MergeExecutionReadiness,
) -> Option<LoweredMergeRejectedReason> {
    if readiness != MergeExecutionReadiness::Rejected {
        return None;
    }

    let mut reject_reason: Option<LoweredMergeRejectedReason> = None;
    for aspect in aspect_outcomes
        .iter()
        .filter_map(|aspect| aspect.rejected_reason)
    {
        reject_reason = Some(match reject_reason {
            None => aspect,
            Some(existing) if existing == aspect => existing,
            Some(_) => LoweredMergeRejectedReason::MixedPolicyRejectClasses,
        });
    }
    reject_reason
}

pub(super) fn denial_bundle_for_record(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    aspect_outcomes: &[LoweredAspectOutcome],
    readiness: MergeExecutionReadiness,
) -> Option<LoweredRecordDenialBundle> {
    match readiness {
        MergeExecutionReadiness::Admitted => None,
        MergeExecutionReadiness::Blocked => {
            let aspects = denial_aspect_intents(aspect_outcomes);
            Some(LoweredRecordDenialBundle {
                kind: blocked_denial_kind_for_record(
                    classification,
                    resolution_class,
                    aspects.as_slice(),
                ),
                aspects: Arc::from(aspects),
            })
        }
        MergeExecutionReadiness::Rejected => {
            let aspects = denial_aspect_intents(aspect_outcomes);
            Some(LoweredRecordDenialBundle {
                kind: rejected_denial_kind_for_record(aspects.as_slice()),
                aspects: Arc::from(aspects),
            })
        }
    }
}

fn denial_aspect_intents(
    aspect_outcomes: &[LoweredAspectOutcome],
) -> Vec<LoweredRecordDenialAspectIntent> {
    aspect_outcomes
        .iter()
        .filter_map(|outcome| {
            Some(LoweredRecordDenialAspectIntent {
                aspect_key: outcome.aspect_key.clone(),
                intent: outcome.denial_intent?,
            })
        })
        .collect()
}

pub(super) fn lowered_aspect_action_for_resolution(
    classification: crate::merge::data::MergeConflictClass,
    comparison: AspectComparisonState,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredAspectAction> {
    if readiness != MergeExecutionReadiness::Admitted {
        return None;
    }
    match (classification, comparison) {
        (_, AspectComparisonState::Unavailable) => None,
        (
            crate::merge::data::MergeConflictClass::SourceOnlyAddition,
            AspectComparisonState::SourceOnly,
        ) => Some(LoweredAspectAction::AdoptSourceAspect),
        (
            crate::merge::data::MergeConflictClass::ExactSharedTruth,
            AspectComparisonState::Equal,
        ) => Some(LoweredAspectAction::KeepSharedAspect),
        (
            crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence
            | crate::merge::data::MergeConflictClass::DivergentVisibleState,
            AspectComparisonState::Equal
            | AspectComparisonState::SourceOnly
            | AspectComparisonState::TargetOnly
            | AspectComparisonState::Divergent,
        ) => Some(LoweredAspectAction::ReconcileCorrespondedAspect),
        _ => None,
    }
}

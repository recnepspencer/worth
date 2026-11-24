use std::collections::BTreeMap;

use crate::merge::data::{
    AspectComparisonState, AuthorizedAspectValueSurface, AuthorizedAspectValueUsage,
    LoweredAspectDenialIntent, LoweredAspectExecutionIntent, LoweredAspectOutcome,
    LoweredMergeRejectedReason, MergeExecutionReadiness, MergePlanningError,
    MergePolicyDecisionBoundary, MergeResolutionClass,
};
use worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;

use super::record_intents::lowered_aspect_action_for_resolution;
use super::resolution::blocked_reason_for_aspect;

pub(super) fn lowered_aspect_outcomes_for_record(
    aspect_plans: &AspectContractPlanCatalog,
    source_record: &crate::merge::data::VisibleMergeRecord,
    policy_record: &crate::merge::data::MergePolicyResolutionRecord,
    resolution_class: MergeResolutionClass,
) -> Result<Vec<LoweredAspectOutcome>, MergePlanningError> {
    let Some(plan) =
        crate::merge::aspect_plan_lookup::lowered_plan_for_record(aspect_plans, source_record)
    else {
        return Ok(Vec::new());
    };
    let policy_by_aspect = policy_record
        .aspect_resolutions
        .iter()
        .map(|aspect| (aspect.aspect_key.clone(), aspect))
        .collect::<BTreeMap<_, _>>();

    Ok(plan
        .executable_bindings
        .iter()
        .map(|binding| {
            let aspect_resolution = policy_by_aspect.get(binding.aspect_key()).copied();
            let readiness = aspect_resolution
                .map(|aspect| readiness_for_policy_decision(aspect.decision_boundary))
                .unwrap_or(MergeExecutionReadiness::Blocked);
            let applied_policy = aspect_resolution.and_then(|aspect| aspect.applied_policy.clone());
            LoweredAspectOutcome {
                aspect_key: binding.aspect_key().clone(),
                applied_policy,
                readiness,
                lowered_action: aspect_resolution.and_then(|aspect| {
                    lowered_aspect_action_for_resolution(
                        policy_record.classification,
                        aspect.comparison,
                        readiness,
                    )
                }),
                authorized_values: aspect_resolution.and_then(|aspect| {
                    authorized_values_for_aspect(
                        policy_record.classification,
                        aspect.comparison,
                        readiness,
                    )
                }),
                execution_intent: aspect_resolution.and_then(|aspect| {
                    lowered_aspect_execution_intent(
                        policy_record.classification,
                        aspect.comparison,
                        readiness,
                    )
                }),
                resolved_value_strategy: aspect_resolution
                    .and_then(|aspect| aspect.resolved_value_strategy.clone()),
                denial_intent: aspect_resolution.and_then(|aspect| {
                    lowered_aspect_denial_intent(
                        policy_record.classification,
                        resolution_class,
                        aspect.comparison,
                        aspect.decision_boundary,
                        readiness,
                    )
                }),
                blocked_reason: aspect_resolution.and_then(|aspect| {
                    blocked_reason_for_aspect(
                        policy_record.classification,
                        resolution_class,
                        aspect.comparison,
                        aspect.decision_boundary,
                        readiness,
                    )
                }),
                rejected_reason: aspect_resolution.and_then(|aspect| {
                    rejected_reason_for_aspect(aspect.decision_boundary, readiness)
                }),
            }
        })
        .collect())
}

pub(super) fn readiness_for_policy_decision(
    decision_boundary: MergePolicyDecisionBoundary,
) -> MergeExecutionReadiness {
    match decision_boundary {
        MergePolicyDecisionBoundary::AutoResolved => MergeExecutionReadiness::Admitted,
        MergePolicyDecisionBoundary::RequiresManualResolution { .. } => {
            MergeExecutionReadiness::Blocked
        }
        MergePolicyDecisionBoundary::Reject { .. } => MergeExecutionReadiness::Rejected,
    }
}

fn lowered_aspect_denial_intent(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    comparison: AspectComparisonState,
    decision_boundary: MergePolicyDecisionBoundary,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredAspectDenialIntent> {
    match readiness {
        MergeExecutionReadiness::Admitted => None,
        MergeExecutionReadiness::Blocked => match blocked_reason_for_aspect(
            classification,
            resolution_class,
            comparison,
            decision_boundary,
            readiness,
        )? {
            crate::merge::data::LoweredMergeBlockedReason::MissingVisibleState => {
                Some(LoweredAspectDenialIntent::BlockedMissingVisibleState)
            }
            crate::merge::data::LoweredMergeBlockedReason::MissingAncestorValueBasis => {
                Some(LoweredAspectDenialIntent::BlockedMissingAncestorValueBasis)
            }
            crate::merge::data::LoweredMergeBlockedReason::UnvalidatedSchemaCorrespondence => {
                Some(LoweredAspectDenialIntent::BlockedUnvalidatedSchemaCorrespondence)
            }
            crate::merge::data::LoweredMergeBlockedReason::SourceDeletedTargetLive => {
                Some(LoweredAspectDenialIntent::BlockedSourceDeletedTargetLive)
            }
            crate::merge::data::LoweredMergeBlockedReason::SourceLiveTargetDeleted => {
                Some(LoweredAspectDenialIntent::BlockedSourceLiveTargetDeleted)
            }
            crate::merge::data::LoweredMergeBlockedReason::DeletedOnBothSides => {
                Some(LoweredAspectDenialIntent::BlockedDeletedOnBothSides)
            }
            crate::merge::data::LoweredMergeBlockedReason::DeletedVsModified => {
                Some(LoweredAspectDenialIntent::BlockedDeletedVsModified)
            }
            crate::merge::data::LoweredMergeBlockedReason::DeletedVsRewired => {
                Some(LoweredAspectDenialIntent::BlockedDeletedVsRewired)
            }
            crate::merge::data::LoweredMergeBlockedReason::RelationEndpointRewiredLocal => {
                Some(LoweredAspectDenialIntent::BlockedRelationEndpointRewiredLocal)
            }
            crate::merge::data::LoweredMergeBlockedReason::RelationEndpointRewiredEscalated => {
                Some(LoweredAspectDenialIntent::BlockedRelationEndpointRewiredEscalated)
            }
            crate::merge::data::LoweredMergeBlockedReason::TopologyRegionConflict => {
                Some(LoweredAspectDenialIntent::BlockedTopologyRegionConflict)
            }
            crate::merge::data::LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution => {
                Some(LoweredAspectDenialIntent::BlockedStrategyIntentConflict)
            }
            crate::merge::data::LoweredMergeBlockedReason::ManualConflictResolutionRequired => {
                Some(LoweredAspectDenialIntent::BlockedManualResolution)
            }
        },
        MergeExecutionReadiness::Rejected => {
            rejected_reason_for_aspect(decision_boundary, readiness)?;
            Some(match decision_boundary {
                MergePolicyDecisionBoundary::Reject {
                    class: crate::merge::data::MergePolicyRejectClass::BuiltInFailOnConflict,
                } => LoweredAspectDenialIntent::RejectedPolicy,
                MergePolicyDecisionBoundary::Reject {
                    class:
                        crate::merge::data::MergePolicyRejectClass::LastWriterWinsCausalConflict
                        | crate::merge::data::MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                } => LoweredAspectDenialIntent::RejectedPolicy,
                MergePolicyDecisionBoundary::Reject {
                    class: crate::merge::data::MergePolicyRejectClass::CustomPolicyRejected,
                } => LoweredAspectDenialIntent::RejectedCustomPolicy,
                MergePolicyDecisionBoundary::Reject {
                    class: crate::merge::data::MergePolicyRejectClass::MixedAspectRejectClasses,
                } => LoweredAspectDenialIntent::RejectedMixedPolicyClasses,
                _ => LoweredAspectDenialIntent::RejectedPolicy,
            })
        }
    }
}

fn authorized_values_for_aspect(
    classification: crate::merge::data::MergeConflictClass,
    comparison: AspectComparisonState,
    readiness: MergeExecutionReadiness,
) -> Option<AuthorizedAspectValueSurface> {
    if readiness != MergeExecutionReadiness::Admitted {
        return None;
    }
    match (classification, comparison) {
        (
            crate::merge::data::MergeConflictClass::SourceOnlyAddition,
            AspectComparisonState::SourceOnly,
        ) => Some(AuthorizedAspectValueSurface {
            source: AuthorizedAspectValueUsage::ConsumeVisibleValue,
            target: AuthorizedAspectValueUsage::NotAuthorized,
            base: AuthorizedAspectValueUsage::NotAuthorized,
        }),
        (
            crate::merge::data::MergeConflictClass::ExactSharedTruth,
            AspectComparisonState::Equal,
        ) => Some(AuthorizedAspectValueSurface {
            source: AuthorizedAspectValueUsage::EqualityWitnessOnly,
            target: AuthorizedAspectValueUsage::EqualityWitnessOnly,
            base: AuthorizedAspectValueUsage::NotAuthorized,
        }),
        (
            crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence
            | crate::merge::data::MergeConflictClass::DivergentVisibleState,
            AspectComparisonState::Equal
            | AspectComparisonState::SourceOnly
            | AspectComparisonState::TargetOnly
            | AspectComparisonState::Divergent,
        ) => Some(AuthorizedAspectValueSurface {
            source: AuthorizedAspectValueUsage::ConsumeVisibleValue,
            target: AuthorizedAspectValueUsage::ConsumeVisibleValue,
            base: AuthorizedAspectValueUsage::ConsumeBaseValue,
        }),
        _ => None,
    }
}

fn lowered_aspect_execution_intent(
    classification: crate::merge::data::MergeConflictClass,
    comparison: AspectComparisonState,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredAspectExecutionIntent> {
    let authorized_values = authorized_values_for_aspect(classification, comparison, readiness)?;
    match (classification, comparison) {
        (
            crate::merge::data::MergeConflictClass::SourceOnlyAddition,
            AspectComparisonState::SourceOnly,
        ) => Some(LoweredAspectExecutionIntent::AdoptSourceValue { authorized_values }),
        (
            crate::merge::data::MergeConflictClass::ExactSharedTruth,
            AspectComparisonState::Equal,
        ) => Some(LoweredAspectExecutionIntent::PreserveSharedValue { authorized_values }),
        (
            crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence
            | crate::merge::data::MergeConflictClass::DivergentVisibleState,
            AspectComparisonState::Equal
            | AspectComparisonState::SourceOnly
            | AspectComparisonState::TargetOnly
            | AspectComparisonState::Divergent,
        ) => Some(LoweredAspectExecutionIntent::ReconcileVisibleValues { authorized_values }),
        _ => None,
    }
}

fn rejected_reason_for_aspect(
    decision_boundary: MergePolicyDecisionBoundary,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredMergeRejectedReason> {
    (readiness == MergeExecutionReadiness::Rejected
        && matches!(
            decision_boundary,
            MergePolicyDecisionBoundary::Reject { .. }
        ))
    .then_some(match decision_boundary {
        MergePolicyDecisionBoundary::Reject {
            class: crate::merge::data::MergePolicyRejectClass::BuiltInFailOnConflict,
        } => LoweredMergeRejectedReason::FailOnConflictPolicy,
        MergePolicyDecisionBoundary::Reject {
            class:
                crate::merge::data::MergePolicyRejectClass::LastWriterWinsCausalConflict
                | crate::merge::data::MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
        } => LoweredMergeRejectedReason::FailOnConflictPolicy,
        MergePolicyDecisionBoundary::Reject {
            class: crate::merge::data::MergePolicyRejectClass::CustomPolicyRejected,
        } => LoweredMergeRejectedReason::CustomPolicyRejected,
        MergePolicyDecisionBoundary::Reject {
            class: crate::merge::data::MergePolicyRejectClass::MixedAspectRejectClasses,
        } => LoweredMergeRejectedReason::MixedPolicyRejectClasses,
        _ => LoweredMergeRejectedReason::FailOnConflictPolicy,
    })
}

#[cfg(test)]
pub mod tests {
    use super::rejected_reason_for_aspect;
    use crate::merge::data::{
        LoweredMergeRejectedReason, MergeExecutionReadiness, MergePolicyDecisionBoundary,
        MergePolicyRejectClass,
    };

    #[test]
    fn rejected_reason_for_aspect_preserves_specific_reject_class() {
        assert_eq!(
            rejected_reason_for_aspect(
                MergePolicyDecisionBoundary::Reject {
                    class: MergePolicyRejectClass::BuiltInFailOnConflict,
                },
                MergeExecutionReadiness::Rejected,
            ),
            Some(LoweredMergeRejectedReason::FailOnConflictPolicy)
        );
        assert_eq!(
            rejected_reason_for_aspect(
                MergePolicyDecisionBoundary::Reject {
                    class: MergePolicyRejectClass::CustomPolicyRejected,
                },
                MergeExecutionReadiness::Rejected,
            ),
            Some(LoweredMergeRejectedReason::CustomPolicyRejected)
        );
    }
}

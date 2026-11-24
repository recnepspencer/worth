//! Descriptive merge-policy classifications shared by planning and witness comparison.
use super::conflicts::{DeletionMergeClass, MergeConflictClass};
use super::policy::{
    AspectPolicyResolutionRecord, MergeManualResolutionClass, MergePolicyDecisionBoundary,
    MergePolicyOwnershipClass, MergePolicyOwnershipSurface, MergePolicyRejectClass,
    ResolvedAspectMergePolicy,
};

pub(crate) fn ownership_surface_for_policies(
    applied_policies: &[ResolvedAspectMergePolicy],
) -> MergePolicyOwnershipSurface {
    if applied_policies
        .iter()
        .any(|policy| policy.policy.ownership_class() == MergePolicyOwnershipClass::CustomPolicy)
    {
        MergePolicyOwnershipSurface::ContainsCustomPolicy
    } else {
        MergePolicyOwnershipSurface::RuntimeOnly
    }
}

pub(crate) fn aggregate_record_resolution(
    classification: MergeConflictClass,
    aspects: &[AspectPolicyResolutionRecord],
) -> MergePolicyDecisionBoundary {
    if aspects.is_empty() {
        return match classification {
            MergeConflictClass::SourceOnlyAddition
            | MergeConflictClass::ExactSharedTruth
            | MergeConflictClass::Deletion(DeletionMergeClass::DeletedOnBothSides) => {
                MergePolicyDecisionBoundary::AutoResolved
            }
            MergeConflictClass::SchemaDeclaredCorrespondence
            | MergeConflictClass::Deletion(DeletionMergeClass::SourceDeletedTargetLive)
            | MergeConflictClass::Deletion(DeletionMergeClass::SourceLiveTargetDeleted)
            | MergeConflictClass::Deletion(DeletionMergeClass::DeletedVsModified)
            | MergeConflictClass::Deletion(DeletionMergeClass::DeletedVsRewired)
            | MergeConflictClass::DivergentVisibleState
            | MergeConflictClass::RelationEndpointDivergence => {
                MergePolicyDecisionBoundary::RequiresManualResolution {
                    class: MergeManualResolutionClass::GenericRuntimeConflict,
                }
            }
            MergeConflictClass::StrategyIntentConflict => {
                MergePolicyDecisionBoundary::RequiresManualResolution {
                    class: MergeManualResolutionClass::StrategyIntentConflict,
                }
            }
        };
    }
    let mut reject_class: Option<MergePolicyRejectClass> = None;
    let mut manual_class: Option<MergeManualResolutionClass> = None;

    for aspect in aspects {
        match aspect.decision_boundary {
            MergePolicyDecisionBoundary::AutoResolved => {}
            MergePolicyDecisionBoundary::RequiresManualResolution { class } => {
                manual_class = Some(match manual_class {
                    None => class,
                    Some(existing) if existing == class => existing,
                    Some(_) => MergeManualResolutionClass::MixedAspectManualResolution,
                });
            }
            MergePolicyDecisionBoundary::Reject { class } => {
                reject_class = Some(match reject_class {
                    None => class,
                    Some(existing) if existing == class => existing,
                    Some(_) => MergePolicyRejectClass::MixedAspectRejectClasses,
                });
            }
        }
    }

    if let Some(class) = reject_class {
        MergePolicyDecisionBoundary::Reject { class }
    } else if deletion_conflict_requires_manual_resolution(classification) {
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::GenericRuntimeConflict,
        }
    } else if classification == MergeConflictClass::StrategyIntentConflict {
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::StrategyIntentConflict,
        }
    } else if let Some(class) = manual_class {
        MergePolicyDecisionBoundary::RequiresManualResolution { class }
    } else {
        MergePolicyDecisionBoundary::AutoResolved
    }
}

pub(crate) fn deletion_conflict_requires_manual_resolution(
    classification: MergeConflictClass,
) -> bool {
    matches!(
        classification,
        MergeConflictClass::Deletion(
            DeletionMergeClass::SourceDeletedTargetLive
                | DeletionMergeClass::SourceLiveTargetDeleted
                | DeletionMergeClass::DeletedVsModified
                | DeletionMergeClass::DeletedVsRewired
        )
    )
}

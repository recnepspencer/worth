pub(crate) use worth_relational_contracts::internal_port::merge::data::blocked_posture::{
    blocked_reason_for_deletion_class, blocked_reason_for_topology_resolution_class,
};

use crate::merge::data::{
    AspectComparisonState, DeletionExecutionClass, LoweredAspectOutcome, LoweredMergeBlockedReason,
    LoweredRecordExecutionIntentKind, MergeExecutableClass, MergeExecutionReadiness,
    MergePolicyDecisionBoundary, MergeResolutionClass, TopologyExecutionClass,
    TopologyRewireAdmissionPolicy,
};

pub(super) fn resolution_class_for_record(
    classification: crate::merge::data::MergeConflictClass,
    relation_evidence: Option<&crate::merge::data::RelationConflictEvidence>,
) -> MergeResolutionClass {
    match classification {
        crate::merge::data::MergeConflictClass::SourceOnlyAddition => {
            MergeResolutionClass::SourceOnlyAddition
        }
        crate::merge::data::MergeConflictClass::ExactSharedTruth => {
            MergeResolutionClass::ExactSharedTruth
        }
        crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence => {
            MergeResolutionClass::SchemaDeclaredCorrespondence
        }
        crate::merge::data::MergeConflictClass::Deletion(class) => {
            MergeResolutionClass::Deletion(match class {
                crate::merge::data::DeletionMergeClass::SourceDeletedTargetLive => {
                    DeletionExecutionClass::SourceDeletedTargetLive
                }
                crate::merge::data::DeletionMergeClass::SourceLiveTargetDeleted => {
                    DeletionExecutionClass::SourceLiveTargetDeleted
                }
                crate::merge::data::DeletionMergeClass::DeletedOnBothSides => {
                    DeletionExecutionClass::DeletedOnBothSides
                }
                crate::merge::data::DeletionMergeClass::DeletedVsModified => {
                    DeletionExecutionClass::DeletedVsModified
                }
                crate::merge::data::DeletionMergeClass::DeletedVsRewired => {
                    DeletionExecutionClass::DeletedVsRewired
                }
            })
        }
        crate::merge::data::MergeConflictClass::RelationEndpointDivergence => {
            MergeResolutionClass::Topology(topology_resolution_class_for_record(relation_evidence))
        }
        crate::merge::data::MergeConflictClass::DivergentVisibleState
        | crate::merge::data::MergeConflictClass::StrategyIntentConflict => {
            MergeResolutionClass::DivergentVisibleState
        }
    }
}

fn topology_resolution_class_for_record(
    relation_evidence: Option<&crate::merge::data::RelationConflictEvidence>,
) -> TopologyExecutionClass {
    let admission_policy = crate::merge::policy::current_topology_rewire_admission_policy();
    let Some(evidence) = relation_evidence else {
        return TopologyExecutionClass::TopologyRegionConflict;
    };

    match evidence.propagation {
        crate::merge::data::RelationConflictPropagation::RelationLocalOnly => {
            match evidence.endpoint_continuity {
                crate::merge::data::EndpointContinuityClass::EndpointsStable => {
                    TopologyExecutionClass::RelationEndpointStable
                }
                crate::merge::data::EndpointContinuityClass::SourceEndpointRewired
                | crate::merge::data::EndpointContinuityClass::TargetEndpointRewired
                | crate::merge::data::EndpointContinuityClass::BothEndpointsRewired => {
                    TopologyExecutionClass::RelationEndpointRewiredLocal
                }
            }
        }
        crate::merge::data::RelationConflictPropagation::RelationLocalRewireCandidate => {
            match admission_policy {
                TopologyRewireAdmissionPolicy::AlwaysEscalateToTopologyRegion => {
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                }
            }
        }
        crate::merge::data::RelationConflictPropagation::EscalatesToTopologyRegionConflict => {
            match admission_policy {
                TopologyRewireAdmissionPolicy::AlwaysEscalateToTopologyRegion => {
                    TopologyExecutionClass::TopologyRegionConflict
                }
            }
        }
    }
}

pub(super) fn executable_class_for_record(
    resolution_class: MergeResolutionClass,
    readiness: MergeExecutionReadiness,
    execution_bundle_kind: Option<LoweredRecordExecutionIntentKind>,
) -> Option<MergeExecutableClass> {
    if readiness != MergeExecutionReadiness::Admitted {
        return None;
    }
    match (resolution_class, execution_bundle_kind) {
        (
            MergeResolutionClass::SourceOnlyAddition,
            Some(LoweredRecordExecutionIntentKind::AdoptSourceRecord),
        ) => Some(MergeExecutableClass::AdoptSourceRecord),
        (
            MergeResolutionClass::ExactSharedTruth,
            Some(LoweredRecordExecutionIntentKind::PreserveSharedRecord),
        ) => Some(MergeExecutableClass::PreserveSharedRecord),
        (
            MergeResolutionClass::SchemaDeclaredCorrespondence,
            Some(LoweredRecordExecutionIntentKind::ReconcileRecord),
        )
        | (
            MergeResolutionClass::DivergentVisibleState,
            Some(LoweredRecordExecutionIntentKind::ReconcileRecord),
        ) => Some(MergeExecutableClass::ReconcileRecord),
        (
            MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides),
            Some(LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides),
        ) => Some(MergeExecutableClass::ConvergeDeletedOnBothSides),
        _ => None,
    }
}

pub(super) fn blocked_reason_for_record(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    aspect_outcomes: &[LoweredAspectOutcome],
    readiness: MergeExecutionReadiness,
) -> Option<LoweredMergeBlockedReason> {
    if readiness != MergeExecutionReadiness::Blocked {
        return None;
    }
    if aspect_outcomes.is_empty() {
        return Some(classification_blocked_reason(
            classification,
            resolution_class,
        ));
    }
    if let Some(reason) = aspect_outcomes.iter().find_map(|aspect| {
        aspect
            .blocked_reason
            .filter(|reason| is_deletion_blocked_reason(*reason))
    }) {
        Some(reason)
    } else if let Some(reason) = aspect_outcomes.iter().find_map(|aspect| {
        aspect.blocked_reason.filter(|reason| {
            matches!(
                reason,
                LoweredMergeBlockedReason::RelationEndpointRewiredLocal
                    | LoweredMergeBlockedReason::RelationEndpointRewiredEscalated
                    | LoweredMergeBlockedReason::TopologyRegionConflict
            )
        })
    }) {
        Some(reason)
    } else if aspect_outcomes
        .iter()
        .any(|aspect| aspect.blocked_reason.is_some())
    {
        Some(LoweredMergeBlockedReason::ManualConflictResolutionRequired)
    } else if classification_requires_record_level_blocked_reason(classification) {
        Some(classification_blocked_reason(
            classification,
            resolution_class,
        ))
    } else if classification == crate::merge::data::MergeConflictClass::StrategyIntentConflict {
        Some(LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution)
    } else {
        None
    }
}

fn classification_requires_record_level_blocked_reason(
    classification: crate::merge::data::MergeConflictClass,
) -> bool {
    matches!(
        classification,
        crate::merge::data::MergeConflictClass::Deletion(_)
            | crate::merge::data::MergeConflictClass::RelationEndpointDivergence
    )
}

fn classification_blocked_reason(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
) -> LoweredMergeBlockedReason {
    match classification {
        crate::merge::data::MergeConflictClass::Deletion(class) => {
            blocked_reason_for_deletion_class(class)
        }
        crate::merge::data::MergeConflictClass::RelationEndpointDivergence => {
            blocked_reason_for_topology_resolution_class(resolution_class)
        }
        crate::merge::data::MergeConflictClass::StrategyIntentConflict => {
            LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution
        }
        crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence
        | crate::merge::data::MergeConflictClass::DivergentVisibleState
        | crate::merge::data::MergeConflictClass::ExactSharedTruth
        | crate::merge::data::MergeConflictClass::SourceOnlyAddition => {
            LoweredMergeBlockedReason::ManualConflictResolutionRequired
        }
    }
}

pub(super) fn blocked_reason_for_aspect(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    comparison: AspectComparisonState,
    decision_boundary: MergePolicyDecisionBoundary,
    readiness: MergeExecutionReadiness,
) -> Option<LoweredMergeBlockedReason> {
    if readiness != MergeExecutionReadiness::Blocked {
        return None;
    }
    if let MergePolicyDecisionBoundary::RequiresManualResolution { class } = decision_boundary {
        match class {
            crate::merge::data::MergeManualResolutionClass::MissingVisibleState => {
                return Some(LoweredMergeBlockedReason::MissingVisibleState);
            }
            crate::merge::data::MergeManualResolutionClass::MissingAncestorValueBasis => {
                return Some(LoweredMergeBlockedReason::MissingAncestorValueBasis);
            }
            crate::merge::data::MergeManualResolutionClass::UnvalidatedSchemaCorrespondence => {
                return Some(LoweredMergeBlockedReason::UnvalidatedSchemaCorrespondence);
            }
            crate::merge::data::MergeManualResolutionClass::StrategyIntentConflict => {
                return Some(
                    LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution,
                );
            }
            crate::merge::data::MergeManualResolutionClass::GenericRuntimeConflict
            | crate::merge::data::MergeManualResolutionClass::MixedAspectManualResolution => {}
        }
    }
    match (classification, comparison) {
        (crate::merge::data::MergeConflictClass::StrategyIntentConflict, _) => {
            Some(LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution)
        }
        (crate::merge::data::MergeConflictClass::Deletion(class), _) => {
            Some(blocked_reason_for_deletion_class(class))
        }
        (
            crate::merge::data::MergeConflictClass::RelationEndpointDivergence,
            AspectComparisonState::Divergent
            | AspectComparisonState::TargetOnly
            | AspectComparisonState::SourceOnly,
        ) => Some(blocked_reason_for_topology_resolution_class(
            resolution_class,
        )),
        (_, AspectComparisonState::Unavailable) => {
            Some(LoweredMergeBlockedReason::ManualConflictResolutionRequired)
        }
        _ => Some(LoweredMergeBlockedReason::ManualConflictResolutionRequired),
    }
}

pub(super) fn is_deletion_blocked_reason(reason: LoweredMergeBlockedReason) -> bool {
    matches!(
        reason,
        LoweredMergeBlockedReason::SourceDeletedTargetLive
            | LoweredMergeBlockedReason::SourceLiveTargetDeleted
            | LoweredMergeBlockedReason::DeletedOnBothSides
            | LoweredMergeBlockedReason::DeletedVsModified
            | LoweredMergeBlockedReason::DeletedVsRewired
    )
}

#[cfg(test)]
pub mod tests {
    use super::{
        blocked_reason_for_aspect, blocked_reason_for_deletion_class, executable_class_for_record,
        is_deletion_blocked_reason,
    };
    use crate::merge::data::{
        AspectComparisonState, DeletionExecutionClass, DeletionMergeClass,
        LoweredMergeBlockedReason, LoweredRecordExecutionIntentKind, MergeExecutableClass,
        MergeExecutionReadiness, MergeManualResolutionClass, MergePolicyDecisionBoundary,
        MergeResolutionClass,
    };

    #[test]
    fn deletion_classes_map_to_distinct_blocked_reasons() {
        assert_eq!(
            blocked_reason_for_deletion_class(DeletionMergeClass::SourceDeletedTargetLive),
            LoweredMergeBlockedReason::SourceDeletedTargetLive
        );
        assert_eq!(
            blocked_reason_for_deletion_class(DeletionMergeClass::SourceLiveTargetDeleted),
            LoweredMergeBlockedReason::SourceLiveTargetDeleted
        );
        assert_eq!(
            blocked_reason_for_deletion_class(DeletionMergeClass::DeletedOnBothSides),
            LoweredMergeBlockedReason::DeletedOnBothSides
        );
        assert_eq!(
            blocked_reason_for_deletion_class(DeletionMergeClass::DeletedVsModified),
            LoweredMergeBlockedReason::DeletedVsModified
        );
        assert_eq!(
            blocked_reason_for_deletion_class(DeletionMergeClass::DeletedVsRewired),
            LoweredMergeBlockedReason::DeletedVsRewired
        );
    }

    #[test]
    fn deletion_reason_detector_is_specific_to_deletion_blocking() {
        assert!(is_deletion_blocked_reason(
            LoweredMergeBlockedReason::DeletedVsModified
        ));
        assert!(!is_deletion_blocked_reason(
            LoweredMergeBlockedReason::ManualConflictResolutionRequired
        ));
        assert!(!is_deletion_blocked_reason(
            LoweredMergeBlockedReason::TopologyRegionConflict
        ));
    }

    #[test]
    fn deleted_on_both_sides_maps_to_explicit_executable_class_when_admitted() {
        assert_eq!(
            executable_class_for_record(
                MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides),
                MergeExecutionReadiness::Admitted,
                Some(LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides),
            ),
            Some(MergeExecutableClass::ConvergeDeletedOnBothSides)
        );
    }

    #[test]
    fn blocked_reason_for_aspect_preserves_specific_manual_resolution_class() {
        assert_eq!(
            blocked_reason_for_aspect(
                crate::merge::data::MergeConflictClass::DivergentVisibleState,
                MergeResolutionClass::DivergentVisibleState,
                AspectComparisonState::Unavailable,
                MergePolicyDecisionBoundary::RequiresManualResolution {
                    class: MergeManualResolutionClass::MissingVisibleState,
                },
                MergeExecutionReadiness::Blocked,
            ),
            Some(LoweredMergeBlockedReason::MissingVisibleState)
        );
    }
}

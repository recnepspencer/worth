use crate::merge::data::{
    LoweredAspectDenialIntent, LoweredRecordDenialAspectIntent, LoweredRecordDenialKind,
    MergeResolutionClass,
};

use super::resolution::{
    blocked_reason_for_deletion_class, blocked_reason_for_topology_resolution_class,
};

pub(super) fn rejected_denial_kind_for_record(
    aspects: &[LoweredRecordDenialAspectIntent],
) -> LoweredRecordDenialKind {
    if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::RejectedCustomPolicy)
    {
        LoweredRecordDenialKind::RejectedCustomPolicy
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::RejectedMixedPolicyClasses)
    {
        LoweredRecordDenialKind::RejectedMixedPolicyClasses
    } else {
        LoweredRecordDenialKind::RejectedPolicy
    }
}

pub(super) fn blocked_denial_kind_for_record(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
    aspects: &[LoweredRecordDenialAspectIntent],
) -> LoweredRecordDenialKind {
    if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedMissingVisibleState)
    {
        LoweredRecordDenialKind::BlockedMissingVisibleState
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedMissingAncestorValueBasis)
    {
        LoweredRecordDenialKind::BlockedMissingAncestorValueBasis
    } else if aspects.iter().any(|aspect| {
        aspect.intent == LoweredAspectDenialIntent::BlockedUnvalidatedSchemaCorrespondence
    }) {
        LoweredRecordDenialKind::BlockedUnvalidatedSchemaCorrespondence
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedSourceDeletedTargetLive)
    {
        LoweredRecordDenialKind::BlockedSourceDeletedTargetLive
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedSourceLiveTargetDeleted)
    {
        LoweredRecordDenialKind::BlockedSourceLiveTargetDeleted
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedDeletedOnBothSides)
    {
        LoweredRecordDenialKind::BlockedDeletedOnBothSides
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedDeletedVsModified)
    {
        LoweredRecordDenialKind::BlockedDeletedVsModified
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedDeletedVsRewired)
    {
        LoweredRecordDenialKind::BlockedDeletedVsRewired
    } else if aspects.iter().any(|aspect| {
        aspect.intent == LoweredAspectDenialIntent::BlockedRelationEndpointRewiredLocal
    }) {
        LoweredRecordDenialKind::BlockedRelationEndpointRewiredLocal
    } else if aspects.iter().any(|aspect| {
        aspect.intent == LoweredAspectDenialIntent::BlockedRelationEndpointRewiredEscalated
    }) {
        LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated
    } else if aspects
        .iter()
        .any(|aspect| aspect.intent == LoweredAspectDenialIntent::BlockedTopologyRegionConflict)
    {
        LoweredRecordDenialKind::BlockedTopologyRegionConflict
    } else {
        fallback_blocked_denial_kind_for_record(classification, resolution_class)
    }
}

fn fallback_blocked_denial_kind_for_record(
    classification: crate::merge::data::MergeConflictClass,
    resolution_class: MergeResolutionClass,
) -> LoweredRecordDenialKind {
    match classification {
        crate::merge::data::MergeConflictClass::Deletion(class) => {
            blocked_denial_kind_from_reason(blocked_reason_for_deletion_class(class))
        }
        crate::merge::data::MergeConflictClass::RelationEndpointDivergence => {
            blocked_denial_kind_from_reason(blocked_reason_for_topology_resolution_class(
                resolution_class,
            ))
        }
        crate::merge::data::MergeConflictClass::SchemaDeclaredCorrespondence
        | crate::merge::data::MergeConflictClass::DivergentVisibleState
        | crate::merge::data::MergeConflictClass::StrategyIntentConflict
        | crate::merge::data::MergeConflictClass::ExactSharedTruth
        | crate::merge::data::MergeConflictClass::SourceOnlyAddition => {
            LoweredRecordDenialKind::BlockedManualResolution
        }
    }
}

pub(super) fn blocked_denial_kind_from_reason(
    reason: crate::merge::data::LoweredMergeBlockedReason,
) -> LoweredRecordDenialKind {
    match reason {
        crate::merge::data::LoweredMergeBlockedReason::MissingVisibleState => {
            LoweredRecordDenialKind::BlockedMissingVisibleState
        }
        crate::merge::data::LoweredMergeBlockedReason::MissingAncestorValueBasis => {
            LoweredRecordDenialKind::BlockedMissingAncestorValueBasis
        }
        crate::merge::data::LoweredMergeBlockedReason::UnvalidatedSchemaCorrespondence => {
            LoweredRecordDenialKind::BlockedUnvalidatedSchemaCorrespondence
        }
        crate::merge::data::LoweredMergeBlockedReason::SourceDeletedTargetLive => {
            LoweredRecordDenialKind::BlockedSourceDeletedTargetLive
        }
        crate::merge::data::LoweredMergeBlockedReason::SourceLiveTargetDeleted => {
            LoweredRecordDenialKind::BlockedSourceLiveTargetDeleted
        }
        crate::merge::data::LoweredMergeBlockedReason::DeletedOnBothSides => {
            LoweredRecordDenialKind::BlockedDeletedOnBothSides
        }
        crate::merge::data::LoweredMergeBlockedReason::DeletedVsModified => {
            LoweredRecordDenialKind::BlockedDeletedVsModified
        }
        crate::merge::data::LoweredMergeBlockedReason::DeletedVsRewired => {
            LoweredRecordDenialKind::BlockedDeletedVsRewired
        }
        crate::merge::data::LoweredMergeBlockedReason::RelationEndpointRewiredLocal => {
            LoweredRecordDenialKind::BlockedRelationEndpointRewiredLocal
        }
        crate::merge::data::LoweredMergeBlockedReason::RelationEndpointRewiredEscalated => {
            LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated
        }
        crate::merge::data::LoweredMergeBlockedReason::TopologyRegionConflict => {
            LoweredRecordDenialKind::BlockedTopologyRegionConflict
        }
        crate::merge::data::LoweredMergeBlockedReason::StrategyIntentConflictRequiresManualResolution => {
            LoweredRecordDenialKind::BlockedManualResolution
        }
        crate::merge::data::LoweredMergeBlockedReason::ManualConflictResolutionRequired => {
            LoweredRecordDenialKind::BlockedManualResolution
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::blocked_denial_kind_from_reason;
    use crate::merge::data::{LoweredMergeBlockedReason, LoweredRecordDenialKind};

    #[test]
    fn deletion_blocked_reasons_map_to_distinct_denial_kinds() {
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::SourceDeletedTargetLive),
            LoweredRecordDenialKind::BlockedSourceDeletedTargetLive
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::SourceLiveTargetDeleted),
            LoweredRecordDenialKind::BlockedSourceLiveTargetDeleted
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::DeletedOnBothSides),
            LoweredRecordDenialKind::BlockedDeletedOnBothSides
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::DeletedVsModified),
            LoweredRecordDenialKind::BlockedDeletedVsModified
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::DeletedVsRewired),
            LoweredRecordDenialKind::BlockedDeletedVsRewired
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::TopologyRegionConflict),
            LoweredRecordDenialKind::BlockedTopologyRegionConflict
        );
        assert_eq!(
            blocked_denial_kind_from_reason(LoweredMergeBlockedReason::MissingVisibleState),
            LoweredRecordDenialKind::BlockedMissingVisibleState
        );
        assert_eq!(
            blocked_denial_kind_from_reason(
                LoweredMergeBlockedReason::UnvalidatedSchemaCorrespondence
            ),
            LoweredRecordDenialKind::BlockedUnvalidatedSchemaCorrespondence
        );
    }
}

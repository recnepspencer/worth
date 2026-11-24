//! Typed reasons carried by deletion and topology witnesses.
use super::policy::{LoweredMergeBlockedReason, MergeResolutionClass, TopologyExecutionClass};

pub fn blocked_reason_for_deletion_class(
    class: super::conflicts::DeletionMergeClass,
) -> LoweredMergeBlockedReason {
    match class {
        super::conflicts::DeletionMergeClass::SourceDeletedTargetLive => {
            LoweredMergeBlockedReason::SourceDeletedTargetLive
        }
        super::conflicts::DeletionMergeClass::SourceLiveTargetDeleted => {
            LoweredMergeBlockedReason::SourceLiveTargetDeleted
        }
        super::conflicts::DeletionMergeClass::DeletedOnBothSides => {
            LoweredMergeBlockedReason::DeletedOnBothSides
        }
        super::conflicts::DeletionMergeClass::DeletedVsModified => {
            LoweredMergeBlockedReason::DeletedVsModified
        }
        super::conflicts::DeletionMergeClass::DeletedVsRewired => {
            LoweredMergeBlockedReason::DeletedVsRewired
        }
    }
}

pub fn blocked_reason_for_topology_resolution_class(
    resolution_class: MergeResolutionClass,
) -> LoweredMergeBlockedReason {
    match resolution_class {
        MergeResolutionClass::Topology(TopologyExecutionClass::RelationEndpointStable) => {
            LoweredMergeBlockedReason::ManualConflictResolutionRequired
        }
        MergeResolutionClass::Topology(TopologyExecutionClass::RelationEndpointRewiredLocal) => {
            LoweredMergeBlockedReason::RelationEndpointRewiredLocal
        }
        MergeResolutionClass::Topology(
            TopologyExecutionClass::RelationEndpointRewiredEscalated,
        ) => LoweredMergeBlockedReason::RelationEndpointRewiredEscalated,
        MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict) => {
            LoweredMergeBlockedReason::TopologyRegionConflict
        }
        _ => LoweredMergeBlockedReason::ManualConflictResolutionRequired,
    }
}

//! Graph truth lane: admission → identity → topology → neighborhood → inspection → mutation → closeout.

mod admission;
mod allocation_neighborhood;
#[cfg(test)]
pub(crate) use allocation_neighborhood::tests::{
    allocation_constraint_bound_reconciliation_test_support,
    allocation_constraint_equal_share_test_support, allocation_constraint_projection_tests,
    allocation_constraint_sibling_support_test_support,
};
pub(crate) use allocation_neighborhood::UiAllocationNeighborhoodMintAuthority;
pub(crate) use allocation_neighborhood::UiGraphConstraintMintAuthority;
pub(crate) use allocation_neighborhood::{
    UiAdmittedAllocationConstraintBasis, UiAllocationConstraintProvenance,
    UiGraphScrollPlanningAuthority,
};
#[cfg(test)]
pub(crate) mod allocation_neighborhood_test_support;
mod closeout;
mod identity;
mod indexes;
mod inspection;
#[cfg(test)]
mod measurement_neighborhood_hint;
#[cfg(test)]
mod measurement_neighborhood_hint_tests;
mod mount_eligibility;
mod mutation;
mod participation;
mod snapshot;
mod topology;

pub(crate) use snapshot::UiGraphAuthorityIdentity;

// --- admission (declaration → graph instantiation) ---
pub(crate) use admission::{
    admit_graph_handoffs, UiGraphNodeInstantiationInput, UiGraphTopologySeedInput,
};
pub use admission::{
    UiGraphCoreIndexContributionSeed, UiGraphInstantiationDenial, UiGraphInstantiationLocalDenial,
    UiGraphInstantiationLocalDenialKind, UiGraphInstantiationPlan, UiGraphNodeInstantiationEntry,
    UiGraphParticipationSeed, UiGraphTopologyLocalDenial, UiGraphTopologySeed,
    UiRuntimeInstanceBasisAdmission,
};

// --- allocation neighborhood (graph → planning handoff; admission sealed pub(crate)) ---
pub(crate) use allocation_neighborhood::select_replan_neighborhoods;
pub(crate) use allocation_neighborhood::UiAdmittedReplanNeighborhood;
pub(crate) use allocation_neighborhood::UiGraphNeighborhoodActivationTransition;
pub(crate) use allocation_neighborhood::UiGraphReplanConsequences;
pub(crate) use allocation_neighborhood::UiGraphReplanTransactionBasis;
pub(crate) use allocation_neighborhood::UiHostMeasurementReplanConsequence;
pub(crate) use allocation_neighborhood::UiQueryMeasurementReplanConsequence;
pub use allocation_neighborhood::{
    UiAdmittedAllocationCatalogBasisSet, UiAdmittedAllocationCatalogDelta,
    UiAdmittedAllocationInvalidationTargetSet, UiAdmittedReplanNeighborhoodSet,
    UiAllocationCatalogBasisAdmissionDenial, UiAllocationCatalogDeltaAdmissionDenial,
    UiAllocationNeighborhoodDenial, UiReplanLocalityDenial, UiReplanOverlapDisposition,
    UiReplanRootPosture, UiReplanWidenReason,
};
pub(crate) use allocation_neighborhood::{
    UiAdmittedAllocationInvalidationTarget, UiAdmittedAllocationPlanReference,
    UiGraphReplanAdmission, UiGraphReplanAuthority, UiGraphReplanTargetDisposition,
    UiReplanGenerationKey,
};

// --- closeout ---
pub use closeout::{
    UiGraphAuthority, UiGraphClosedSemanticLane, UiGraphCloseoutGuarantee, UiGraphCloseoutNonGoal,
    UiGraphCloseoutReport, UiGraphInspectionStopPoint, UiGraphInspectionSupportReport,
    UiGraphMountEligibilityRecord, UiGraphNodeRecord, UiGraphTopologyRecord,
};

// --- identity ---
pub use identity::{
    UiGraphGeneration, UiGraphGenerationRelation, UiGraphNodeIdentity, UiGraphSessionIdentityError,
    UiGraphSessionLabel, UiGraphSnapshotComparable, UiGraphWorldDifferenceKind,
    UiGraphWorldProfile, UiPreviewSessionIdentity, UiRepeatedInstanceBasis,
    UiRepeatedInstanceBasisDenial, UiRepeatedInstanceBasisKind, UiRuntimeDataInstanceKeyKind,
    UiRuntimeDataInstanceKeyToken,
};

// --- indexes / lookup ---
pub(crate) use indexes::UiAuthoredDeclarationLookup;
#[cfg(any(test, feature = "certification-support"))]
pub use indexes::UiGraphFactLookupCost;
pub use indexes::{
    UiGraphAspectConsumer, UiGraphAspectConsumerKind, UiGraphAspectPublisher,
    UiGraphAspectPublisherKind, UiGraphConsumedFactIndex, UiGraphCoreIndexes,
    UiGraphFactConsumerIdentity, UiGraphFactConsumerKey, UiGraphFactConsumerKind,
    UiGraphFactIndexBasis, UiGraphFactIndexEntry, UiGraphFactLookupDenial,
    UiGraphFactLookupReceipt, UiGraphLookup, UiGraphLookupCostClass, UiGraphLookupFamily,
    UiGraphLookupReceipt, UiGraphLookupSurface, UiGraphMosaicMembershipIndex,
    UiGraphMountEligibilityIndex, UiGraphPageMembershipIndex, UiGraphPageParticipationIndex,
    UiGraphPageParticipationMember, UiGraphParentChildIndex, UiGraphRegionMembershipIndex,
    UiGraphSlotOccupancyIndex,
};

// --- inspection ---
pub(crate) use inspection::UiGraphEvidenceRecord;
pub use inspection::{
    project_aspect_evidence_ref, project_aspect_evidence_refs, UiAspectEvidenceLane,
    UiAspectEvidenceRefProjection, UiAspectEvidenceSubjectKind, UiGraphEvidenceRef,
    UiGraphEvidenceRefKind, UiGraphInspection, UiGraphInspectionSupport, UiGraphInspectionTarget,
    UiGraphInspectionTargetKind,
};
pub(crate) use inspection::{
    UiGraphAspectEvidenceIndexes, UiGraphNodeEvidenceIndex, WorthUiAspectInspectionBoundary,
    WorthUiGraphInspectionBoundary,
};

#[cfg(test)]
pub(crate) use measurement_neighborhood_hint::UiGraphMeasurementNeighborhoodHint;

// --- mount eligibility ---
pub(crate) use mount_eligibility::materialize_graph_mount_eligibilities;
pub use mount_eligibility::{
    UiGraphMountEligibilityIdentity, UiGraphMountEligibilityMutation,
    UiGraphMountEligibilityMutationKind, UiGraphMountEligibilityRelationship,
    UiGraphMountEligibilityReservation, UiGraphMountEligibilitySeed, UiGraphMountEligibilitySlot,
    UiGraphMountEligibilityStore, UiGraphMountEligibilityTransition,
};

// --- mutation ---
#[cfg(test)]
pub(crate) use mutation::adversarial_snapshot_with_swapped_node_index_for_test;
pub(crate) use mutation::UiGraphMutationStage;
pub use mutation::{
    UiGraphMountEligibilityAdmissionDenial, UiGraphMutationCommitDenial,
    UiGraphMutationCommitResult,
};

// --- participation ---
pub(crate) use participation::materialize_graph_participation_posture;
pub use participation::{
    UiGraphAxisParticipation, UiGraphPageParticipationMutation,
    UiGraphPageParticipationMutationKind, UiGraphParticipationAxis,
    UiGraphParticipationEvidenceHandle, UiGraphParticipationMutation, UiGraphParticipationPosture,
    UiGraphParticipationReasonCode, UiGraphParticipationReasonSource, UiGraphParticipationStatus,
};

// --- snapshot ---
pub(crate) use snapshot::UiGraphNodeInput;
pub use snapshot::{
    UiGraphAttachmentPosture, UiGraphDeclarationCorrespondence, UiGraphNode, UiGraphSnapshot,
};

// --- topology ---
pub(crate) use topology::materialize_graph_topology;
pub use topology::{
    UiGraphContainmentClaim, UiGraphMembershipFacts, UiGraphMosaicMembership, UiGraphNodeTopology,
    UiGraphPageMembership, UiGraphParentResolutionClaim, UiGraphRegionMembership,
    UiGraphSlotTopology, UiGraphTopology,
};

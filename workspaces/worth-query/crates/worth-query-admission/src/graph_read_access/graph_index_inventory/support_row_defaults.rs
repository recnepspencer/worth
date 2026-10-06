use super::{
    WorthQueryGraphIndexLifecycleClass, WorthQueryGraphIndexLifecycleOwner,
    WorthQueryGraphIndexPosture, WorthQueryGraphIndexSupportState,
};
use crate::graph_read_access::{
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessRebuildBasis, WorthQueryGraphReadAccessRequirementKind,
};

pub(crate) fn support_state_for_posture(
    posture: &WorthQueryGraphIndexPosture,
) -> WorthQueryGraphIndexSupportState {
    match posture {
        WorthQueryGraphIndexPosture::Verified
        | WorthQueryGraphIndexPosture::RuntimeMaintained
        | WorthQueryGraphIndexPosture::LowerRuntimeOwned
        | WorthQueryGraphIndexPosture::EphemeralAvailable => {
            WorthQueryGraphIndexSupportState::Available
        }
        WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex => {
            WorthQueryGraphIndexSupportState::StoreOwnedUnavailable
        }
        WorthQueryGraphIndexPosture::RequiresAccessCapabilityRegistration => {
            WorthQueryGraphIndexSupportState::Declared
        }
        WorthQueryGraphIndexPosture::TemporarilyUnavailable => {
            WorthQueryGraphIndexSupportState::TemporarilyUnavailable
        }
        WorthQueryGraphIndexPosture::Denied => WorthQueryGraphIndexSupportState::Unsupported,
    }
}

pub(crate) fn support_posture_for_requirement(
    requirement_kind: &WorthQueryGraphReadAccessRequirementKind,
) -> (
    WorthQueryGraphIndexLifecycleOwner,
    WorthQueryGraphIndexLifecycleClass,
    WorthQueryGraphIndexPosture,
    Option<String>,
) {
    match requirement_kind {
        WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport
        | WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => (
            WorthQueryGraphIndexLifecycleOwner::DomainRegistration,
            WorthQueryGraphIndexLifecycleClass::AccessCapabilityRegistrationRequired,
            WorthQueryGraphIndexPosture::RequiresAccessCapabilityRegistration,
            Some(owning_milestone(requirement_kind)),
        ),
        _ => (
            WorthQueryGraphIndexLifecycleOwner::QueryRuntime,
            WorthQueryGraphIndexLifecycleClass::RuntimeMaintained,
            WorthQueryGraphIndexPosture::Verified,
            None,
        ),
    }
}

fn owning_milestone(requirement_kind: &WorthQueryGraphReadAccessRequirementKind) -> String {
    const PREFIX: &str = "worth-query-9.10-";
    let suffix = requirement_kind.as_str();
    let mut milestone = String::with_capacity(PREFIX.len() + suffix.len());
    milestone.push_str(PREFIX);
    milestone.push_str(suffix);
    milestone
}

pub(crate) fn default_bases_for_requirement(
    requirement_kind: &WorthQueryGraphReadAccessRequirementKind,
) -> (
    WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessComplexityContract,
) {
    match requirement_kind {
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency => (
            WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta,
            WorthQueryGraphReadAccessComplexityContract::DirectionalRelationLookup,
        ),
        WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency => (
            WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta,
            WorthQueryGraphReadAccessComplexityContract::ReverseRelationLookup,
        ),
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport => (
            WorthQueryGraphReadAccessRebuildBasis::SelectivityProof,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta,
            WorthQueryGraphReadAccessComplexityContract::CandidatePredicateSupport,
        ),
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport => (
            WorthQueryGraphReadAccessRebuildBasis::AuthoritativeFieldTruth,
            WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta,
            WorthQueryGraphReadAccessComplexityContract::CandidateOrderingSupport,
        ),
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
        | WorthQueryGraphReadAccessRequirementKind::VisitedSet
        | WorthQueryGraphReadAccessRequirementKind::DedupSet => (
            WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof,
            WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
            WorthQueryGraphReadAccessComplexityContract::BoundedTraversalWorkset,
        ),
        WorthQueryGraphReadAccessRequirementKind::ProofSupport => (
            WorthQueryGraphReadAccessRebuildBasis::ReadGraphProof,
            WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
            WorthQueryGraphReadAccessComplexityContract::ProofEvidenceSupport,
        ),
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer => (
            WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof,
            WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta,
            WorthQueryGraphReadAccessComplexityContract::ResultPressureBuffer,
        ),
        WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle
        | WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport
        | WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => (
            WorthQueryGraphReadAccessRebuildBasis::RuntimeSupportRequired,
            WorthQueryGraphReadAccessInvalidationBasis::RuntimeLifecycleDelta,
            WorthQueryGraphReadAccessComplexityContract::LifecycleSupportAdmission,
        ),
    }
}

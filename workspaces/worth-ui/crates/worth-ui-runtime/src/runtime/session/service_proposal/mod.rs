mod cancellation;
mod census;
mod compiler;
mod coordination;
mod occupancy;
mod participation;
mod request_basis;

#[cfg(any(test, feature = "certification-support"))]
use participation::fixture_service_family_participation;
#[cfg(feature = "certification-support")]
use request_basis::{fixture_application_generation, fixture_service_request_coherence_in};
#[cfg(test)]
use request_basis::{fixture_application_generation_in_session, fixture_service_request_coherence};

pub(in crate::runtime) use cancellation::UiServiceProposalCancellationDenial;
pub(in crate::runtime) use census::{UiServiceProposalCensus, UiServiceProposalCensusDenial};
#[cfg(feature = "certification-support")]
pub(crate) use compiler::proposal_scale_evidence;
pub(in crate::runtime) use compiler::{
    UiServiceFamilyProposal, UiServiceMountedWorkReference, UiServiceProducedFactReference,
    UiServiceProposalCandidate, UiServiceProposalCompiler,
    UiServiceProposalCompilerShutdownReceipt, UiServiceProposalDemandConstructionDenial,
    UiServiceProposalIdentity, UiServiceProposalOwnerAcknowledgement,
    UiServiceProposalPreflightDenial, UiServiceProposalPublicationDenial,
    UiServiceProposalPublicationDisposition, UiServiceProposalPublicationReceipt,
    UiServiceProposalReservationDenial, UiServiceProposalReservationOutcome,
    UiServiceProposalSettlement, UiServiceProposalStageReceipt, UiServiceProposalStagedBatch,
    UiServiceProposalStaging, UiServiceProposalStagingDenial, UiServiceProposalTeardown,
    UiServiceProposalTeardownDenial, UiServiceProposalTerminalOwnerOutcome,
    UiServiceProposalTerminalReason,
};
pub(in crate::runtime) use coordination::{
    UiDeclaredFocusSelectionAction, UiFocusRevealRequirement, UiSelectionInvocationCause,
};
#[cfg(test)]
pub(in crate::runtime) use occupancy::{
    UiServiceProposalConflictDisposition, UiServiceProposalConflictPolicy,
};
pub(in crate::runtime) use occupancy::{
    UiServiceProposalDisplacement, UiServiceProposalOccupancyDenial,
    UiServiceProposalOccupancyLease, UiServiceProposalOccupancyScopeIdentity,
    UiServiceProposalOccupancyWorkCounters,
};
pub(in crate::runtime) use participation::{
    UiServiceFamilyParticipation, UiServiceFamilyParticipationDenial,
};

#[cfg(any(test, feature = "certification-support"))]
pub(in crate::runtime) use request_basis::UiPortalCertificationServiceRequestAuthority;
#[cfg(test)]
pub(in crate::runtime) use request_basis::UiServiceRequestCoherenceAxes;
pub(in crate::runtime) use request_basis::{
    UiAdmittedIntentServiceRequestAuthority, UiPortalDismissalServiceRequestAuthority,
    UiPortalExitTerminalServiceRequestAuthority, UiServiceCancellationIdentity,
    UiServiceRequestBasis, UiServiceRequestBasisDenial, UiServiceRequestCoherence,
    UiServiceRequestCoherenceDrift, UiServiceRequestIdentity, UiServiceRequestOriginAuthority,
    UiServiceSurfaceBasis,
};

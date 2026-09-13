mod admission;
mod application_commit_authorization;
pub(in crate::domain_computation) mod application_disclosure;
mod authorization_revalidation;
mod bridge_binding;
mod bridge_observation;
mod capability_binding_lowering;
mod capability_decision_fact;
mod capability_elevation_projection;
mod capability_lowering;
mod capability_observation;
mod capability_registry;
mod capability_revalidation;
mod capability_revocation_progression;
mod decision_facts;
mod delegation_admission;
mod delegation_progression;
mod denial;
mod elevation_progression;
mod graph_work_session;
mod installed_policy;
mod lowering;
mod operation_progression;
mod operation_scope_binding;
mod retained_capability_request;
pub(in crate::domain_computation) use retained_capability_request::WorthQueryRetainedCapabilityRequest;
mod retained_capability_support;

pub(in crate::domain_computation) use crate::domain_computation::runtime_time::{
    WorthQueryRuntimeClock, WorthQueryRuntimeTimeSample,
};
pub(in crate::domain_computation) use application_commit_authorization::WorthQueryApplicationCommitAuthorization;
pub(super) use bridge_binding::bridge_authorization_binding_identity;
pub(in crate::domain_computation) use capability_decision_fact::{
    WorthQueryCapabilityCommitBasis, WorthQueryRetainedCapabilityAuthorization,
};
#[cfg(test)]
pub(in crate::domain_computation) use capability_elevation_projection::validate_elevation_projection;
pub use capability_registry::WorthQueryCapabilityPlanCompilationEvidence;
pub(in crate::domain_computation) use capability_revocation_progression::WorthQueryCapabilityRevocationBinding;
use decision_facts::WorthQueryCommitAuthorizationBasis;
pub(in crate::domain_computation) use decision_facts::{
    WorthQueryAuthorizationDecisionFact, WorthQueryPrincipalCurrentnessDependency,
    WorthQueryProviderAuthorizationDecisionFacts, WorthQueryProviderCommitAuthorization,
    WorthQueryProviderDecisionFactBinding, WorthQueryRegisteredCommitAuthorization,
    WorthQueryRetainedAuthorizationDecisionFacts,
};
pub(in crate::domain_computation) use delegation_progression::{
    WorthQueryDelegationActivationBinding, WorthQueryDelegationActivationEffect,
};
pub use denial::{
    WorthQueryApplicationAuthorizationExplanationCause, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialIdentity, WorthQueryOperationAuthorizationDenialKind,
};
pub(in crate::domain_computation) use elevation_progression::{
    WorthQueryCurrentElevationSupport, WorthQueryElevationApprovalBinding,
    WorthQueryElevationApprovalBindingPermit, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};
pub use elevation_progression::{
    WorthQueryElevationApprovalAuthorizationDenial, WorthQueryElevationCloseAuthorizationDenial,
    WorthQueryMandatoryReviewAuthorizationDenial,
};
pub(in crate::domain_computation) use installed_policy::WorthQueryInstalledAuthorizationRegistry;
pub(in crate::domain_computation) use operation_progression::admit_capability_access;
pub(in crate::domain_computation) use operation_progression::progress_conventional_operation;
pub use operation_progression::WorthQueryAdmittedApplicationCapabilityAccess;
pub use operation_progression::WorthQueryAdmittedApplicationOperation;
pub(in crate::domain_computation) use operation_progression::WorthQueryOperationAdmissionIdentity;
pub use operation_scope_binding::{
    WorthQueryOperationScopeBinding, WorthQueryOperationScopeEntityBinding,
};
pub(in crate::domain_computation) use retained_capability_support::{
    WorthQueryCapabilitySupportCommitBasis, WorthQueryRetainedCapabilitySupport,
};

fn authorization_denial(
    subject: impl Into<String>,
    _detail: &'static str,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy,
        subject,
    )
}

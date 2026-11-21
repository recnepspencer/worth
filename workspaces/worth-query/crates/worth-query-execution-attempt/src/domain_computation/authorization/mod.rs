pub use worth_query_execution_lineage::facade::domain_computation::authorization::*;
pub mod capability_revocation_progression;
pub mod operation_admission;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub mod admission;
pub mod application_commit_authorization;
pub use home_incarnation::refuse_stale_home_authority;
pub use worth_query_execution_lineage::facade::domain_computation::authorization::{
    admitted_request, bridge_binding, capability_binding_lowering, capability_lowering,
    capability_registry, denial, home_incarnation, lowering, operation_scope_binding,
};
pub mod authorization_revalidation;
pub mod bridge_observation;
pub mod capability_decision_fact;
pub mod capability_elevation_projection;
pub mod capability_observation;
pub mod capability_revalidation;
pub mod decision_facts;
pub use decision_facts::AuthorizationCurrentnessStop;
pub use decision_facts::{
    SourceAuthorizationLocus, SourceReadSupport, SourceReadSupportCurrentnessStop,
};
pub mod delegation_admission;
pub mod delegation_progression;
pub mod elevation_progression;
pub mod installed_policy;
pub mod operation_progression;
pub use operation_scope_binding::WorthQueryDecisionScopeWitness;
pub mod retained_capability_request;
pub use retained_capability_request::WorkflowApprovalRequestDecodingDenial;
pub use retained_capability_request::WorkflowApprovalRequestEncodingDenial;
pub use retained_capability_request::WorthQueryCapabilityUseIdentity;
pub use retained_capability_request::WorthQueryRetainedCapabilityRequest;
pub use retained_capability_request::MAXIMUM_PORTABLE_BYTES;
pub mod retained_capability_support;
pub mod workflow_approval_authority;
pub use workflow_approval_authority::decoding::decode_lineage;
pub use workflow_approval_authority::decoding::decode_principal;
pub use workflow_approval_authority::decoding::DecodedApprovalLineage;
pub mod workflow_approval_dependencies;
pub use workflow_approval_authority::decode_workflow_approval_support;
pub use workflow_approval_authority::encode_workflow_approval_support;
pub use workflow_approval_authority::WorthQueryWorkflowApprovalAuthorityBasis;
pub use workflow_approval_dependencies::encode_workflow_approval_dependencies;
pub use workflow_approval_dependencies::WorthQueryWorkflowApprovalDependencies;

pub use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
pub use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;

pub use application_commit_authorization::WorthQueryApplicationCommitAuthorization;
pub use bridge_binding::bridge_authorization_binding_identity;
pub use capability_decision_fact::WorthQueryCapabilityCommitBasis;

pub use capability_registry::WorthQueryCapabilityPlanCompilationEvidence;
pub use decision_facts::WorthQueryAuthorizationDecisionFact;
pub use decision_facts::WorthQueryCommitAuthorizationBasis;
pub use decision_facts::WorthQueryDurableAuthorizationDependencies;
pub use decision_facts::WorthQueryDurableCapabilityLineage;
pub use decision_facts::WorthQueryPrincipalCurrentnessDependency;
pub use decision_facts::WorthQueryProviderAuthorizationDecisionFacts;
pub use decision_facts::WorthQueryProviderDecisionFactBinding;
pub use decision_facts::WorthQueryRetainedAuthorizationDecisionFacts;
pub use denial::WorthQueryAdmissionLapse;
pub use denial::WorthQueryApplicationAuthorizationExplanationCause;
pub use denial::WorthQueryOperationAuthorizationDenial;
pub use denial::WorthQueryOperationAuthorizationDenialIdentity;
pub use denial::WorthQueryOperationAuthorizationDenialKind;
pub use denial::WorthQueryOperationAuthorizationRefusalKind;
pub use installed_policy::WorthQueryInstalledAuthorizationRegistry;
pub use operation_scope_binding::WorthQueryOperationScopeBinding;
pub use operation_scope_binding::WorthQueryOperationScopeEntityBinding;
pub use retained_capability_support::WorthQueryCapabilitySupportCommitBasis;
pub use retained_capability_support::WorthQueryCapabilitySupportRole;
pub use retained_capability_support::WorthQueryRetainedCapabilitySupport;

fn authorization_denial(
    subject: impl Into<String>,
    detail: &'static str,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        ),
        format!("{}: {detail}", subject.into()),
    )
}

pub use capability_decision_fact::WorthQueryRetainedCapabilityAuthorization;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub mod inline_type_budget;

pub use capability_revocation_progression::*;
pub use elevation_progression::*;
pub use operation_admission::{
    WorthQueryAdmittedApplicationOperation, WorthQueryOperationAdmissionIdentity,
};

pub use decision_facts::{
    WorthQueryProviderCommitAuthorization, WorthQueryRegisteredCommitAuthorization,
};
pub use delegation_progression::binding::WorthQueryDelegationActivationBinding;
pub use operation_admission::admitted_operation_core::{
    WorthQueryAdmittedOperation, WorthQueryAdmittedOperationData,
};
pub use operation_admission::ReadCompletionAdmission;

pub use operation_admission::brand_tag::BrandTag;

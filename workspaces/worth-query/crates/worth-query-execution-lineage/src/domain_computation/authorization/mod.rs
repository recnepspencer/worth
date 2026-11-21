pub mod admitted_request;
pub mod application_disclosure;
pub mod bridge_binding;
pub mod capability_binding_lowering;
pub mod capability_lowering;
pub mod capability_registry;
pub mod decision_facts {
    pub mod source_read_support;
    pub use source_read_support::*;
}
pub use decision_facts::SourceReadSupportCurrentnessStop;
pub mod denial;
pub mod home_incarnation;
pub mod lowering;
pub mod operation_scope_binding;

pub use crate::domain_computation::runtime_time::{
    WorthQueryRuntimeClock, WorthQueryRuntimeTimeSample,
};
pub use bridge_binding::bridge_authorization_binding_identity;
pub use capability_registry::WorthQueryCapabilityPlanCompilationEvidence;
pub use denial::{
    WorthQueryAdmissionLapse, WorthQueryApplicationAuthorizationExplanationCause,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialIdentity,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
};
pub use home_incarnation::refuse_stale_home_authority;
pub use operation_scope_binding::{
    WorthQueryDecisionScopeWitness, WorthQueryOperationScopeBinding,
    WorthQueryOperationScopeEntityBinding,
};
pub use worth_query_execution_contracts::facade::WorthQueryOperationAdmissionIdentity;

pub fn authorization_denial(
    subject: impl Into<String>,
    detail: &'static str,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::InvalidInstalledPolicy,
        ),
        format!("{}: {detail}", subject.into()),
    )
}

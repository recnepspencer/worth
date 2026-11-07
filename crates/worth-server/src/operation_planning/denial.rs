use crate::{
    request_context::DiagnosticRichnessProfile, WorthServerOperationReadinessDenial,
    WorthServerOperationReadinessDenialCode, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDenialCode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerOperationPlanDenialCode {
    EvidenceIdentityDerivationDenied(
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    PreparedIntentMismatch,
    WorkspaceBindingFailed,
    SupportDenied,
    PreconditionDenied,
    DownstreamDeliveryRequiresReadIntent,
    RuntimeBackedResumeUnsupported,
    DurableResumeDeferred,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationPlanDenial {
    code: WorthServerOperationPlanDenialCode,
    diagnostics_profile: DiagnosticRichnessProfile,
    detail: String,
}

impl WorthServerOperationPlanDenial {
    pub(crate) fn new(
        code: WorthServerOperationPlanDenialCode,
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            diagnostics_profile,
            detail: detail.into(),
        }
    }

    pub(crate) fn from_readiness_denial(
        denial: WorthServerOperationReadinessDenial,
        diagnostics_profile: DiagnosticRichnessProfile,
    ) -> Self {
        let code = match denial.code() {
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput
            | WorthServerOperationReadinessDenialCode::PreconditionFailed => {
                WorthServerOperationPlanDenialCode::PreconditionDenied
            }
            WorthServerOperationReadinessDenialCode::DownstreamDeliveryRequiresReadIntent => {
                WorthServerOperationPlanDenialCode::DownstreamDeliveryRequiresReadIntent
            }
            WorthServerOperationReadinessDenialCode::RuntimeBackedResumeUnsupported => {
                WorthServerOperationPlanDenialCode::RuntimeBackedResumeUnsupported
            }
            WorthServerOperationReadinessDenialCode::DurableResumeDeferred => {
                WorthServerOperationPlanDenialCode::DurableResumeDeferred
            }
            WorthServerOperationReadinessDenialCode::MissingQuerySupport
            | WorthServerOperationReadinessDenialCode::UnsupportedQuerySupport
            | WorthServerOperationReadinessDenialCode::UnsupportedProductSupport
            | WorthServerOperationReadinessDenialCode::UnknownProductSupport
            | WorthServerOperationReadinessDenialCode::FixtureOnlyProductSupport
            | WorthServerOperationReadinessDenialCode::IncompatibleSupportBasis => {
                WorthServerOperationPlanDenialCode::SupportDenied
            }
        };
        Self::new(code, diagnostics_profile, denial.detail())
    }

    pub(crate) fn into_query_handoff_denial(self) -> WorthServerQueryHandoffDenial {
        let code = match self.code {
            WorthServerOperationPlanDenialCode::PreparedIntentMismatch => {
                WorthServerQueryHandoffDenialCode::PreparedIntentMismatch
            }
            WorthServerOperationPlanDenialCode::WorkspaceBindingFailed => {
                WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed
            }
            WorthServerOperationPlanDenialCode::SupportDenied => {
                WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
            }
            WorthServerOperationPlanDenialCode::PreconditionDenied => {
                WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
            }
            WorthServerOperationPlanDenialCode::DownstreamDeliveryRequiresReadIntent => {
                WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
            }
            WorthServerOperationPlanDenialCode::RuntimeBackedResumeUnsupported => {
                WorthServerQueryHandoffDenialCode::RuntimeBackedResumeUnsupported
            }
            WorthServerOperationPlanDenialCode::DurableResumeDeferred => {
                WorthServerQueryHandoffDenialCode::DurableResumeDeferred
            }
        };
        WorthServerQueryHandoffDenial::new(code, self.diagnostics_profile, self.detail)
    }

    pub fn code(&self) -> WorthServerOperationPlanDenialCode {
        self.code
    }

    pub fn diagnostics_profile(&self) -> DiagnosticRichnessProfile {
        self.diagnostics_profile
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

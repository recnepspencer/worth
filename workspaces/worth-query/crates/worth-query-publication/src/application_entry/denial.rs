use worth_query_execution::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryOperationAuthorizationDenial,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryEntityResolutionDenial, WorthQueryPrincipalResolutionDenial,
    WorthQueryProductBranchAdmissionDenial,
};
use worth_query_installation::facade::{
    WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationOperationInstallationDenial, WorthQueryApplicationQueryInstallationDenial,
    WorthQueryApplicationQueryLimitDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationRequestQueryDenialKind {
    BindingInstallation,
    Limit,
    ProductSelection,
    OutputSettlement,
    PrincipalResolution,
    ScopeResolution,
    Admission,
    Execution,
    ContinuationExecution,
    RequestMode,
    CapabilityInstallation,
    CapabilityAdmission,
}

#[derive(Debug)]
pub enum WorthQueryApplicationRequestQueryDenial {
    BindingInstallation(WorthQueryApplicationQueryInstallationDenial),
    Limit(WorthQueryApplicationQueryLimitDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    OutputSettlement(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    ScopeResolution(WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Execution(WorthQueryApplicationOneShotDenial),
    ContinuationExecution(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationContinuationDenial,
    ),
    RequestMode,
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    CapabilityAdmission(WorthQueryOperationAuthorizationDenial),
}

impl WorthQueryApplicationRequestQueryDenial {
    pub const fn kind(&self) -> WorthQueryApplicationRequestQueryDenialKind {
        match self {
            Self::BindingInstallation(_) => {
                WorthQueryApplicationRequestQueryDenialKind::BindingInstallation
            }
            Self::Limit(_) => WorthQueryApplicationRequestQueryDenialKind::Limit,
            Self::ProductSelection(_) => {
                WorthQueryApplicationRequestQueryDenialKind::ProductSelection
            }
            Self::OutputSettlement(_) => {
                WorthQueryApplicationRequestQueryDenialKind::OutputSettlement
            }
            Self::PrincipalResolution(_) => {
                WorthQueryApplicationRequestQueryDenialKind::PrincipalResolution
            }
            Self::ScopeResolution(_) => {
                WorthQueryApplicationRequestQueryDenialKind::ScopeResolution
            }
            Self::Admission(_) => WorthQueryApplicationRequestQueryDenialKind::Admission,
            Self::Execution(_) => WorthQueryApplicationRequestQueryDenialKind::Execution,
            Self::ContinuationExecution(_) => {
                WorthQueryApplicationRequestQueryDenialKind::ContinuationExecution
            }
            Self::RequestMode => WorthQueryApplicationRequestQueryDenialKind::RequestMode,
            Self::CapabilityInstallation(_) => {
                WorthQueryApplicationRequestQueryDenialKind::CapabilityInstallation
            }
            Self::CapabilityAdmission(_) => {
                WorthQueryApplicationRequestQueryDenialKind::CapabilityAdmission
            }
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationRequestQueryDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application request query denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryApplicationRequestQueryDenial {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationRequestMutationDenialKind {
    BindingInstallation,
    CapabilityInstallation,
    ProductSelection,
    PrincipalResolution,
    ScopeResolution,
    Authorization,
    Idempotency,
    Handler,
    SourceExpectation,
    ApplicationProgramRequired,
    ApplicationProgramMismatch,
    RequiresWorkflowTransition,
    WorkflowAuthoritySpent,
    WorkflowTransitionCurrentness,
}

#[derive(Debug)]
pub enum WorthQueryApplicationRequestMutationDenial {
    BindingInstallation(WorthQueryApplicationOperationInstallationDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    ScopeResolution(WorthQueryEntityResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Idempotency(WorthQueryApplicationIdempotencyResolutionDenial),
    Handler(MutationHandlerExecutionDenial),
    SourceExpectation(
        worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial,
    ),
    ApplicationProgramRequired,
    ApplicationProgramMismatch,
    RequiresWorkflowTransition,
    WorkflowAuthoritySpent,
    WorkflowTransitionCurrentness(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationAttemptDenial,
    ),
}

impl WorthQueryApplicationRequestMutationDenial {
    pub const fn kind(&self) -> WorthQueryApplicationRequestMutationDenialKind {
        match self {
            Self::BindingInstallation(_) => {
                WorthQueryApplicationRequestMutationDenialKind::BindingInstallation
            }
            Self::CapabilityInstallation(_) => {
                WorthQueryApplicationRequestMutationDenialKind::CapabilityInstallation
            }
            Self::ProductSelection(_) => {
                WorthQueryApplicationRequestMutationDenialKind::ProductSelection
            }
            Self::PrincipalResolution(_) => {
                WorthQueryApplicationRequestMutationDenialKind::PrincipalResolution
            }
            Self::ScopeResolution(_) => {
                WorthQueryApplicationRequestMutationDenialKind::ScopeResolution
            }
            Self::Authorization(_) => WorthQueryApplicationRequestMutationDenialKind::Authorization,
            Self::Idempotency(_) => WorthQueryApplicationRequestMutationDenialKind::Idempotency,
            Self::Handler(_) => WorthQueryApplicationRequestMutationDenialKind::Handler,
            Self::SourceExpectation(_) => {
                WorthQueryApplicationRequestMutationDenialKind::SourceExpectation
            }
            Self::ApplicationProgramRequired => {
                WorthQueryApplicationRequestMutationDenialKind::ApplicationProgramRequired
            }
            Self::ApplicationProgramMismatch => {
                WorthQueryApplicationRequestMutationDenialKind::ApplicationProgramMismatch
            }
            Self::RequiresWorkflowTransition => {
                WorthQueryApplicationRequestMutationDenialKind::RequiresWorkflowTransition
            }
            Self::WorkflowAuthoritySpent => {
                WorthQueryApplicationRequestMutationDenialKind::WorkflowAuthoritySpent
            }
            Self::WorkflowTransitionCurrentness(_) => {
                WorthQueryApplicationRequestMutationDenialKind::WorkflowTransitionCurrentness
            }
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationRequestMutationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application mutation request denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryApplicationRequestMutationDenial {}

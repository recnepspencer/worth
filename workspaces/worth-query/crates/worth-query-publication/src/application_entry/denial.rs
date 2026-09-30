use worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial;
use worth_query_execution::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationIdempotencyResolutionDenialKind, WorthQueryOperationAuthorizationDenial,
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

/// The kind of a `WorthQueryApplicationRequestQueryDenial`, without its detail.
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

/// Why an application query request was refused before or during execution.
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

/// The kind of a `WorthQueryApplicationRequestMutationDenial`, without its detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationRequestMutationDenialKind {
    BindingInstallation,
    CapabilityInstallation,
    ProductSelection,
    PrincipalResolution,
    ScopeResolution,
    Authorization,
    Idempotency,
    /// The key is recorded with the same intent, and that commit took effect,
    /// but this runtime no longer holds its receipt. Retrying the same request
    /// cannot commit it again.
    IdempotencyReceiptNotRetained,
    /// The key's recorded intent was written by an earlier encoding that
    /// cannot be checked against this request.
    IdempotencyIntentUnverifiable,
    Identity,
    /// The handler ran and refused, or its projection, read attempt or
    /// candidate program failed.
    Handler,
    /// The binding is a workflow control step, which only a workflow transition
    /// may commit. Retrying the same request cannot succeed.
    WorkflowControl,
    /// The admission governed a different input than the request carries.
    /// Retrying the same request cannot succeed.
    InputNotAdmitted,
    /// The runtime installed no handler for the binding. Retrying the same
    /// request cannot succeed until the runtime is reconfigured.
    HandlerNotInstalled,
    SourceExpectation,
    ProgramSelection,
    ApplicationProgramRequired,
    ApplicationProgramMismatch,
    RequiresWorkflowTransition,
    WorkflowAuthoritySpent,
    WorkflowTransitionCurrentness,
}

/// Why an application mutation request was refused before its commit was attempted.
#[derive(Debug)]
pub enum WorthQueryApplicationRequestMutationDenial {
    BindingInstallation(WorthQueryApplicationOperationInstallationDenial),
    CapabilityInstallation(WorthQueryApplicationCapabilityInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    ScopeResolution(WorthQueryEntityResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Idempotency(WorthQueryApplicationIdempotencyResolutionDenial),
    /// The idempotency key or the input could not be encoded into its canonical
    /// identity, so no retry could be told apart from a changed request. The
    /// payload names which of the two failed.
    Identity(ApplicationMutationIdentityDenial),
    Handler(MutationHandlerExecutionDenial),
    SourceExpectation(
        worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial,
    ),
    /// The branch's adopted program could not be inspected, so no
    /// installed owner could be chosen for it.
    ProgramSelection(
        worth_query_execution::facade::primary_graph::WorthQuerySelectedProgramInspectionDenial,
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
            Self::Idempotency(denial) => match denial.kind() {
                WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
                    ..
                } => WorthQueryApplicationRequestMutationDenialKind::IdempotencyReceiptNotRetained,
                WorthQueryApplicationIdempotencyResolutionDenialKind::RecordedIntentUnverifiable => {
                    WorthQueryApplicationRequestMutationDenialKind::IdempotencyIntentUnverifiable
                }
                _ => WorthQueryApplicationRequestMutationDenialKind::Idempotency,
            },
            Self::Identity(_) => WorthQueryApplicationRequestMutationDenialKind::Identity,
            Self::Handler(MutationHandlerExecutionDenial::WorkflowControl) => {
                WorthQueryApplicationRequestMutationDenialKind::WorkflowControl
            }
            Self::Handler(MutationHandlerExecutionDenial::InputNotAdmitted) => {
                WorthQueryApplicationRequestMutationDenialKind::InputNotAdmitted
            }
            Self::Handler(MutationHandlerExecutionDenial::HandlerNotInstalled) => {
                WorthQueryApplicationRequestMutationDenialKind::HandlerNotInstalled
            }
            Self::Handler(_) => WorthQueryApplicationRequestMutationDenialKind::Handler,
            Self::SourceExpectation(_) => {
                WorthQueryApplicationRequestMutationDenialKind::SourceExpectation
            }
            Self::ProgramSelection(_) => {
                WorthQueryApplicationRequestMutationDenialKind::ProgramSelection
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

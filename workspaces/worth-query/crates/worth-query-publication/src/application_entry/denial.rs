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
    WorthQueryApplicationOperationInstallationDenial, WorthQueryApplicationQueryInstallationDenial,
    WorthQueryApplicationQueryLimitDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationRequestQueryDenialKind {
    BindingInstallation,
    Limit,
    ProductSelection,
    PrincipalResolution,
    ScopeResolution,
    Admission,
    Execution,
}

#[derive(Debug)]
pub enum WorthQueryApplicationRequestQueryDenial {
    BindingInstallation(WorthQueryApplicationQueryInstallationDenial),
    Limit(WorthQueryApplicationQueryLimitDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    ScopeResolution(WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Execution(WorthQueryApplicationOneShotDenial),
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
            Self::PrincipalResolution(_) => {
                WorthQueryApplicationRequestQueryDenialKind::PrincipalResolution
            }
            Self::ScopeResolution(_) => {
                WorthQueryApplicationRequestQueryDenialKind::ScopeResolution
            }
            Self::Admission(_) => WorthQueryApplicationRequestQueryDenialKind::Admission,
            Self::Execution(_) => WorthQueryApplicationRequestQueryDenialKind::Execution,
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
    ProductSelection,
    PrincipalResolution,
    ScopeResolution,
    Authorization,
    Idempotency,
    Handler,
}

#[derive(Debug)]
pub enum WorthQueryApplicationRequestMutationDenial {
    BindingInstallation(WorthQueryApplicationOperationInstallationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenial),
    ScopeResolution(WorthQueryEntityResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Idempotency(WorthQueryApplicationIdempotencyResolutionDenial),
    Handler(MutationHandlerExecutionDenial),
}

impl WorthQueryApplicationRequestMutationDenial {
    pub const fn kind(&self) -> WorthQueryApplicationRequestMutationDenialKind {
        match self {
            Self::BindingInstallation(_) => {
                WorthQueryApplicationRequestMutationDenialKind::BindingInstallation
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

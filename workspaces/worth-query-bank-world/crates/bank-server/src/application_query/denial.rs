//! Bank-owned closed descriptions of public application-query denials.

mod admission;
mod execution;
mod installation;

pub use admission::{
    BankApplicationQueryAdmissionDenialKind, BankApplicationQueryParameterDenialKind,
    BankGraphReadPlanReviewDenialKind,
};
pub use execution::{
    BankApplicationContinuationDenialKind, BankApplicationLiveOpenDenialKind,
    BankApplicationOneShotDenialKind, BankApplicationPreviewSessionDenialKind,
    BankApplicationProjectionDenialKind, BankProductSelectionDenialKind,
};
pub use installation::{
    BankApplicationCapabilityInstallationDenialKind, BankApplicationQueryInstallationDenialKind,
};

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistorySelectionDenial, WorthQueryApplicationLiveOpenRequestDenial,
    WorthQueryApplicationRequestQueryDenial,
};
use worth_query_host::facade::domain::{
    WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationQueryInstallationDenial, WorthQueryApplicationQueryLimitDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationContinuationDenial, WorthQueryApplicationLiveOpenDenial,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryEntityResolutionDenial, WorthQueryOperationAuthorizationDenial,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPrincipalResolutionDenialKind, WorthQueryProductBranchAdmissionDenial,
};

use crate::{BankAuthorizationDenial, BankEntityResolutionDenial};
use admission::admission;
use execution::{continuation, live, one_shot, product_selection};
use installation::{capability_installation, query_installation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankApplicationQueryLaneDenial<Kind> {
    kind: Kind,
    authorization: Option<BankAuthorizationDenial>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationOutputSettlementDenialKind {
    ForeignSource,
    MissingApplicableProducer,
    AmbiguousApplicableProducer,
    ProducerUnavailable,
    ProductSelection(BankProductSelectionDenialKind),
    SchedulingRejected,
    SchedulingDeferred,
    PublicationStale,
    NoEffect,
    Superseded,
    Cancelled,
    TimedOut,
    WorkBudgetExceeded,
    RetentionBudgetExceeded,
    PublicationCapacityExceeded,
    ForeignDemand,
    ForeignSettlement,
    RetainedBasisUnavailable,
    Closed,
    DuplicatePerformedSource,
    IncompleteDependencyCoverage,
}

impl<Kind> BankApplicationQueryLaneDenial<Kind>
where
    Kind: Copy,
{
    pub const fn kind(self) -> Kind {
        self.kind
    }

    pub const fn authorization(self) -> Option<BankAuthorizationDenial> {
        self.authorization
    }

    fn from_query(
        kind: Kind,
        authorization: Option<&WorthQueryOperationAuthorizationDenial>,
    ) -> Self {
        Self {
            kind,
            authorization: authorization
                .cloned()
                .map(BankAuthorizationDenial::from_query),
        }
    }
}

#[derive(Debug)]
pub enum BankApplicationQueryDenial {
    RequestMode,
    LiveRetainedBasis,
    LiveControls(worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControlDenial),
    Installation(BankApplicationQueryInstallationDenialKind),
    CapabilityInstallation(BankApplicationCapabilityInstallationDenialKind),
    CapabilityAdmission(BankAuthorizationDenial),
    ScopeResolution(BankEntityResolutionDenial),
    PrincipalResolution(WorthQueryPrincipalResolutionDenialKind),
    Limit(WorthQueryApplicationQueryLimitDenial),
    ProductSelection(BankProductSelectionDenialKind),
    HistoricalCommitUnavailable,
    PreviewSession(BankApplicationPreviewSessionDenialKind),
    Admission(BankApplicationQueryLaneDenial<BankApplicationQueryAdmissionDenialKind>),
    Execution(BankApplicationQueryLaneDenial<BankApplicationOneShotDenialKind>),
    OutputSettlement(BankApplicationOutputSettlementDenialKind),
    ContinuationExecution(BankApplicationQueryLaneDenial<BankApplicationContinuationDenialKind>),
    LiveOpen(BankApplicationQueryLaneDenial<BankApplicationLiveOpenDenialKind>),
}

impl BankApplicationQueryDenial {
    pub(crate) fn from_history_selection(
        denial: WorthQueryApplicationHistorySelectionDenial,
    ) -> Self {
        match denial {
            WorthQueryApplicationHistorySelectionDenial::ProductSelection(denial) => {
                Self::from_product_selection(denial)
            }
            WorthQueryApplicationHistorySelectionDenial::CommitUnavailable => {
                Self::HistoricalCommitUnavailable
            }
        }
    }

    pub(crate) fn from_live_request_open(
        denial: WorthQueryApplicationLiveOpenRequestDenial,
    ) -> Self {
        match denial {
            WorthQueryApplicationLiveOpenRequestDenial::RetainedBasis => Self::LiveRetainedBasis,
            WorthQueryApplicationLiveOpenRequestDenial::BindingInstallation(denial) => {
                Self::from_installation(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::CapabilityInstallation(denial) => {
                Self::from_capability_installation(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::CapabilityAdmission(denial) => {
                Self::from_capability_admission(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::Limit(denial) => Self::Limit(denial),
            WorthQueryApplicationLiveOpenRequestDenial::ProductSelection(denial) => {
                Self::from_product_selection(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution(denial) => {
                Self::PrincipalResolution(denial.kind())
            }
            WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution(denial) => {
                Self::from_scope_resolution(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::Controls(denial) => {
                Self::LiveControls(denial)
            }
            WorthQueryApplicationLiveOpenRequestDenial::Open(denial) => {
                Self::from_live_open(denial)
            }
        }
    }
    pub(crate) fn from_request_query(denial: WorthQueryApplicationRequestQueryDenial) -> Self {
        match denial {
            WorthQueryApplicationRequestQueryDenial::ProductSelection(denial) => {
                Self::from_product_selection(denial)
            }
            WorthQueryApplicationRequestQueryDenial::BindingInstallation(denial) => {
                Self::from_installation(denial)
            }
            WorthQueryApplicationRequestQueryDenial::Limit(denial) => Self::Limit(denial),
            WorthQueryApplicationRequestQueryDenial::PrincipalResolution(denial) => {
                Self::PrincipalResolution(denial.kind())
            }
            WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial) => {
                Self::from_scope_resolution(denial)
            }
            WorthQueryApplicationRequestQueryDenial::Admission(denial) => {
                Self::from_admission(denial)
            }
            WorthQueryApplicationRequestQueryDenial::Execution(denial) => {
                Self::from_execution(denial)
            }
            WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial) => {
                Self::from_continuation_execution(denial)
            }
            WorthQueryApplicationRequestQueryDenial::RequestMode => Self::RequestMode,
            WorthQueryApplicationRequestQueryDenial::CapabilityInstallation(denial) => {
                Self::from_capability_installation(denial)
            }
            WorthQueryApplicationRequestQueryDenial::CapabilityAdmission(denial) => {
                Self::from_capability_admission(denial)
            }
            WorthQueryApplicationRequestQueryDenial::OutputSettlement(denial) => {
                Self::from_output_settlement(denial)
            }
        }
    }

    pub(crate) fn from_installation(denial: WorthQueryApplicationQueryInstallationDenial) -> Self {
        Self::Installation(query_installation(denial.kind()))
    }

    pub(crate) fn from_capability_installation(
        denial: WorthQueryApplicationCapabilityInstallationDenial,
    ) -> Self {
        Self::CapabilityInstallation(capability_installation(denial.kind()))
    }

    pub(crate) fn from_capability_admission(
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self::CapabilityAdmission(BankAuthorizationDenial::from_query(denial))
    }

    pub(crate) fn from_scope_resolution(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::ScopeResolution(BankEntityResolutionDenial::from_query(denial.kind()))
    }

    pub(crate) const fn from_product_selection(
        denial: WorthQueryProductBranchAdmissionDenial,
    ) -> Self {
        Self::ProductSelection(product_selection(denial))
    }

    pub(crate) fn from_admission(denial: WorthQueryApplicationQueryAdmissionDenial) -> Self {
        Self::Admission(BankApplicationQueryLaneDenial::from_query(
            admission(denial.kind()),
            denial.authorization_denial(),
        ))
    }

    pub(crate) fn from_execution(denial: WorthQueryApplicationOneShotDenial) -> Self {
        Self::Execution(BankApplicationQueryLaneDenial::from_query(
            one_shot(denial.kind()),
            denial.authorization_denial(),
        ))
    }

    fn from_output_settlement(denial: WorthQueryOutputDemandDenial) -> Self {
        use BankApplicationOutputSettlementDenialKind as Bank;
        use WorthQueryOutputDemandDenialKind as Query;

        let kind = match denial.kind() {
            Query::ForeignSource => Bank::ForeignSource,
            Query::MissingApplicableProducer => Bank::MissingApplicableProducer,
            Query::AmbiguousApplicableProducer => Bank::AmbiguousApplicableProducer,
            Query::ProducerUnavailable => Bank::ProducerUnavailable,
            Query::ProductSelection(denial) => Bank::ProductSelection(product_selection(denial)),
            Query::SchedulingRejected => Bank::SchedulingRejected,
            Query::SchedulingDeferred => Bank::SchedulingDeferred,
            Query::PublicationStale => Bank::PublicationStale,
            Query::NoEffect => Bank::NoEffect,
            Query::Superseded => Bank::Superseded,
            Query::Cancelled => Bank::Cancelled,
            Query::TimedOut => Bank::TimedOut,
            Query::WorkBudgetExceeded => Bank::WorkBudgetExceeded,
            Query::RetentionBudgetExceeded => Bank::RetentionBudgetExceeded,
            Query::PublicationCapacityExceeded => Bank::PublicationCapacityExceeded,
            Query::ForeignDemand => Bank::ForeignDemand,
            Query::ForeignSettlement => Bank::ForeignSettlement,
            Query::RetainedBasisUnavailable => Bank::RetainedBasisUnavailable,
            Query::Closed => Bank::Closed,
            Query::DuplicatePerformedSource => Bank::DuplicatePerformedSource,
            Query::IncompleteDependencyCoverage => Bank::IncompleteDependencyCoverage,
        };
        Self::OutputSettlement(kind)
    }

    pub(crate) fn from_continuation_execution(
        denial: WorthQueryApplicationContinuationDenial,
    ) -> Self {
        Self::ContinuationExecution(BankApplicationQueryLaneDenial::from_query(
            continuation(denial.kind()),
            denial.authorization_denial(),
        ))
    }

    pub(crate) fn from_live_open(denial: WorthQueryApplicationLiveOpenDenial) -> Self {
        Self::LiveOpen(BankApplicationQueryLaneDenial::from_query(
            live(denial.kind()),
            denial.authorization_denial(),
        ))
    }
}

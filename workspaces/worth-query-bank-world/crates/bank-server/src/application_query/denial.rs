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

use worth_query_host::facade::domain::{
    WorthQueryApplicationCapabilityInstallationDenial, WorthQueryApplicationQueryInstallationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationContinuationDenial, WorthQueryApplicationLiveOpenDenial,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryEntityResolutionDenial, WorthQueryOperationAuthorizationDenial,
    WorthQueryProductBranchAdmissionDenial,
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
    Installation(BankApplicationQueryInstallationDenialKind),
    CapabilityInstallation(BankApplicationCapabilityInstallationDenialKind),
    CapabilityAdmission(BankAuthorizationDenial),
    ScopeResolution(BankEntityResolutionDenial),
    ProductSelection(BankProductSelectionDenialKind),
    HistoricalCommitUnavailable,
    PreviewSession(BankApplicationPreviewSessionDenialKind),
    Admission(BankApplicationQueryLaneDenial<BankApplicationQueryAdmissionDenialKind>),
    Execution(BankApplicationQueryLaneDenial<BankApplicationOneShotDenialKind>),
    ContinuationExecution(BankApplicationQueryLaneDenial<BankApplicationContinuationDenialKind>),
    LiveOpen(BankApplicationQueryLaneDenial<BankApplicationLiveOpenDenialKind>),
}

impl BankApplicationQueryDenial {
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

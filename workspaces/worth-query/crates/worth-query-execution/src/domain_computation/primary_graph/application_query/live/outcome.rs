use std::marker::PhantomData;

use super::super::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAccessReceipt,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveOpenDenialKind {
    LiveContractMissing,
    BindingMismatch,
    BufferCapacityExceedsInstalled,
    WorkLimitExceedsInstalled,
    Admission(WorthQueryApplicationQueryAdmissionDenialKind),
    AuthorizationDenied(WorthQueryOperationAuthorizationDenialKind),
    ScopeIdentityUnavailable,
    BasisReleaseFailed,
    ProviderVersionUnavailable,
    BridgeBasisRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveOpenDenial {
    kind: WorthQueryApplicationLiveOpenDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryApplicationLiveOpenDenial {
    pub(super) fn new(
        kind: WorthQueryApplicationLiveOpenDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            authorization_denial: None,
            subject: subject.into(),
        }
    }

    pub(super) fn with_authorization(
        kind: WorthQueryApplicationLiveOpenDenialKind,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind,
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationLiveOpenDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }
}

pub struct WorthQueryApplicationLiveUpdate<Query, QueryResult> {
    product_publication:
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    result: QueryResult,
    receipt: WorthQueryApplicationQueryAccessReceipt,
    _query: PhantomData<fn() -> Query>,
}

impl<Query, QueryResult> WorthQueryApplicationLiveUpdate<Query, QueryResult> {
    pub(super) fn new(
        product_publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        result: QueryResult,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            product_publication,
            result,
            receipt,
            _query: PhantomData,
        }
    }

    pub const fn product_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.product_publication
    }

    pub const fn result(&self) -> &QueryResult {
        &self.result
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        QueryResult,
        WorthQueryApplicationQueryAccessReceipt,
    ) {
        (self.product_publication, self.result, self.receipt)
    }

    pub fn into_admitted_disclosed(
        self,
    ) -> (
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        super::super::WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult>,
    ) {
        (
            self.product_publication,
            super::super::WorthQueryAdmittedDisclosedApplicationResult::new(
                vec![self.result],
                self.receipt,
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveCauseDenialKind {
    TargetIdentityUnavailable,
    TargetOutsideScope,
    ResultShapeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub enum WorthQueryApplicationLiveCloseOutcome {
    Completed(crate::domain_computation::provider_session::WorthQueryGraphReadCompletion),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveOverflow {
    missed_commit_count: u64,
}

impl WorthQueryApplicationLiveOverflow {
    pub(super) const fn new(missed_commit_count: u64) -> Self {
        Self {
            missed_commit_count,
        }
    }

    pub const fn missed_commit_batches(self) -> u64 {
        self.missed_commit_count
    }
}

pub enum WorthQueryApplicationLiveOutcome<Query, QueryResult> {
    Delivered(WorthQueryApplicationLiveUpdate<Query, QueryResult>),
    Pending,
    Overflow(WorthQueryApplicationLiveOverflow),
    AuthorizationDenied(Box<WorthQueryOperationAuthorizationDenial>),
    StalePrincipal,
    StaleScope,
    ProjectionDenied(WorthQueryApplicationProjectionDenialKind),
    CauseDenied(WorthQueryApplicationLiveCauseDenialKind),
    Cancelled,
    DeadlineExceeded,
    Closed,
    Unavailable,
}

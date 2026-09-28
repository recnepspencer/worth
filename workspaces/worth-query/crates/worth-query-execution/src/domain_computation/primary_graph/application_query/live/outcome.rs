use std::marker::PhantomData;

use super::super::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAccessReceipt,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// The specific reason a live read could not be opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveOpenDenialKind {
    /// The installed query declares no live contract.
    LiveContractMissing,
    /// The live cause binding does not match the installed live contract.
    BindingMismatch,
    /// The requested buffer capacity exceeds the installed ceiling.
    BufferCapacityExceedsInstalled,
    /// The requested per-delivery work limit exceeds the installed ceiling.
    WorkLimitExceedsInstalled,
    /// The initial query admission was refused.
    Admission(WorthQueryApplicationQueryAdmissionDenialKind),
    /// Authorization refused the read; the full denial is attached.
    AuthorizationDenied(WorthQueryOperationAuthorizationDenialKind),
    /// The scope's identity could not be encoded for cause matching.
    ScopeIdentityUnavailable,
    /// The opening basis could not be released cleanly.
    BasisReleaseFailed,
    /// The provider's current version could not be read.
    ProviderVersionUnavailable,
    /// The Runtime Bridge rejected the live execution basis.
    BridgeBasisRejected,
}

/// Refusal to open a live read. No subscription was opened.
///
/// [`Self::kind`] says why; for authorization causes,
/// [`Self::authorization_denial`] carries the full denial.
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

/// One live delivery: the product publication that caused it, the freshly read
/// result, and its access receipt.
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

/// Why a buffered live cause could not be turned into a delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveCauseDenialKind {
    /// The cause's target identity could not be resolved.
    TargetIdentityUnavailable,
    /// The cause's target lies outside the subscription's scope.
    TargetOutsideScope,
    /// The read produced no result in the shape the query declares.
    ResultShapeUnavailable,
}

/// What closing a live read produced.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub enum WorthQueryApplicationLiveCloseOutcome {
    /// The subscription ended cleanly with its read-completion evidence.
    Completed(crate::domain_computation::provider_session::WorthQueryGraphReadCompletion),
    /// The subscription had already ended or could not end cleanly.
    Unavailable,
}

/// Report that a live read's buffer overflowed and commit batches were missed.
///
/// The subscription ends; open a new live read and re-read current state.
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

/// The result of one `next` call on a live read.
///
/// `Delivered`, `Pending`, and `ProjectionDenied` leave the subscription open.
/// Every other outcome ends it.
pub enum WorthQueryApplicationLiveOutcome<Query, QueryResult> {
    /// A fresh result for the next committed change.
    Delivered(WorthQueryApplicationLiveUpdate<Query, QueryResult>),
    /// No change is waiting; call `next` again later.
    Pending,
    /// The buffer overflowed and commit batches were missed.
    Overflow(WorthQueryApplicationLiveOverflow),
    /// Re-authorization for this delivery was refused.
    AuthorizationDenied(Box<WorthQueryOperationAuthorizationDenial>),
    /// The fresh request's principal no longer matches the subscription.
    StalePrincipal,
    /// The fresh request's scope no longer matches the subscription.
    StaleScope,
    /// The changed row could not be projected; the cause is skipped.
    ProjectionDenied(WorthQueryApplicationProjectionDenialKind),
    /// The buffered cause could not be turned into a delivery.
    CauseDenied(WorthQueryApplicationLiveCauseDenialKind),
    /// The request was cancelled.
    Cancelled,
    /// The request reached its deadline.
    DeadlineExceeded,
    /// The subscription was already closed or its source completed.
    Closed,
    /// The runtime could not serve the delivery or end the subscription cleanly.
    Unavailable,
}

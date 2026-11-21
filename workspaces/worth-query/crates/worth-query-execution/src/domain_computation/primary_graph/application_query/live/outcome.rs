use std::marker::PhantomData;

use super::super::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial;

pub use worth_query_execution_lineage::facade::primary_graph::application_query::live::outcome::{
    WorthQueryApplicationLiveCauseDenialKind, WorthQueryApplicationLiveCloseOutcome,
    WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind,
    WorthQueryApplicationLiveOverflow,
};

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
    /// The caller's resource or execution request stopped the read.
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// The request was cancelled.
    Cancelled,
    /// The request reached its deadline.
    DeadlineExceeded,
    /// The subscription was already closed or its source completed.
    Closed,
    /// The runtime could not serve the delivery or end the subscription cleanly.
    Unavailable,
}

use super::super::super::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationQueryAccessReceipt,
    WorthQueryObservedResultSet, WorthQueryObservedSource,
};
use super::super::outcome::WorthQueryApplicationLiveOverflow;
use crate::domain_computation::primary_graph::{
    WorthQueryCommittedProductPublication, WorthQueryOperationAuthorizationDenial,
};

/// One complete whole-result snapshot captured at open or at an invalidated
/// committed batch, preserving rows, row sources, result-set observation, and
/// access receipt.
pub struct WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult> {
    rows: Vec<QueryResult>,
    observed_sources: Vec<WorthQueryObservedSource<Query>>,
    result_set_observation: WorthQueryObservedResultSet<Query>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

impl<Query, QueryResult> WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult> {
    pub(crate) fn new(
        rows: Vec<QueryResult>,
        observed_sources: Vec<WorthQueryObservedSource<Query>>,
        result_set_observation: WorthQueryObservedResultSet<Query>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources,
            result_set_observation,
            receipt,
        }
    }

    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub fn into_rows(self) -> Vec<QueryResult> {
        self.rows
    }

    pub fn observed_sources(&self) -> &[WorthQueryObservedSource<Query>] {
        &self.observed_sources
    }

    pub const fn result_set_observation(&self) -> &WorthQueryObservedResultSet<Query> {
        &self.result_set_observation
    }

    pub fn into_result_set_observation(self) -> WorthQueryObservedResultSet<Query> {
        self.result_set_observation
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<QueryResult>,
        Vec<WorthQueryObservedSource<Query>>,
        WorthQueryObservedResultSet<Query>,
        WorthQueryApplicationQueryAccessReceipt,
    ) {
        (
            self.rows,
            self.observed_sources,
            self.result_set_observation,
            self.receipt,
        )
    }
}

/// One whole-result live delivery: the committed product publication that
/// invalidated the previous observed footprint and the freshly read complete
/// snapshot at that publication.
pub struct WorthQueryApplicationWholeResultLiveUpdate<Query, QueryResult> {
    product_publication: WorthQueryCommittedProductPublication,
    snapshot: WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>,
}

impl<Query, QueryResult> WorthQueryApplicationWholeResultLiveUpdate<Query, QueryResult> {
    pub(crate) fn new(
        product_publication: WorthQueryCommittedProductPublication,
        snapshot: WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>,
    ) -> Self {
        Self {
            product_publication,
            snapshot,
        }
    }

    pub const fn product_publication(&self) -> &WorthQueryCommittedProductPublication {
        &self.product_publication
    }

    pub fn rows(&self) -> &[QueryResult] {
        self.snapshot.rows()
    }

    pub fn observed_sources(&self) -> &[WorthQueryObservedSource<Query>] {
        self.snapshot.observed_sources()
    }

    pub const fn result_set_observation(&self) -> &WorthQueryObservedResultSet<Query> {
        self.snapshot.result_set_observation()
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        self.snapshot.receipt()
    }

    pub const fn snapshot(
        &self,
    ) -> &WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult> {
        &self.snapshot
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryCommittedProductPublication,
        WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>,
    ) {
        (self.product_publication, self.snapshot)
    }
}

/// The result of one `next` call on a whole-result live subscription.
///
/// `Delivered` and `Pending` leave the subscription open. Every other outcome
/// ends it fail-closed.
pub enum WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
    /// A fresh complete result after an observed fact was invalidated.
    Delivered(WorthQueryApplicationWholeResultLiveUpdate<Query, QueryResult>),
    /// No unseen commit invalidated any observed fact; call `next` again later.
    Pending,
    /// The commit batch ring overflowed and batches were missed.
    Overflow(WorthQueryApplicationLiveOverflow),
    /// Re-authorization for this poll or delivery was refused.
    AuthorizationDenied(Box<WorthQueryOperationAuthorizationDenial>),
    /// The fresh request's principal no longer matches or is disabled/expired.
    StalePrincipal,
    /// The fresh request's scope is no longer valid or live.
    StaleScope,
    /// Projecting the refreshed rows failed.
    ProjectionDenied(WorthQueryApplicationProjectionDenialKind),
    /// Refreshing exceeded the subscription's configured `maximum_results`.
    ResultLimitExceeded,
    /// The caller's resource or execution request stopped the poll/refresh.
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// The request was cancelled.
    Cancelled,
    /// The request reached its deadline.
    DeadlineExceeded,
    /// The subscription was already closed or its delivery source completed.
    Closed,
    /// The runtime could not serve the poll/delivery or close cleanly.
    Unavailable,
}

impl<Query, QueryResult: std::fmt::Debug> std::fmt::Debug
    for WorthQueryApplicationWholeResultLiveSnapshot<Query, QueryResult>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationWholeResultLiveSnapshot")
            .field("rows", &self.rows)
            .field("observed_sources_len", &self.observed_sources.len())
            .field("receipt", &self.receipt)
            .finish()
    }
}

impl<Query, QueryResult: std::fmt::Debug> std::fmt::Debug
    for WorthQueryApplicationWholeResultLiveUpdate<Query, QueryResult>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationWholeResultLiveUpdate")
            .field("product_publication", &self.product_publication)
            .field("snapshot", &self.snapshot)
            .finish()
    }
}

impl<Query, QueryResult: std::fmt::Debug> std::fmt::Debug
    for WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Delivered(u) => f.debug_tuple("Delivered").field(u).finish(),
            Self::Pending => f.write_str("Pending"),
            Self::Overflow(o) => f.debug_tuple("Overflow").field(o).finish(),
            Self::AuthorizationDenied(d) => f.debug_tuple("AuthorizationDenied").field(d).finish(),
            Self::StalePrincipal => f.write_str("StalePrincipal"),
            Self::StaleScope => f.write_str("StaleScope"),
            Self::ProjectionDenied(k) => f.debug_tuple("ProjectionDenied").field(k).finish(),
            Self::ResultLimitExceeded => f.write_str("ResultLimitExceeded"),
            Self::ExecutionRequest(k) => f.debug_tuple("ExecutionRequest").field(k).finish(),
            Self::Cancelled => f.write_str("Cancelled"),
            Self::DeadlineExceeded => f.write_str("DeadlineExceeded"),
            Self::Closed => f.write_str("Closed"),
            Self::Unavailable => f.write_str("Unavailable"),
        }
    }
}

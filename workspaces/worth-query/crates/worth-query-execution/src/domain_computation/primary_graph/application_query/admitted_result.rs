use super::WorthQueryApplicationQueryAccessReceipt;

/// Query-admitted consumer shape. Construction is private to completed query
/// lanes, after governed projection has replaced every protected slot with a
/// typed disclosed-or-omitted value.
///
/// A consumer holding a terminal receipt still cannot construct an admitted
/// result and bypass governed projection:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedDisclosedApplicationResult,
///     WorthQueryApplicationQueryAccessReceipt,
/// };
///
/// fn counterfeit(
///     receipt: WorthQueryApplicationQueryAccessReceipt,
/// ) -> WorthQueryAdmittedDisclosedApplicationResult<(), ()> {
///     WorthQueryAdmittedDisclosedApplicationResult::new(vec![()], receipt)
/// }
/// ```
pub struct WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
    rows: Vec<QueryResult>,
    observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
    receipt: WorthQueryApplicationQueryAccessReceipt,
}

impl<Query, QueryResult> WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
    pub(super) fn new(
        rows: Vec<QueryResult>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources: Vec::new(),
            receipt,
        }
    }

    pub(super) fn new_with_sources(
        rows: Vec<QueryResult>,
        observed_sources: Vec<super::WorthQueryObservedSource<Query>>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
    ) -> Self {
        Self {
            rows,
            observed_sources,
            receipt,
        }
    }

    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    /// Consumes the governed result at a downstream publication boundary.
    /// The receipt remains execution-owned and must be projected before drop.
    pub fn into_parts(
        self,
    ) -> (
        Vec<QueryResult>,
        Vec<super::WorthQueryObservedSource<Query>>,
        WorthQueryApplicationQueryAccessReceipt,
    ) {
        (self.rows, self.observed_sources, self.receipt)
    }
}

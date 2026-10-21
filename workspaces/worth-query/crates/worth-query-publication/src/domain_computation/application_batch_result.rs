use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedDisclosedApplicationResult, WorthQueryApplicationQueryBatchResourceDenial,
};

use super::{publish_application_result, WorthQueryPublishedApplicationResult};

/// A complete ordered collection of independently disclosed scalar results.
/// The receipt describes collection work; it is not a combined scalar scope
/// receipt, observed source, or mutation authority. No partially admitted
/// collection can construct this value.
///
/// ```no_run
/// use worth_query_publication::facade::domain_computation::WorthQueryPublishedApplicationQueryBatch;
/// fn inspect(batch: &WorthQueryPublishedApplicationQueryBatch<(), ()>) {
///     for item in batch.items() { let _ = item.receipt().inspect().read_work(); }
///     let _ = batch.receipt().item_count();
/// }
/// ```
///
/// ```compile_fail,E0616
/// use worth_query_publication::facade::domain_computation::WorthQueryPublishedApplicationQueryBatch;
/// fn rewrite(mut batch: WorthQueryPublishedApplicationQueryBatch<(), ()>) {
///     batch.results = vec![];
/// }
/// ```
pub struct WorthQueryPublishedApplicationQueryBatch<Query, QueryResult> {
    results: Vec<WorthQueryPublishedApplicationResult<Query, QueryResult>>,
    receipt: WorthQueryApplicationQueryBatchPublicationReceipt,
}

/// Closed collection evidence. Individual entries retain their exact basis,
/// scope/source and disclosure receipts; this description opens no authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryBatchPublicationReceipt {
    item_count: usize,
    authorization_work_units: usize,
}

impl<Query, QueryResult> WorthQueryPublishedApplicationQueryBatch<Query, QueryResult> {
    pub fn items(&self) -> &[WorthQueryPublishedApplicationResult<Query, QueryResult>] {
        &self.results
    }
    pub const fn receipt(&self) -> &WorthQueryApplicationQueryBatchPublicationReceipt {
        &self.receipt
    }
}

impl WorthQueryApplicationQueryBatchPublicationReceipt {
    pub const fn item_count(self) -> usize {
        self.item_count
    }

    pub const fn authorization_work_units(self) -> usize {
        self.authorization_work_units
    }
}

/// Called only after every item completed ordinary authorized disclosure.
pub(crate) fn publish_batch<Query, QueryResult>(
    staged: Vec<WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult>>,
) -> Result<
    WorthQueryPublishedApplicationQueryBatch<Query, QueryResult>,
    WorthQueryApplicationQueryBatchResourceDenial,
> {
    let authorization_work_units = staged.iter().fold(0usize, |sum, item| {
        sum.saturating_add(item.receipt().authorization_work().observation_work_units())
    });
    let mut results = Vec::new();
    results
        .try_reserve_exact(staged.len())
        .map_err(|_| WorthQueryApplicationQueryBatchResourceDenial::AllocationFailed)?;
    for admitted in staged {
        results.push(publish_application_result(admitted));
    }
    let receipt = WorthQueryApplicationQueryBatchPublicationReceipt {
        item_count: results.len(),
        authorization_work_units,
    };
    Ok(WorthQueryPublishedApplicationQueryBatch { results, receipt })
}

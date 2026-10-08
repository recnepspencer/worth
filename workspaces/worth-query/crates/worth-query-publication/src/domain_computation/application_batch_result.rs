use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedDisclosedApplicationResult, WorthQueryApplicationQueryBatchAdmission,
    WorthQueryApplicationQueryBatchMemory, WorthQueryApplicationQueryBatchResourceDenial,
    WorthQueryApplicationQueryBatchWork,
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
    _claims: Vec<WorthQueryApplicationQueryBatchMemory>,
    _vector_claim: WorthQueryApplicationQueryBatchMemory,
    _publication_claim: WorthQueryApplicationQueryBatchMemory,
}

/// Closed collection evidence. Individual entries retain their exact basis,
/// scope/source and disclosure receipts; this description opens no authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryBatchPublicationReceipt {
    item_count: usize,
    read: WorthQueryApplicationQueryBatchWork,
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
    pub const fn read_work(self) -> WorthQueryApplicationQueryBatchWork {
        self.read
    }
    pub const fn authorization_work_units(self) -> usize {
        self.authorization_work_units
    }
}

/// Called only after every item has completed ordinary disclosure. All
/// Publication-owned vector and receipt-text backing is claimed before copies;
/// source charges remain attached to genuine lower custody, not descriptions.
pub(crate) fn publish_batch<Query, QueryResult>(
    staged: Vec<WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult>>,
    claims: Vec<WorthQueryApplicationQueryBatchMemory>,
    vector_claim: WorthQueryApplicationQueryBatchMemory,
    admission: &WorthQueryApplicationQueryBatchAdmission,
) -> Result<
    WorthQueryPublishedApplicationQueryBatch<Query, QueryResult>,
    WorthQueryApplicationQueryBatchResourceDenial,
> {
    let mut bytes = staged
        .len()
        .checked_mul(std::mem::size_of::<
            WorthQueryPublishedApplicationResult<Query, QueryResult>,
        >())
        .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
    let mut authorization_work_units = 0_usize;
    for item in &staged {
        let terminal = item.receipt();
        // Query and parameter digests render as hex; the basis copies its
        // actual branch text. Other closed receipt projections are inline.
        let text = terminal
            .query_identity()
            .as_bytes()
            .len()
            .checked_add(terminal.parameter_binding_identity().bytes().len())
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(terminal.basis_identity().branch_id().0.len()))
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        bytes = bytes
            .checked_add(text)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        authorization_work_units = authorization_work_units
            .checked_add(terminal.authorization_work().observation_work_units())
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
    }
    let publication_claim = admission.claim_memory(bytes)?;
    let mut results = Vec::with_capacity(staged.len());
    for admitted in staged {
        results.push(publish_application_result(admitted));
    }
    let receipt = WorthQueryApplicationQueryBatchPublicationReceipt {
        item_count: results.len(),
        read: admission.observe(),
        authorization_work_units,
    };
    Ok(WorthQueryPublishedApplicationQueryBatch {
        results,
        receipt,
        _claims: claims,
        _vector_claim: vector_claim,
        _publication_claim: publication_claim,
    })
}

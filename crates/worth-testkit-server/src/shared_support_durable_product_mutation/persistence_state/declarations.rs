pub use {std::collections::HashMap, std::collections::HashSet, std::sync::atomic::AtomicU64, std::sync::atomic::AtomicUsize, std::sync::Arc, std::sync::Mutex};

pub use {worth_server::WorthServerAdmittedDurableProductMutation, worth_server::WorthServerDurableProductMutationCompletion, worth_server::WorthServerProductIdempotencyRetention};

pub use {crate::shared_support_durable_product_mutation::concurrency_probe::TestConcurrencyProbe};

#[derive(Clone)]
pub struct DurableRecord {
    pub request_digest: String,
    pub completion: WorthServerDurableProductMutationCompletion,
    pub retain_until: Option<u64>,
}

#[derive(Default)]
pub struct DurableScopeState {
    pub current_version: u64,
    pub records: HashMap<String, DurableRecord>,
    pub consumed_crashes: HashSet<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DurableScopeIdentity {
    pub tenant_id: String,
    pub workspace_id: String,
    pub authority_scope: String,
}

#[derive(Default)]
pub struct DurableProductState {
    pub scopes: Mutex<HashMap<DurableScopeIdentity, Arc<Mutex<DurableScopeState>>>>,
    pub recoveries: Mutex<HashMap<String, DurableRecord>>,
    pub commit_count: AtomicUsize,
    pub now_seconds: AtomicU64,
    pub concurrency_probe: Mutex<Option<TestConcurrencyProbe>>,
    pub recovery_override: Mutex<Option<WorthServerDurableProductMutationCompletion>>,
    pub observed_attempts: Mutex<Vec<(String, String)>>,
}

pub fn retention_deadline(
    attempt: &WorthServerAdmittedDurableProductMutation,
    now: u64,
) -> Option<u64> {
    match attempt.durable_contract().idempotency_retention() {
        worth_server::WorthServerProductIdempotencyRetention::AtLeastSeconds(seconds) => {
            Some(now.saturating_add(*seconds))
        }
        worth_server::WorthServerProductIdempotencyRetention::Indefinite => None,
    }
}

pub fn retained_record<'a>(
    records: &'a mut HashMap<String, DurableRecord>,
    key: &str,
    now: u64,
) -> Option<&'a DurableRecord> {
    let expired = records
        .get(key)
        .and_then(|record| record.retain_until)
        .is_some_and(|deadline| now >= deadline);
    if expired {
        records.remove(key);
        return None;
    }
    records.get(key)
}
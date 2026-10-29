use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, AtomicUsize},
        Arc, Mutex,
    },
};

use worth_server::{
    WorthServerAdmittedDurableProductMutation, WorthServerDurableProductMutationCompletion,
    WorthServerProductIdempotencyRetention,
};

use super::TestConcurrencyProbe;

#[derive(Clone)]
pub(super) struct DurableRecord {
    pub(super) request_digest: String,
    pub(super) completion: WorthServerDurableProductMutationCompletion,
    pub(super) retain_until: Option<u64>,
}

#[derive(Default)]
pub(super) struct DurableScopeState {
    pub(super) current_version: u64,
    pub(super) records: HashMap<String, DurableRecord>,
    pub(super) consumed_crashes: HashSet<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct DurableScopeIdentity {
    pub(super) tenant_id: String,
    pub(super) workspace_id: String,
    pub(super) authority_scope: String,
}

#[derive(Default)]
pub(super) struct DurableProductState {
    pub(super) scopes: Mutex<HashMap<DurableScopeIdentity, Arc<Mutex<DurableScopeState>>>>,
    pub(super) recoveries: Mutex<HashMap<String, DurableRecord>>,
    pub(super) commit_count: AtomicUsize,
    pub(super) now_seconds: AtomicU64,
    pub(super) concurrency_probe: Mutex<Option<TestConcurrencyProbe>>,
    pub(super) recovery_override: Mutex<Option<WorthServerDurableProductMutationCompletion>>,
    pub(super) observed_attempts: Mutex<Vec<(String, String)>>,
}

pub(super) fn retention_deadline(
    attempt: &WorthServerAdmittedDurableProductMutation,
    now: u64,
) -> Option<u64> {
    match attempt.durable_contract().idempotency_retention() {
        WorthServerProductIdempotencyRetention::AtLeastSeconds(seconds) => {
            Some(now.saturating_add(*seconds))
        }
        WorthServerProductIdempotencyRetention::Indefinite => None,
    }
}

pub(super) fn retained_record<'a>(
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

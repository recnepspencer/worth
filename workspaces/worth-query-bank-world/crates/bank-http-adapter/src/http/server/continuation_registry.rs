//! Finite shared transport inventory; concrete purposes retain their native owners.
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::OsRng,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
mod activity;
pub(super) use activity::ResumeAdmission;
pub(super) mod correction_history;
use activity::{ActivityRecord, InitialRequestIdentity};
use correction_history::CorrectionHistoryRecord;

const TOKEN_PREFIX: &str = "bank-continuation-v1_";
#[cfg(test)]
mod tests;
pub(super) type SharedContinuationRegistry = Arc<Mutex<BankHttpContinuationRegistry>>;

pub(super) struct BankHttpContinuationRegistry {
    records: HashMap<String, ContinuationRecord>,
    initial_requests: HashMap<InitialRequestIdentity, String>,
    history_requests: HashMap<
        (
            super::authenticated_owner::BankHttpAuthenticatedOwner,
            String,
        ),
        String,
    >,
    capacity: usize,
    lifetime: Duration,
    retired: Vec<ContinuationRecord>,
    retired_activity: Vec<bank_server::BankAccountActivityContinuation>,
    retired_pages:
        Vec<Arc<worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage>>,
}
enum ContinuationRecord {
    Activity(ActivityRecord),
    Correction(CorrectionHistoryRecord),
}
impl ContinuationRecord {
    fn expires_at(&self) -> Instant {
        match self {
            Self::Activity(record) => record.expires_at(),
            Self::Correction(record) => record.expires_at,
        }
    }
}
/// Native custody removed under the inventory lock is dropped only after unlocking.
pub(super) async fn access<R>(
    shared: &SharedContinuationRegistry,
    operation: impl FnOnce(&mut BankHttpContinuationRegistry) -> R,
) -> R {
    let (result, retired, activity, pages) = {
        let mut registry = shared.lock().await;
        let result = operation(&mut registry);
        (
            result,
            std::mem::take(&mut registry.retired),
            std::mem::take(&mut registry.retired_activity),
            std::mem::take(&mut registry.retired_pages),
        )
    };
    drop(retired);
    drop(activity);
    drop(pages);
    result
}
impl BankHttpContinuationRegistry {
    pub(super) fn recognizes_token(token: &str) -> bool {
        token.strip_prefix(TOKEN_PREFIX).is_some_and(|random| {
            random.len() == 40 && random.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
    }
    pub(super) fn new(capacity: usize, lifetime: Duration) -> Self {
        Self {
            records: HashMap::with_capacity(capacity),
            initial_requests: HashMap::with_capacity(capacity),
            history_requests: HashMap::with_capacity(capacity),
            capacity,
            lifetime,
            retired: Vec::new(),
            retired_activity: Vec::new(),
            retired_pages: Vec::new(),
        }
    }
    fn new_token(&self) -> String {
        loop {
            let token = format!(
                "{TOKEN_PREFIX}{}",
                Alphanumeric.sample_string(&mut OsRng, 40)
            );
            if !self.records.contains_key(&token) {
                return token;
            }
        }
    }
    fn purge_expired(&mut self) {
        let now = Instant::now();
        let expired: Vec<_> = self
            .records
            .iter()
            .filter(|(_, record)| record.expires_at() <= now)
            .map(|(token, _)| token.clone())
            .collect();
        for token in expired {
            if let Some(record) = self.records.remove(&token) {
                self.retired.push(record);
            }
        }
        self.initial_requests
            .retain(|_, token| self.records.contains_key(token));
        self.history_requests
            .retain(|_, token| self.records.contains_key(token));
    }
}

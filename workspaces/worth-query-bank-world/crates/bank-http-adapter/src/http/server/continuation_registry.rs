//! Bounded transport inventory for native account-activity continuations.
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
use activity::{ActivityRecord, InitialRequestIdentity};

const TOKEN_PREFIX: &str = "bank-continuation-v1_";
#[cfg(test)]
mod tests;
pub(super) type SharedContinuationRegistry = Arc<Mutex<BankHttpContinuationRegistry>>;

pub(super) struct BankHttpContinuationRegistry {
    records: HashMap<String, ActivityRecord>,
    initial_requests: HashMap<InitialRequestIdentity, String>,
    capacity: usize,
    lifetime: Duration,
    retired: Vec<ActivityRecord>,
    retired_activity: Vec<bank_server::BankAccountActivityContinuation>,
}
/// Native custody removed under the inventory lock is dropped only after unlocking.
pub(super) async fn access<R>(
    shared: &SharedContinuationRegistry,
    operation: impl FnOnce(&mut BankHttpContinuationRegistry) -> R,
) -> R {
    let (result, retired, activity) = {
        let mut registry = shared.lock().await;
        let result = operation(&mut registry);
        (
            result,
            std::mem::take(&mut registry.retired),
            std::mem::take(&mut registry.retired_activity),
        )
    };
    drop(retired);
    drop(activity);
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
            capacity,
            lifetime,
            retired: Vec::new(),
            retired_activity: Vec::new(),
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
    }
}

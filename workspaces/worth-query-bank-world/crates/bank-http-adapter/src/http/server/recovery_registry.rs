use std::collections::HashMap;
use std::time::{Duration, Instant};

use bank_domain::estate::EstateAction;
use bank_server::{
    BankCommitRecoveryHandle, BankIdentityRuntime, BankRecoveryExpiryEvaluation,
    BankRecoverySafeRetryDenial, BankRecoverySafeRetryReceipt,
};
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;

use super::authenticated_owner::BankHttpAuthenticatedOwner;

mod commit;
mod state;

pub(super) use state::{BankHttpCommitReplay, BankHttpRecoveryRegistration, BankHttpRecoveryRetry};
use state::{CommitReplayKey, RecoveryRecord, RecoveryRetryResult};

const TOKEN_PREFIX: &str = "bank-recovery-v1_";

pub(super) struct BankHttpRecoveryRegistry {
    records: HashMap<String, RecoveryRecord>,
    replay_tokens: HashMap<CommitReplayKey, String>,
    capacity: usize,
    lifetime: Duration,
}

pub(super) struct BankHttpRecoveryInspection<'registry> {
    handle: &'registry BankCommitRecoveryHandle,
    action: EstateAction,
}

impl BankHttpRecoveryInspection<'_> {
    pub(super) const fn handle(&self) -> &BankCommitRecoveryHandle {
        self.handle
    }

    pub(super) const fn action(&self) -> EstateAction {
        self.action
    }
}

impl BankHttpRecoveryRegistry {
    pub(super) fn recognizes_token(token: &str) -> bool {
        token.strip_prefix(TOKEN_PREFIX).is_some_and(|random| {
            random.len() == 40 && random.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
    }

    pub(super) fn new(capacity: usize, lifetime: Duration) -> Self {
        Self {
            records: HashMap::with_capacity(capacity),
            replay_tokens: HashMap::with_capacity(capacity),
            capacity,
            lifetime,
        }
    }

    pub(super) fn recovery(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
    ) -> Option<BankHttpRecoveryInspection<'_>> {
        let record = self.records.get(token)?;
        (&record.owner == owner).then_some(())?;
        Some(BankHttpRecoveryInspection {
            handle: record.handle.as_ref()?,
            action: record.action,
        })
    }

    pub(super) fn retry(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
        attempt: impl FnOnce(
            BankCommitRecoveryHandle,
            EstateAction,
        )
            -> Result<BankRecoverySafeRetryReceipt, BankRecoverySafeRetryDenial>,
    ) -> BankHttpRecoveryRetry {
        let Some(record) = self
            .records
            .get_mut(token)
            .filter(|record| &record.owner == owner)
        else {
            return BankHttpRecoveryRetry::Missing;
        };
        if let Some(retried) = record.retried {
            return BankHttpRecoveryRetry::Applied {
                result: retried,
                replay: true,
            };
        }
        let Some(handle) = record.handle.take() else {
            return BankHttpRecoveryRetry::Missing;
        };
        match attempt(handle, record.action) {
            Ok(receipt) => {
                let retried = RecoveryRetryResult {
                    external_completion: receipt.is_external_completion(),
                    fresh_attempt: receipt.has_fresh_attempt(),
                };
                record.retried = Some(retried);
                BankHttpRecoveryRetry::Applied {
                    result: retried,
                    replay: false,
                }
            }
            Err(denied) => {
                let (denial, handle) = denied.into_parts();
                record.handle = handle;
                BankHttpRecoveryRetry::Denied(denial)
            }
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

    pub(super) fn purge_expired(&mut self, runtime: &BankIdentityRuntime) {
        let now = Instant::now();
        self.records.retain(|_, record| {
            if record.expires_at > now {
                return true;
            }
            let Some(handle) = record.handle.as_ref() else {
                return true;
            };
            match runtime.evaluate_commit_recovery_expiry(handle) {
                Ok(BankRecoveryExpiryEvaluation::Expired(decision)) => {
                    let handle = record
                        .handle
                        .take()
                        .expect("evaluated live recovery handle");
                    let _ = runtime.expire_commit_recovery(handle, decision);
                    false
                }
                Ok(BankRecoveryExpiryEvaluation::Current) | Err(_) => true,
            }
        });
        self.replay_tokens
            .retain(|_, token| self.records.contains_key(token));
    }
}

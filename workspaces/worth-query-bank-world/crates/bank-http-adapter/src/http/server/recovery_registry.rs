use std::collections::HashMap;
use std::time::{Duration, Instant};

use bank_domain::estate::EstateAction;
use bank_server::BankCommitRecoveryHandle;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;

use super::authenticated_owner::BankHttpAuthenticatedOwner;

mod commit;
mod state;

pub(super) use state::{BankHttpCommitReplay, BankHttpRecoveryRegistration};
use state::{CommitReplayKey, RecoveryRecord};

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
        self.purge_expired();
        let record = self.records.get(token)?;
        (&record.owner == owner).then_some(())?;
        Some(BankHttpRecoveryInspection {
            handle: &record.handle,
            action: record.action,
        })
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
        self.records.retain(|_, record| record.expires_at > now);
        self.replay_tokens
            .retain(|_, token| self.records.contains_key(token));
    }
}

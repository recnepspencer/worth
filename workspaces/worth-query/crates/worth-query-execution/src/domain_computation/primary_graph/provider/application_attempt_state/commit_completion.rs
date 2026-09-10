//! Exact completion custody for a provider application attempt in commit.

use super::{
    WorthQueryApplicationAttemptEntry, WorthQueryApplicationAttemptLookupKey,
    WorthQueryPrimaryGraphApplicationAttemptStore,
};
use crate::domain_computation::primary_graph::provider::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters, WorthQueryPrimaryGraphApplicationAttempt,
};
use std::sync::{Arc, Mutex};

pub(super) struct WorthQueryPreparedProviderApplicationAttempt {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
    work: WorthQueryPrimaryMutationWorkCounters,
    completion: WorthQueryApplicationAttemptCompletion,
}

pub(super) struct WorthQueryApplicationAttemptCompletion {
    attempts: Arc<Mutex<WorthQueryPrimaryGraphApplicationAttemptStore>>,
    key: WorthQueryApplicationAttemptLookupKey,
    identity: u64,
}

impl WorthQueryPreparedProviderApplicationAttempt {
    pub(super) fn new(
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: WorthQueryPrimaryMutationWorkCounters,
        attempts: Arc<Mutex<WorthQueryPrimaryGraphApplicationAttemptStore>>,
        key: WorthQueryApplicationAttemptLookupKey,
        identity: u64,
    ) -> Self {
        Self {
            attempt,
            candidate,
            work,
            completion: WorthQueryApplicationAttemptCompletion {
                attempts,
                key,
                identity,
            },
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryPrimaryGraphApplicationAttempt,
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
        WorthQueryPrimaryMutationWorkCounters,
        WorthQueryApplicationAttemptCompletion,
    ) {
        (self.attempt, self.candidate, self.work, self.completion)
    }
}

impl Drop for WorthQueryApplicationAttemptCompletion {
    fn drop(&mut self) {
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owns_marker = attempts.attempts.get(&self.key).is_some_and(|entry| {
            matches!(
                entry,
                WorthQueryApplicationAttemptEntry::Committing {
                    identity: committing_identity
                } if *committing_identity == self.identity
            )
        });
        if owns_marker {
            attempts.attempts.remove(&self.key);
        }
    }
}

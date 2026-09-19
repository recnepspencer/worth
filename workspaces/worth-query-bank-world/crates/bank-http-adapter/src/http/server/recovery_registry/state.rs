use std::time::{Duration, Instant};

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_server::{BankCommitRecoveryHandle, BankEstateProgressionDenial};

use super::super::super::protocol::BankHttpCommitDescription;
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum RecoveryOrigin {
    Notification,
    Disbursement,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub(super) struct CommitReplayKey {
    pub(super) owner: BankHttpAuthenticatedOwner,
    pub(super) origin: RecoveryOrigin,
    pub(super) idempotency_key: BankIdempotencyKey,
}

pub(super) struct RecoveryRecord {
    pub(super) owner: BankHttpAuthenticatedOwner,
    pub(super) action: EstateAction,
    pub(super) commit: BankHttpCommitDescription,
    pub(super) expires_at: Instant,
    pub(super) handle: Option<BankCommitRecoveryHandle>,
    pub(super) retried: Option<RecoveryRetryResult>,
}

impl RecoveryRecord {
    pub(super) fn new(
        replay: CommitReplayKey,
        action: EstateAction,
        registration: BankHttpRecoveryRegistration,
        lifetime: Duration,
    ) -> Self {
        Self {
            owner: replay.owner,
            action,
            commit: registration.commit,
            expires_at: Instant::now() + lifetime,
            handle: Some(registration.handle),
            retried: None,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::http::server) struct RecoveryRetryResult {
    pub(in crate::http::server) external_completion: bool,
    pub(in crate::http::server) fresh_attempt: bool,
}

pub(in crate::http::server) enum BankHttpRecoveryRetry {
    Missing,
    Applied {
        result: RecoveryRetryResult,
        replay: bool,
    },
    Denied(BankEstateProgressionDenial),
}

pub(in crate::http::server) enum BankHttpCommitReplay {
    Missing,
    Applied {
        commit: BankHttpCommitDescription,
        recovery: String,
        completed: bool,
    },
    Conflicting,
}

pub(in crate::http::server) struct BankHttpRecoveryRegistration {
    pub(in crate::http::server) commit: BankHttpCommitDescription,
    pub(in crate::http::server) handle: BankCommitRecoveryHandle,
}

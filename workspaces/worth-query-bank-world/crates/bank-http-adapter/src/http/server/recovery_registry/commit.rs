use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;

use super::state::{
    BankHttpCommitReplay, BankHttpRecoveryRegistration, CommitReplayKey, RecoveryOrigin,
    RecoveryRecord,
};
use super::BankHttpRecoveryRegistry;
use crate::http::server::authenticated_owner::BankHttpAuthenticatedOwner;

pub(in crate::http::server) struct BankHttpRecoveryReservation<'registry> {
    registry: &'registry mut BankHttpRecoveryRegistry,
    token: String,
    replay: CommitReplayKey,
    action: EstateAction,
}

impl BankHttpRecoveryReservation<'_> {
    pub(in crate::http::server) fn register(
        self,
        registration: BankHttpRecoveryRegistration,
    ) -> String {
        let Self {
            registry,
            token,
            replay,
            action,
        } = self;
        registry.register_commit(token, replay, action, registration)
    }
}

impl BankHttpRecoveryRegistry {
    pub(in crate::http::server) fn notification_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        key: &BankIdempotencyKey,
        action: EstateAction,
    ) -> BankHttpCommitReplay {
        self.commit_replay(owner, key, RecoveryOrigin::Notification, action)
    }

    pub(in crate::http::server) fn disbursement_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        key: &BankIdempotencyKey,
        action: EstateAction,
    ) -> BankHttpCommitReplay {
        self.commit_replay(owner, key, RecoveryOrigin::Disbursement, action)
    }

    pub(in crate::http::server) fn reserve_notification(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        action: EstateAction,
    ) -> Option<BankHttpRecoveryReservation<'_>> {
        self.reserve_commit(owner, key, RecoveryOrigin::Notification, action)
    }

    pub(in crate::http::server) fn reserve_disbursement(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        action: EstateAction,
    ) -> Option<BankHttpRecoveryReservation<'_>> {
        self.reserve_commit(owner, key, RecoveryOrigin::Disbursement, action)
    }

    fn commit_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        key: &BankIdempotencyKey,
        origin: RecoveryOrigin,
        action: EstateAction,
    ) -> BankHttpCommitReplay {
        let replay = CommitReplayKey {
            owner: owner.clone(),
            origin,
            idempotency_key: key.clone(),
        };
        let Some(token) = self.replay_tokens.get(&replay) else {
            return BankHttpCommitReplay::Missing;
        };
        let Some(record) = self.records.get(token) else {
            return BankHttpCommitReplay::Missing;
        };
        if record.action != action {
            return BankHttpCommitReplay::Conflicting;
        }
        BankHttpCommitReplay::Applied {
            commit: record.commit,
            recovery: token.clone(),
            completed: record.retried.is_some(),
        }
    }

    fn reserve_commit(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        idempotency_key: BankIdempotencyKey,
        origin: RecoveryOrigin,
        action: EstateAction,
    ) -> Option<BankHttpRecoveryReservation<'_>> {
        if self.records.len() >= self.capacity {
            self.evict_terminal_replay();
        }
        if self.records.len() >= self.capacity {
            return None;
        }
        let token = self.new_token();
        Some(BankHttpRecoveryReservation {
            registry: self,
            token,
            replay: CommitReplayKey {
                owner,
                origin,
                idempotency_key,
            },
            action,
        })
    }

    fn evict_terminal_replay(&mut self) {
        let oldest = self
            .records
            .iter()
            .filter(|(_, record)| record.handle.is_none())
            .min_by(|(left_token, left), (right_token, right)| {
                left.expires_at
                    .cmp(&right.expires_at)
                    .then_with(|| left_token.cmp(right_token))
            })
            .map(|(token, _)| token.clone());
        if let Some(token) = oldest {
            self.records.remove(&token);
            self.replay_tokens.retain(|_, indexed| indexed != &token);
        }
    }

    fn register_commit(
        &mut self,
        token: String,
        replay: CommitReplayKey,
        action: EstateAction,
        registration: BankHttpRecoveryRegistration,
    ) -> String {
        self.records.insert(
            token.clone(),
            RecoveryRecord::new(replay.clone(), action, registration, self.lifetime),
        );
        self.replay_tokens.insert(replay, token.clone());
        token
    }
}

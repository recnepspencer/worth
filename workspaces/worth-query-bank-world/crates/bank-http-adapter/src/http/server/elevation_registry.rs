use std::collections::HashMap;
use std::time::Duration;

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_server::{
    BankApprovedEstateElevation, BankEstateElevationClosureKind, BankEstateMandatoryReview,
    BankRequestedEstateElevation,
};
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;

use super::authenticated_owner::BankHttpAuthenticatedOwner;

mod state;

pub(super) use state::BankHttpElevationContext;
use state::{BankHttpElevationRecord, BankHttpElevationState, BankHttpElevationTransitionReplay};

const TOKEN_PREFIX: &str = "bank-elevation-v1_";

#[derive(Clone, Copy)]
pub(super) struct BankHttpElevationFacts {
    pub(super) changed_record_count: usize,
    pub(super) emitted_effect_count: usize,
}

#[derive(Clone, Copy)]
pub(super) struct BankHttpElevationReviewFacts {
    pub(super) closure: BankEstateElevationClosureKind,
    pub(super) changed_record_count: usize,
}

pub(super) enum BankHttpElevationReplay<Facts> {
    Applied(Facts),
    Conflicting,
    Missing,
}

pub(super) struct BankHttpRequestedElevation {
    pub(super) context: BankHttpElevationContext,
    pub(super) authority: BankRequestedEstateElevation,
}

pub(super) struct BankHttpApprovedElevation {
    pub(super) context: BankHttpElevationContext,
    pub(super) authority: BankApprovedEstateElevation,
}

pub(super) struct BankHttpMandatoryElevationReview {
    pub(super) context: BankHttpElevationContext,
    pub(super) authority: BankEstateMandatoryReview,
}

pub(super) struct BankHttpElevationRegistry {
    records: HashMap<String, BankHttpElevationRecord>,
    request_tokens: HashMap<(BankHttpAuthenticatedOwner, BankIdempotencyKey), String>,
    capacity: usize,
    lifetime: Duration,
}

pub(super) struct BankHttpElevationRequestReservation {
    _private: (),
}

impl BankHttpElevationRegistry {
    pub(super) fn recognizes_token(token: &str) -> bool {
        token.strip_prefix(TOKEN_PREFIX).is_some_and(|random| {
            random.len() == 40 && random.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
    }

    pub(super) fn new(capacity: usize, lifetime: Duration) -> Self {
        Self {
            records: HashMap::with_capacity(capacity),
            request_tokens: HashMap::with_capacity(capacity),
            capacity,
            lifetime,
        }
    }

    pub(super) fn request_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        key: &BankIdempotencyKey,
        action: EstateAction,
    ) -> BankHttpElevationReplay<(String, BankHttpElevationFacts)> {
        self.purge_expired();
        let Some(token) = self.request_tokens.get(&(owner.clone(), key.clone())) else {
            return BankHttpElevationReplay::Missing;
        };
        let Some(record) = self.records.get(token) else {
            return BankHttpElevationReplay::Missing;
        };
        if record.request_action != action {
            return BankHttpElevationReplay::Conflicting;
        }
        BankHttpElevationReplay::Applied((token.clone(), record.request_facts))
    }

    pub(super) fn register_requested(
        &mut self,
        _reservation: BankHttpElevationRequestReservation,
        owner: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        action: EstateAction,
        context: BankHttpElevationContext,
        authority: BankRequestedEstateElevation,
        facts: BankHttpElevationFacts,
    ) -> String {
        let token = self.new_token();
        self.request_tokens
            .insert((owner.clone(), key.clone()), token.clone());
        self.records.insert(
            token.clone(),
            BankHttpElevationRecord {
                request_action: action,
                request_facts: facts,
                context,
                state: BankHttpElevationState::Requested(authority),
                approval: None,
                close: None,
                review: None,
                expires_at: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + self.lifetime,
            },
        );
        token
    }

    pub(super) fn reserve_request(&mut self) -> Option<BankHttpElevationRequestReservation> {
        self.purge_expired();
        (self.records.len() < self.capacity)
            .then_some(BankHttpElevationRequestReservation { _private: () })
    }

    pub(super) fn approval_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
        key: &BankIdempotencyKey,
    ) -> BankHttpElevationReplay<BankHttpElevationFacts> {
        self.transition_replay(token, owner, key, |record| record.approval.as_ref())
    }

    pub(super) fn take_requested(&mut self, token: &str) -> Option<BankHttpRequestedElevation> {
        let record = self.record_mut(token)?;
        match std::mem::replace(&mut record.state, BankHttpElevationState::Terminal) {
            BankHttpElevationState::Requested(authority) => Some(BankHttpRequestedElevation {
                context: record.context,
                authority,
            }),
            state => {
                record.state = state;
                None
            }
        }
    }

    pub(super) fn restore_requested(
        &mut self,
        token: &str,
        authority: BankRequestedEstateElevation,
    ) {
        self.install_state(token, BankHttpElevationState::Requested(authority));
    }

    pub(super) fn install_approved(
        &mut self,
        token: &str,
        actor: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        authority: BankApprovedEstateElevation,
        facts: BankHttpElevationFacts,
    ) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = BankHttpElevationState::Approved(authority);
            record.approval = Some(BankHttpElevationTransitionReplay { actor, key, facts });
            record.expires_at = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + self.lifetime;
        }
    }

    pub(super) fn close_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
        key: &BankIdempotencyKey,
    ) -> BankHttpElevationReplay<BankHttpElevationReviewFacts> {
        self.transition_replay(token, owner, key, |record| record.close.as_ref())
    }

    pub(super) fn take_approved(&mut self, token: &str) -> Option<BankHttpApprovedElevation> {
        let record = self.record_mut(token)?;
        match std::mem::replace(&mut record.state, BankHttpElevationState::Terminal) {
            BankHttpElevationState::Approved(authority) => Some(BankHttpApprovedElevation {
                context: record.context,
                authority,
            }),
            state => {
                record.state = state;
                None
            }
        }
    }

    pub(super) fn restore_approved(&mut self, token: &str, authority: BankApprovedEstateElevation) {
        self.install_state(token, BankHttpElevationState::Approved(authority));
    }

    pub(super) fn install_mandatory_review(
        &mut self,
        token: &str,
        actor: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        authority: BankEstateMandatoryReview,
        facts: BankHttpElevationReviewFacts,
    ) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = BankHttpElevationState::MandatoryReview(authority);
            record.close = Some(BankHttpElevationTransitionReplay { actor, key, facts });
            record.expires_at = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + self.lifetime;
        }
    }

    pub(super) fn review_replay(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
        key: &BankIdempotencyKey,
    ) -> BankHttpElevationReplay<BankHttpElevationReviewFacts> {
        self.transition_replay(token, owner, key, |record| record.review.as_ref())
    }

    pub(super) fn take_mandatory_review(
        &mut self,
        token: &str,
    ) -> Option<BankHttpMandatoryElevationReview> {
        let record = self.record_mut(token)?;
        match std::mem::replace(&mut record.state, BankHttpElevationState::Terminal) {
            BankHttpElevationState::MandatoryReview(authority) => {
                Some(BankHttpMandatoryElevationReview {
                    context: record.context,
                    authority,
                })
            }
            state => {
                record.state = state;
                None
            }
        }
    }

    pub(super) fn restore_mandatory_review(
        &mut self,
        token: &str,
        authority: BankEstateMandatoryReview,
    ) {
        self.install_state(token, BankHttpElevationState::MandatoryReview(authority));
    }

    pub(super) fn install_reviewed(
        &mut self,
        token: &str,
        actor: BankHttpAuthenticatedOwner,
        key: BankIdempotencyKey,
        facts: BankHttpElevationReviewFacts,
    ) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = BankHttpElevationState::Terminal;
            record.review = Some(BankHttpElevationTransitionReplay { actor, key, facts });
            record.expires_at = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + self.lifetime;
        }
    }

    fn transition_replay<Facts: Copy>(
        &mut self,
        token: &str,
        owner: &BankHttpAuthenticatedOwner,
        key: &BankIdempotencyKey,
        select: impl FnOnce(
            &BankHttpElevationRecord,
        ) -> Option<&BankHttpElevationTransitionReplay<Facts>>,
    ) -> BankHttpElevationReplay<Facts> {
        let Some(record) = self.record_mut(token) else {
            return BankHttpElevationReplay::Missing;
        };
        let Some(replay) = select(record) else {
            return BankHttpElevationReplay::Missing;
        };
        if &replay.actor != owner {
            return BankHttpElevationReplay::Missing;
        }
        if &replay.key != key {
            return BankHttpElevationReplay::Conflicting;
        }
        BankHttpElevationReplay::Applied(replay.facts)
    }

    fn record_mut(&mut self, token: &str) -> Option<&mut BankHttpElevationRecord> {
        self.purge_expired();
        self.records.get_mut(token)
    }

    fn install_state(&mut self, token: &str, state: BankHttpElevationState) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = state;
            record.expires_at = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + self.lifetime;
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
        let now = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now();
        self.records.retain(|_, record| record.expires_at > now);
        self.request_tokens
            .retain(|_, token| self.records.contains_key(token));
    }
}

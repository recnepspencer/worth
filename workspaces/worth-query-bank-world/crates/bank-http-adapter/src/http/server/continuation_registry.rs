use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::authenticated_owner::BankHttpAuthenticatedOwner;
use bank_domain::model::AccountId;
use bank_server::BankAccountActivityContinuation;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;

use super::super::protocol::{BankHttpAccountActivityPageOutcome, BankHttpDenial};

const TOKEN_PREFIX: &str = "bank-continuation-v1_";

pub(super) struct BankHttpContinuationRegistry {
    records: HashMap<String, ContinuationRecord>,
    initial_requests: HashMap<InitialRequestIdentity, String>,
    capacity: usize,
    lifetime: Duration,
}

struct ContinuationRecord {
    owner: BankHttpAuthenticatedOwner,
    account: AccountId,
    expires_at: Instant,
    state: ContinuationState,
    initial_request_id: String,
    initial_outcome: BankHttpAccountActivityPageOutcome,
    last_request_id: String,
    last_outcome: BankHttpAccountActivityPageOutcome,
}

enum ContinuationState {
    Ready(BankAccountActivityContinuation),
    InFlight,
    Terminal,
}

#[derive(Eq, Hash, PartialEq)]
struct InitialRequestIdentity {
    owner: BankHttpAuthenticatedOwner,
    account: AccountId,
    request_id: String,
}

pub(super) enum ResumeAdmission {
    Execute(BankAccountActivityContinuation),
    Replay(BankHttpAccountActivityPageOutcome),
    InFlight,
    Unavailable,
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
        }
    }

    pub(super) fn replay_initial(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        account: AccountId,
        request_id: &str,
    ) -> Option<(String, BankHttpAccountActivityPageOutcome)> {
        self.purge_expired();
        let key = InitialRequestIdentity {
            owner: owner.clone(),
            account,
            request_id: request_id.to_owned(),
        };
        let token = self.initial_requests.get(&key)?;
        self.records
            .get(token)
            .map(|record| (token.clone(), record.initial_outcome.clone()))
    }

    pub(super) fn register_initial(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        account: AccountId,
        request_id: String,
        continuation: Option<BankAccountActivityContinuation>,
        activity: super::super::protocol::BankHttpAccountActivity,
        publication: super::super::protocol::BankHttpQueryPublication,
    ) -> Result<BankHttpAccountActivityPageOutcome, ()> {
        self.purge_expired();
        if self.records.len() >= self.capacity {
            return Err(());
        }
        let token = self.new_token();
        let exposed = continuation.as_ref().map(|_| token.clone());
        let outcome = delivered(request_id.clone(), activity, exposed, publication);
        let state = continuation.map_or(ContinuationState::Terminal, ContinuationState::Ready);
        self.records.insert(
            token.clone(),
            ContinuationRecord {
                owner: owner.clone(),
                account,
                expires_at: Instant::now() + self.lifetime,
                state,
                initial_request_id: request_id.clone(),
                initial_outcome: outcome.clone(),
                last_request_id: request_id.clone(),
                last_outcome: outcome.clone(),
            },
        );
        self.initial_requests.insert(
            InitialRequestIdentity {
                owner,
                account,
                request_id,
            },
            token,
        );
        Ok(outcome)
    }

    pub(super) fn begin_resume(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        account: AccountId,
        request_id: &str,
        token: &str,
    ) -> ResumeAdmission {
        self.purge_expired();
        let Some(record) = self.records.get_mut(token) else {
            return ResumeAdmission::Unavailable;
        };
        if &record.owner != owner || record.account != account {
            return ResumeAdmission::Unavailable;
        }
        if record.last_request_id == request_id {
            return ResumeAdmission::Replay(record.last_outcome.clone());
        }
        match std::mem::replace(&mut record.state, ContinuationState::InFlight) {
            ContinuationState::Ready(continuation) => ResumeAdmission::Execute(continuation),
            ContinuationState::InFlight => ResumeAdmission::InFlight,
            ContinuationState::Terminal => {
                record.state = ContinuationState::Terminal;
                ResumeAdmission::Unavailable
            }
        }
    }

    pub(super) fn complete_resume(
        &mut self,
        token: &str,
        request_id: String,
        activity: super::super::protocol::BankHttpAccountActivity,
        continuation: Option<BankAccountActivityContinuation>,
        publication: super::super::protocol::BankHttpQueryPublication,
    ) -> BankHttpAccountActivityPageOutcome {
        let exposed = continuation.as_ref().map(|_| token.to_owned());
        let outcome = delivered(request_id.clone(), activity, exposed, publication);
        if let Some(record) = self.records.get_mut(token) {
            record.state =
                continuation.map_or(ContinuationState::Terminal, ContinuationState::Ready);
            record.last_request_id = request_id;
            record.last_outcome = outcome.clone();
            record.expires_at = Instant::now() + self.lifetime;
        }
        outcome
    }

    pub(super) fn fail_resume(
        &mut self,
        token: &str,
        request_id: String,
        outcome: BankHttpAccountActivityPageOutcome,
        scrub_initial: bool,
    ) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = ContinuationState::Terminal;
            record.last_request_id = request_id;
            if let BankHttpAccountActivityPageOutcome::Denied { denial, .. } = &outcome {
                if scrub_initial {
                    record.initial_outcome = denied(record.initial_request_id.clone(), *denial);
                }
            }
            record.last_outcome = outcome;
            record.expires_at = Instant::now() + self.lifetime;
        }
    }

    pub(super) fn deny_protected_replay(&mut self, token: &str, denial: BankHttpDenial) {
        if let Some(record) = self.records.get_mut(token) {
            record.state = ContinuationState::Terminal;
            record.initial_outcome = denied(record.initial_request_id.clone(), denial);
            record.last_outcome = denied(record.last_request_id.clone(), denial);
            record.expires_at = Instant::now() + self.lifetime;
        }
    }

    fn new_token(&self) -> String {
        loop {
            let random = Alphanumeric.sample_string(&mut OsRng, 40);
            let token = format!("{TOKEN_PREFIX}{random}");
            if !self.records.contains_key(&token) {
                return token;
            }
        }
    }

    fn purge_expired(&mut self) {
        let now = Instant::now();
        self.records.retain(|_, record| record.expires_at > now);
        self.initial_requests
            .retain(|_, token| self.records.contains_key(token));
    }
}

fn delivered(
    request_id: String,
    activity: super::super::protocol::BankHttpAccountActivity,
    continuation: Option<String>,
    publication: super::super::protocol::BankHttpQueryPublication,
) -> BankHttpAccountActivityPageOutcome {
    BankHttpAccountActivityPageOutcome::Delivered {
        request_id,
        activity,
        continuation,
        publication,
    }
}

fn denied(request_id: String, denial: BankHttpDenial) -> BankHttpAccountActivityPageOutcome {
    BankHttpAccountActivityPageOutcome::Denied {
        request_id: Some(request_id),
        denial,
    }
}

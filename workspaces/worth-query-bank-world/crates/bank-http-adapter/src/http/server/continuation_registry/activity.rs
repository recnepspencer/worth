use super::{BankHttpContinuationRegistry, ContinuationRecord};
use std::time::Instant;

use crate::http::server::authenticated_owner::BankHttpAuthenticatedOwner;
use bank_domain::model::AccountId;
use bank_server::BankAccountActivityContinuation;

use crate::http::protocol::{BankHttpAccountActivityPageOutcome, BankHttpDenial};

pub(in crate::http::server) struct ActivityRecord {
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
    InFlight(String),
    Terminal,
}

#[derive(Eq, Hash, PartialEq)]
pub(in crate::http::server) struct InitialRequestIdentity {
    owner: BankHttpAuthenticatedOwner,
    account: AccountId,
    request_id: String,
}

pub(in crate::http::server) enum ResumeAdmission {
    Execute(BankAccountActivityContinuation),
    Replay(BankHttpAccountActivityPageOutcome),
    InFlight,
    Unavailable,
}

impl BankHttpContinuationRegistry {
    pub(in crate::http::server) fn replay_initial(
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
        self.records.get(token).and_then(|record| match record {
            ContinuationRecord::Activity(record) => {
                Some((token.clone(), record.initial_outcome.clone()))
            }
            _ => None,
        })
    }

    pub(in crate::http::server) fn register_initial(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        account: AccountId,
        request_id: String,
        continuation: Option<BankAccountActivityContinuation>,
        activity: crate::http::protocol::BankHttpAccountActivity,
        publication: crate::http::protocol::BankHttpQueryPublication,
    ) -> Result<BankHttpAccountActivityPageOutcome, Option<Box<BankAccountActivityContinuation>>>
    {
        self.purge_expired();
        if self.records.len() >= self.capacity {
            return Err(continuation.map(Box::new));
        }
        let token = self.new_token();
        let exposed = continuation.as_ref().map(|_| token.clone());
        let outcome = delivered(request_id.clone(), activity, exposed, publication);
        let state = continuation.map_or(ContinuationState::Terminal, ContinuationState::Ready);
        self.records.insert(
            token.clone(),
            ContinuationRecord::Activity(ActivityRecord {
                owner: owner.clone(),
                account,
                expires_at: Instant::now() + self.lifetime,
                state,
                initial_request_id: request_id.clone(),
                initial_outcome: outcome.clone(),
                last_request_id: request_id.clone(),
                last_outcome: outcome.clone(),
            }),
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

    pub(in crate::http::server) fn begin_resume(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        account: AccountId,
        request_id: &str,
        token: &str,
    ) -> ResumeAdmission {
        self.purge_expired();
        let Some(ContinuationRecord::Activity(record)) = self.records.get_mut(token) else {
            return ResumeAdmission::Unavailable;
        };
        if &record.owner != owner || record.account != account {
            return ResumeAdmission::Unavailable;
        }
        if record.last_request_id == request_id {
            return ResumeAdmission::Replay(record.last_outcome.clone());
        }
        match std::mem::replace(
            &mut record.state,
            ContinuationState::InFlight(request_id.to_owned()),
        ) {
            ContinuationState::Ready(continuation) => ResumeAdmission::Execute(continuation),
            ContinuationState::InFlight(pending) => {
                record.state = ContinuationState::InFlight(pending);
                ResumeAdmission::InFlight
            }
            ContinuationState::Terminal => {
                record.state = ContinuationState::Terminal;
                ResumeAdmission::Unavailable
            }
        }
    }

    pub(in crate::http::server) fn complete_resume(
        &mut self,
        token: &str,
        request_id: String,
        activity: crate::http::protocol::BankHttpAccountActivity,
        continuation: Option<BankAccountActivityContinuation>,
        publication: crate::http::protocol::BankHttpQueryPublication,
    ) -> Result<BankHttpAccountActivityPageOutcome, Option<Box<BankAccountActivityContinuation>>>
    {
        self.purge_expired();
        let Some(ContinuationRecord::Activity(record)) = self.records.get_mut(token) else {
            return Err(continuation.map(Box::new));
        };
        if !matches!(&record.state, ContinuationState::InFlight(pending) if pending == &request_id)
        {
            return Err(continuation.map(Box::new));
        }
        let exposed = continuation.as_ref().map(|_| token.to_owned());
        let outcome = delivered(request_id.clone(), activity, exposed, publication);
        if let Some(ContinuationRecord::Activity(record)) = self.records.get_mut(token) {
            record.state =
                continuation.map_or(ContinuationState::Terminal, ContinuationState::Ready);
            record.last_request_id = request_id;
            record.last_outcome = outcome.clone();
            record.expires_at = Instant::now() + self.lifetime;
        }
        Ok(outcome)
    }

    pub(in crate::http::server) fn fail_resume(
        &mut self,
        token: &str,
        request_id: String,
        outcome: BankHttpAccountActivityPageOutcome,
        scrub_initial: bool,
    ) {
        if let Some(ContinuationRecord::Activity(record)) = self.records.get_mut(token) {
            if let ContinuationState::Ready(custody) =
                std::mem::replace(&mut record.state, ContinuationState::Terminal)
            {
                self.retired_activity.push(custody);
            }
            record.last_request_id = request_id;
            if let BankHttpAccountActivityPageOutcome::Denied { denial, .. } = &outcome
                && scrub_initial
            {
                record.initial_outcome = denied(record.initial_request_id.clone(), *denial);
            }
            record.last_outcome = outcome;
            record.expires_at = Instant::now() + self.lifetime;
        }
    }

    pub(in crate::http::server) fn deny_protected_replay(
        &mut self,
        token: &str,
        denial: BankHttpDenial,
    ) {
        if let Some(ContinuationRecord::Activity(record)) = self.records.get_mut(token) {
            if let ContinuationState::Ready(custody) =
                std::mem::replace(&mut record.state, ContinuationState::Terminal)
            {
                self.retired_activity.push(custody);
            }
            record.initial_outcome = denied(record.initial_request_id.clone(), denial);
            record.last_outcome = denied(record.last_request_id.clone(), denial);
            record.expires_at = Instant::now() + self.lifetime;
        }
    }
}

impl ActivityRecord {
    pub(in crate::http::server) fn expires_at(&self) -> Instant {
        self.expires_at
    }
}

fn delivered(
    request_id: String,
    activity: crate::http::protocol::BankHttpAccountActivity,
    continuation: Option<String>,
    publication: crate::http::protocol::BankHttpQueryPublication,
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

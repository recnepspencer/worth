//! Signed minor-unit rules; no Bank proposal engine or production reducers.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum Purpose {
    Funding,
    Transfer,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Posting {
    pub account: u64,
    pub sequence: u64,
    pub journal: u64,
    pub amount: i64,
    pub purpose: Purpose,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Outcome {
    Committed(u64),
    Unauthorized,
    Insufficient,
}
#[derive(Clone)]
pub(super) struct Transfer {
    pub actor: u64,
    pub from: u64,
    pub to: u64,
    pub amount: i64,
    pub command: u64,
}
pub(super) struct Journal {
    pub postings: Vec<Posting>,
    pub balances: BTreeMap<u64, i64>,
    owners: BTreeMap<u64, u64>,
    completed: BTreeMap<(u64, u64), u64>,
    next: u64,
}
impl Journal {
    pub fn new(accounts: &[(u64, u64)], cash: u64, funded: u64, amount: i64) -> Self {
        let mut model = Self {
            postings: Vec::new(),
            balances: accounts
                .iter()
                .map(|(account, _)| (*account, 0))
                .chain([(cash, 0)])
                .collect(),
            owners: accounts.iter().copied().collect(),
            completed: BTreeMap::new(),
            next: 1,
        };
        model.fund(cash, funded, amount);
        model
    }
    pub fn fund(&mut self, cash: u64, account: u64, amount: i64) {
        self.post(cash, -amount, Purpose::Funding);
        self.post(account, amount, Purpose::Funding);
        self.next += 1;
    }
    fn post(&mut self, account: u64, amount: i64, purpose: Purpose) {
        *self.balances.get_mut(&account).unwrap() += amount;
        let sequence = 1 + self
            .postings
            .iter()
            .filter(|posting| posting.account == account)
            .count() as u64;
        self.postings.push(Posting {
            account,
            sequence,
            journal: self.next,
            amount,
            purpose,
        });
    }
    pub fn apply(&mut self, transfer: &Transfer) -> Outcome {
        if self.owners[&transfer.from] != transfer.actor {
            return Outcome::Unauthorized;
        }
        if let Some(journal) = self.completed.get(&(transfer.actor, transfer.command)) {
            return Outcome::Committed(*journal);
        }
        if self.balances[&transfer.from] < transfer.amount {
            return Outcome::Insufficient;
        }
        let journal = self.next;
        self.post(transfer.from, -transfer.amount, Purpose::Transfer);
        self.post(transfer.to, transfer.amount, Purpose::Transfer);
        self.next += 1;
        self.completed
            .insert((transfer.actor, transfer.command), journal);
        Outcome::Committed(journal)
    }
    pub fn canonical_postings(&self) -> Vec<Posting> {
        let mut postings = self.postings.clone();
        postings.sort();
        postings
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Assessment {
    Started,
    Proposed,
    Awaiting,
    Settled,
    Accepted,
}
#[derive(Clone, Copy)]
pub(super) enum AssessmentRequest {
    Start,
    Propose,
    Advance,
    Settle,
    RetrySettlement,
    Accept,
}
/// The declared protocol, evaluated without workflow or proposal APIs.
fn assessment_step(state: Option<Assessment>, request: AssessmentRequest) -> Option<Assessment> {
    use AssessmentRequest as R;
    match (state, request) {
        (None, R::Start) => Some(Assessment::Started),
        (Some(Assessment::Started), R::Propose) => Some(Assessment::Proposed),
        (Some(Assessment::Proposed), R::Advance) => Some(Assessment::Awaiting),
        (Some(Assessment::Awaiting), R::Settle) => Some(Assessment::Settled),
        (Some(Assessment::Settled), R::RetrySettlement) => Some(Assessment::Settled),
        (Some(Assessment::Settled), R::Accept) => Some(Assessment::Accepted),
        _ => None,
    }
}
#[test]
fn assessment_model_rejects_acceptance_before_settlement_and_preserves_retry_state() {
    use AssessmentRequest as R;
    let mut state = None;
    for (request, expected) in [
        (R::Start, Assessment::Started),
        (R::Propose, Assessment::Proposed),
        (R::Advance, Assessment::Awaiting),
        (R::Settle, Assessment::Settled),
        (R::RetrySettlement, Assessment::Settled),
        (R::Accept, Assessment::Accepted),
    ] {
        state = assessment_step(state, request);
        assert_eq!(state, Some(expected), "the hand-declared protocol");
    }
    assert_eq!(
        assessment_step(Some(Assessment::Awaiting), AssessmentRequest::Accept),
        None
    );
    assert_eq!(
        assessment_step(
            Some(Assessment::Settled),
            AssessmentRequest::RetrySettlement
        ),
        Some(Assessment::Settled)
    );
}
#[test]
fn journal_rules_conserve_cash_and_retry_without_another_posting() {
    let mut model = Journal::new(&[(11, 1), (12, 2)], 100, 11, 31);
    let transfer = Transfer {
        actor: 1,
        from: 11,
        to: 12,
        amount: 7,
        command: 1,
    };
    assert_eq!(model.apply(&transfer), Outcome::Committed(2));
    assert_eq!(model.apply(&transfer), Outcome::Committed(2));
    assert_eq!(
        model.apply(&Transfer {
            actor: 3,
            ..transfer.clone()
        }),
        Outcome::Unauthorized
    );
    assert_eq!(
        model.apply(&Transfer {
            amount: 25,
            command: 2,
            ..transfer
        }),
        Outcome::Insufficient
    );
    assert_eq!(model.postings.len(), 4);
    assert_eq!(model.balances[&11], 24);
    assert_eq!(model.balances[&12], 7);
    assert_eq!(
        model
            .postings
            .iter()
            .map(|posting| posting.amount)
            .sum::<i64>(),
        0
    );
}

//! Signed minor-unit rules; no Bank proposal engine or production reducers.
pub use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Purpose {
    Funding,
    Transfer,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Posting {
    pub account: u64,
    pub sequence: u64,
    pub journal: u64,
    pub amount: i64,
    pub purpose: Purpose,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Committed(u64),
    Unauthorized,
    Insufficient,
}
#[derive(Clone)]
pub struct Transfer {
    pub actor: u64,
    pub from: u64,
    pub to: u64,
    pub amount: i64,
    pub command: u64,
}
pub struct Journal {
    pub postings: Vec<Posting>,
    pub balances: BTreeMap<u64, i64>,
    pub owners: BTreeMap<u64, u64>,
    pub completed: BTreeMap<(u64, u64), u64>,
    pub next: u64,
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
    pub fn post(&mut self, account: u64, amount: i64, purpose: Purpose) {
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
pub enum Assessment {
    Started,
    Proposed,
    Awaiting,
    Settled,
    Accepted,
}
#[derive(Clone, Copy)]
pub enum AssessmentRequest {
    Start,
    Propose,
    Advance,
    Settle,
    RetrySettlement,
    Accept,
}
/// The declared protocol, evaluated without workflow or proposal APIs.
pub fn assessment_step(
    state: Option<Assessment>,
    request: AssessmentRequest,
) -> Option<Assessment> {
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

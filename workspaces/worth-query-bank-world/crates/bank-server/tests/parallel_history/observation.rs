//! The host policy or sealed request placement accompanies the same real transfers.
//! Runtime-created IDs are opaque symbols: preserve their exact native identity
//! and independently enforce one symbol per command and one command per journal.
mod transfer;
use super::{
    journal_model::{Outcome, Posting, Purpose, Transfer},
    seeded_world::{key, principal, World},
};
use bank_domain::{
    model::*,
    proposals::BankProposalDenial,
    schema::{PostingPurpose, SendMoney},
};
use bank_server::{
    mutations, queries, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
    BankReadControls,
};
use std::collections::BTreeMap;
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryOperationAuthorizationDenialKind,
    },
};
#[derive(Clone, Copy, Debug)]
pub(super) enum Posture {
    Serial,
    Leased(std::num::NonZeroUsize),
}
pub(super) struct AdvancementReport {
    pub observed_read_work: usize,
}
// Native journal symbols are bound from the authoritative receipts.
pub(super) struct Bindings {
    pub accounts: BTreeMap<AccountId, u64>,
    journals: BTreeMap<JournalEntryId, u64>,
    commands: BTreeMap<u64, JournalEntryId>,
}
impl Bindings {
    pub fn new(accounts: impl IntoIterator<Item = (AccountId, u64)>) -> Self {
        Self {
            accounts: accounts.into_iter().collect(),
            journals: BTreeMap::new(),
            commands: BTreeMap::new(),
        }
    }
    pub fn journal(&mut self, symbol: u64, native: JournalEntryId) {
        assert!(
            !self
                .journals
                .iter()
                .any(|(id, existing)| *existing == symbol && *id != native),
            "one semantic journal has one exact native identity"
        );
        if let Some(existing) = self.journals.insert(native, symbol) {
            assert_eq!(
                existing, symbol,
                "distinct semantic journals have distinct identities"
            );
        }
    }
    pub fn committed(&mut self, command: u64, native: JournalEntryId) -> Outcome {
        if let Some(previous) = self.commands.get(&command) {
            assert_eq!(
                *previous, native,
                "retry returns the original journal identity"
            );
        } else {
            self.commands.insert(command, native);
            self.journal(self.journals.len() as u64 + 1, native);
        }
        Outcome::Committed(self.journals[&native])
    }
    pub fn postings(&self, raw: &[NativePosting]) -> Vec<Posting> {
        let mut last = BTreeMap::new();
        for posting in raw {
            let previous = last.entry(posting.account).or_insert(0);
            assert_eq!(posting.sequence,*previous+1,"native postings arrive in each account's declared sequence, before any group normalization");
            *previous = posting.sequence;
        }
        let mut postings = raw
            .iter()
            .map(|posting| Posting {
                account: self.accounts[&posting.account],
                sequence: posting.sequence,
                journal: self.journals[&posting.journal],
                amount: posting.amount,
                purpose: posting.purpose,
            })
            .collect::<Vec<_>>();
        postings.sort();
        postings
    }
}
pub(super) struct Observer<'a> {
    world: &'a World,
    pub bindings: Bindings,
    receipts: BTreeMap<u64, WorthQueryApplicationCommitReceipt>,
    pub last: Vec<worth_query_host::facade::primary_graph::WorthQueryAdvancementObservationForTest>,
}
impl<'a> Observer<'a> {
    pub fn new(world: &'a World) -> Self {
        let mut bindings = Bindings::new([
            (world.accounts[0], 11),
            (world.accounts[1], 12),
            (world.cash, 100),
        ]);
        bindings.journal(1, world.funding_journal);
        Self {
            world,
            bindings,
            receipts: BTreeMap::new(),
            last: Vec::new(),
        }
    }
    pub fn balances(&self) -> BTreeMap<u64, i64> {
        self.bindings
            .accounts
            .iter()
            .map(|(account, symbol)| {
                let actor = match *symbol {
                    11 => 1,
                    12 => 2,
                    100 => 4,
                    _ => unreachable!(),
                };
                let principal = self.world.authenticate(actor);
                let result = self
                    .world
                    .world
                    .runtime
                    .query(queries::account_summary(*account))
                    .as_principal(&principal)
                    .controls(
                        BankReadControls::current(crate::support::request_scope(), 1).unwrap(),
                    )
                    .execute()
                    .unwrap();
                let [summary] = result.rows() else {
                    panic!("one account summary")
                };
                assert_eq!(
                    summary.current_balance(),
                    summary.available_balance(),
                    "no holds in this declared world"
                );
                (*symbol, summary.current_balance().minor_units())
            })
            .collect()
    }
    pub fn journal(&self) -> (Vec<Posting>, AdvancementReport) {
        let auditor = self.world.authenticate(4);
        let (raw, report) =
            journal_from(&self.world.world.runtime, self.world.institution, &auditor);
        (self.bindings.postings(&raw), report)
    }
}
pub(super) struct NativePosting {
    pub account: AccountId,
    pub sequence: u64,
    pub journal: JournalEntryId,
    pub amount: i64,
    pub purpose: Purpose,
}
pub(super) fn journal_from(
    runtime: &BankIdentityRuntime,
    institution: InstitutionId,
    auditor: &BankAuthenticatedPrincipal,
) -> (Vec<NativePosting>, AdvancementReport) {
    let result = runtime
        .query(queries::institution_audit(institution))
        .as_principal(auditor)
        .controls(BankReadControls::current(crate::support::request_scope(), 16).unwrap())
        .execute()
        .unwrap();
    let postings = result.rows()[0]
        .accounts()
        .iter()
        .flat_map(|account| account.entries())
        .map(|entry| NativePosting {
            account: entry.account(),
            sequence: entry.account_sequence().get(),
            journal: entry.journal(),
            amount: entry.amount().minor_units(),
            purpose: match entry.purpose() {
                PostingPurpose::OpeningFunding => Purpose::Funding,
                PostingPurpose::Transfer => Purpose::Transfer,
                other => panic!("unexpected purpose: {other:?}"),
            },
        })
        .collect();
    (
        postings,
        AdvancementReport {
            observed_read_work: result.receipt().inspect().ordinary_work_units(),
        },
    )
}

//! Part-two plug-in points: posture, advancement report, and ordered commits.
//! Until aggregate reports exist, observe typed outcomes and committed facts.
//! Runtime-created IDs are opaque symbols: preserve their exact native identity
//! and independently enforce one symbol per command and one command per journal.
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
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryOperationAuthorizationDenialKind,
    },
};
pub(super) enum Posture {
    Serial,
}
pub(super) struct AdvancementReport {
    pub observed_read_work: usize,
}
// Aggregate ordered commits wait for 7.7; current journal observations prove
// posting identity and content, not request-level semantic commit positions.
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
pub(super) struct SerialObserver<'a> {
    world: &'a World,
    pub bindings: Bindings,
    receipts: BTreeMap<u64, WorthQueryApplicationCommitReceipt>,
}
impl<'a> SerialObserver<'a> {
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
        }
    }
    pub fn transfer(
        &mut self,
        input: &Transfer,
        posture: Posture,
    ) -> Result<Outcome, Box<bank_server::BankMoneyMovementExecution>> {
        let Posture::Serial = posture;
        let actor = self.world.authenticate(input.actor);
        let recipient = if input.to == 12 { 2 } else { 1 };
        let result = self
            .world
            .world
            .runtime
            .mutate(mutations::send_money(SendMoney {
                from: AccountId::new(input.from).unwrap(),
                recipient: principal(recipient),
                amount: Money::from_minor(input.amount).unwrap(),
            }))
            .as_principal(&actor)
            .controls(BankMutationControls::new(
                crate::support::request_scope(),
                key(input.command),
            ))
            .execute();
        match result {
            Ok(WorthQueryApplicationMutationOutcome::Committed { result, receipt }) => {
                self.receipts.insert(input.command, receipt);
                Ok(self.bindings.committed(input.command, result.journal))
            }
            Ok(WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)) => {
                assert!(
                    receipt.is_same_authoritative_commit(&self.receipts[&input.command]),
                    "retry recovers exactly the original commit"
                );
                let journal = self.bindings.commands[&input.command];
                Ok(Outcome::Committed(self.bindings.journals[&journal]))
            }
            Ok(WorthQueryApplicationMutationOutcome::DomainDenied(
                BankProposalDenial::InsufficientFunds(_),
            )) => Ok(Outcome::Insufficient),
            Err(WorthQueryApplicationRequestMutationDenial::Authorization(denial)) => {
                assert_eq!(
                    denial.kind(),
                    WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
                    "typed authorization cause"
                );
                Ok(Outcome::Unauthorized)
            }
            other => Err(Box::new(other)),
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
                        BankReadControls::current(crate::support::request_scope(), 1, 10000)
                            .unwrap(),
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
        .controls(BankReadControls::current(crate::support::request_scope(), 16, 10000).unwrap())
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

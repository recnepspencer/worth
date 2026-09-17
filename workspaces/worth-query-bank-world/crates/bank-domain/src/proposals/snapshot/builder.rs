use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::accounting::BankAccount;
use crate::accounting::BankJournalEntry;
use crate::model::{AccountId, BankPrincipalId, BankSnapshotVersion, BusinessId, InstitutionId};
use crate::payments::BusinessPayment;
use crate::proposals::BankAccountAuthorization;

use super::{BankSnapshot, BankSnapshotAuthority};
use crate::proposals::BankProposalDenial;

pub struct BankSnapshotBuilder {
    snapshot: BankSnapshot,
    valid: bool,
}

impl BankSnapshotBuilder {
    pub fn new(version: BankSnapshotVersion) -> Self {
        Self {
            snapshot: BankSnapshot {
                version,
                authority: Arc::new(BankSnapshotAuthority { _private: () }),
                institutions: BTreeSet::new(),
                principals: BTreeSet::new(),
                businesses: BTreeSet::new(),
                accounts: BTreeMap::new(),
                primary_personal_accounts: BTreeMap::new(),
                business_accounts: BTreeMap::new(),
                institution_cash_accounts: BTreeMap::new(),
                projected_account_revisions: BTreeMap::new(),
                journal: Vec::new(),
                payments: BTreeMap::new(),
                authorizations: BTreeMap::new(),
                reversed_journals: BTreeSet::new(),
            },
            valid: true,
        }
    }

    pub fn institution(mut self, institution: InstitutionId) -> Self {
        self.snapshot.institutions.insert(institution);
        self
    }

    pub fn principal(mut self, principal: BankPrincipalId) -> Self {
        self.snapshot.principals.insert(principal);
        self
    }

    pub fn business(mut self, business: BusinessId) -> Self {
        self.snapshot.businesses.insert(business);
        self
    }

    pub fn institution_cash_account(
        mut self,
        account: AccountId,
        institution: InstitutionId,
    ) -> Self {
        self.insert_fixture_account(BankAccount::institution_cash(account, institution));
        self
    }

    pub fn personal_account(
        mut self,
        account: AccountId,
        institution: InstitutionId,
        owner: BankPrincipalId,
        display_name: crate::model::AccountName,
        status: crate::schema::AccountStatus,
    ) -> Self {
        self.insert_fixture_account(BankAccount::personal_with_status(
            account,
            institution,
            owner,
            display_name,
            status,
        ));
        self
    }

    pub fn business_account_fixture(
        mut self,
        account: AccountId,
        institution: InstitutionId,
        business: BusinessId,
        display_name: crate::model::AccountName,
        status: crate::schema::AccountStatus,
    ) -> Self {
        self.insert_fixture_account(BankAccount::business_with_status(
            account,
            institution,
            business,
            display_name,
            status,
        ));
        self
    }

    pub fn projected_account(mut self, account: BankAccount) -> Self {
        self.insert_fixture_account(account);
        self
    }

    pub fn projected_account_revision(
        mut self,
        account: AccountId,
        revision: crate::model::AccountJournalRevision,
    ) -> Self {
        if !self.snapshot.accounts.contains_key(&account)
            || self
                .snapshot
                .projected_account_revisions
                .insert(account, revision)
                .is_some()
        {
            self.valid = false;
        }
        self
    }

    pub fn projected_journal(mut self, entry: BankJournalEntry) -> Self {
        let collides = self
            .snapshot
            .journal
            .iter()
            .any(|candidate| candidate.id() == entry.id());
        let posting_collision = entry.postings().iter().any(|posting| {
            self.snapshot
                .journal
                .iter()
                .flat_map(BankJournalEntry::postings)
                .any(|candidate| candidate.id() == posting.id())
        });
        let duplicate_posting = entry
            .postings()
            .iter()
            .map(|posting| posting.id())
            .collect::<BTreeSet<_>>()
            .len()
            != entry.postings().len();
        if collides || posting_collision || duplicate_posting {
            self.valid = false;
            return self;
        }
        if let Some(original) = entry.reversal_of() {
            self.snapshot.reversed_journals.insert(original);
        }
        self.snapshot.append_projected_journal(entry);
        self
    }

    pub fn projected_payment(mut self, payment: BusinessPayment) -> Self {
        let collides = self.snapshot.payments.contains_key(&payment.id());
        if collides {
            self.valid = false;
            return self;
        }
        self.snapshot.insert_payment(payment);
        self
    }

    pub fn projected_authorization(mut self, authorization: BankAccountAuthorization) -> Self {
        let collides = self
            .snapshot
            .authorizations
            .contains_key(&authorization.id());
        if collides {
            self.valid = false;
            return self;
        }
        self.snapshot.insert_authorization(authorization);
        self
    }

    pub fn build(self) -> Result<BankSnapshot, BankProposalDenial> {
        if self.valid && self.snapshot.has_valid_topology() {
            Ok(self.snapshot)
        } else {
            Err(BankProposalDenial::SnapshotInvariantViolated)
        }
    }

    pub fn build_decision_projection(
        self,
        required_balance_accounts: impl IntoIterator<Item = AccountId>,
    ) -> Result<super::super::BankDecisionSnapshot, BankProposalDenial> {
        self.build_decision_projection_with_balances(required_balance_accounts, [])
    }

    pub fn build_decision_projection_with_balances(
        self,
        required_balance_accounts: impl IntoIterator<Item = AccountId>,
        starting_balances: impl IntoIterator<
            Item = (AccountId, crate::model::SignedMoney<crate::model::USD>),
        >,
    ) -> Result<super::super::BankDecisionSnapshot, BankProposalDenial> {
        let snapshot = self.build()?;
        let required_balance_accounts = required_balance_accounts
            .into_iter()
            .collect::<BTreeSet<_>>();
        let starting_balances = starting_balances.into_iter().collect::<BTreeMap<_, _>>();
        if required_balance_accounts.iter().any(|account| {
            snapshot.account(*account).is_none() || !starting_balances.contains_key(account)
        }) || starting_balances
            .keys()
            .any(|account| snapshot.account(*account).is_none())
        {
            return Err(BankProposalDenial::SnapshotInvariantViolated);
        }
        Ok(super::super::BankDecisionSnapshot::new(
            snapshot,
            required_balance_accounts,
            starting_balances,
        ))
    }

    fn insert_fixture_account(&mut self, account: BankAccount) {
        let collides = self.snapshot.accounts.contains_key(&account.id())
            || account
                .personal_owner()
                .is_some_and(|owner| self.snapshot.primary_personal_accounts.contains_key(&owner))
            || account
                .business_owner()
                .is_some_and(|business| self.snapshot.business_accounts.contains_key(&business))
            || (account.kind() == crate::schema::AccountKind::InstitutionCash
                && self
                    .snapshot
                    .institution_cash_accounts
                    .contains_key(&account.institution()));
        if collides {
            self.valid = false;
            return;
        }
        self.snapshot.insert_account(account);
    }
}

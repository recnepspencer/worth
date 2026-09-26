use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::accounting::{BankAccount, BankJournalEntry};
use crate::model::{
    AccountAuthorizationId, AccountId, AccountJournalRevision, BankPrincipalId,
    BankSnapshotVersion, BusinessId, CustomerRole, InstitutionId, JournalEntryId, PaymentId,
};
use crate::payments::BusinessPayment;
use crate::schema::PaymentStatus;

use super::BankAccountAuthorization;

mod builder;

pub use builder::BankSnapshotBuilder;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct BankSnapshotAuthority {
    _private: (),
}

pub(crate) struct BankSnapshotBasis {
    version: BankSnapshotVersion,
    authority: Arc<BankSnapshotAuthority>,
}

impl BankSnapshotBasis {
    pub(crate) const fn version(&self) -> BankSnapshotVersion {
        self.version
    }

    pub(crate) fn matches(&self, snapshot: &BankSnapshot) -> bool {
        self.version == snapshot.version && Arc::ptr_eq(&self.authority, &snapshot.authority)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankSnapshot {
    version: BankSnapshotVersion,
    authority: Arc<BankSnapshotAuthority>,
    institutions: BTreeSet<InstitutionId>,
    principals: BTreeSet<BankPrincipalId>,
    businesses: BTreeSet<BusinessId>,
    accounts: BTreeMap<AccountId, BankAccount>,
    primary_personal_accounts: BTreeMap<BankPrincipalId, AccountId>,
    business_accounts: BTreeMap<BusinessId, AccountId>,
    institution_cash_accounts: BTreeMap<InstitutionId, AccountId>,
    projected_account_revisions: BTreeMap<AccountId, AccountJournalRevision>,
    journal: Vec<BankJournalEntry>,
    payments: BTreeMap<PaymentId, BusinessPayment>,
    authorizations: BTreeMap<AccountAuthorizationId, BankAccountAuthorization>,
    reversed_journals: BTreeSet<JournalEntryId>,
}

impl BankSnapshot {
    pub const fn version(&self) -> BankSnapshotVersion {
        self.version
    }

    pub(crate) fn retain_basis(&self) -> BankSnapshotBasis {
        BankSnapshotBasis {
            version: self.version,
            authority: Arc::clone(&self.authority),
        }
    }

    pub fn accounts(&self) -> impl ExactSizeIterator<Item = &BankAccount> {
        self.accounts.values()
    }

    pub fn institutions(&self) -> impl ExactSizeIterator<Item = InstitutionId> + '_ {
        self.institutions.iter().copied()
    }

    pub fn principals(&self) -> impl ExactSizeIterator<Item = BankPrincipalId> + '_ {
        self.principals.iter().copied()
    }

    pub fn businesses(&self) -> impl ExactSizeIterator<Item = BusinessId> + '_ {
        self.businesses.iter().copied()
    }

    pub fn account(&self, id: AccountId) -> Option<&BankAccount> {
        self.accounts.get(&id)
    }

    pub fn account_journal_revision(&self, id: AccountId) -> Option<AccountJournalRevision> {
        self.accounts.get(&id)?;
        if let Some(revision) = self.projected_account_revisions.get(&id) {
            return Some(*revision);
        }
        let posting_count = self
            .journal
            .iter()
            .flat_map(BankJournalEntry::postings)
            .filter(|posting| posting.account() == id)
            .try_fold(0_u64, |count, _| count.checked_add(1))?;
        Some(AccountJournalRevision::from_posting_count(posting_count))
    }

    pub fn journal(&self) -> &[BankJournalEntry] {
        &self.journal
    }

    pub fn payment(&self, id: PaymentId) -> Option<&BusinessPayment> {
        self.payments.get(&id)
    }

    pub fn payments(&self) -> impl ExactSizeIterator<Item = &BusinessPayment> {
        self.payments.values()
    }

    pub fn authorizations(&self) -> impl ExactSizeIterator<Item = &BankAccountAuthorization> {
        self.authorizations.values()
    }

    pub fn authorization(&self, id: AccountAuthorizationId) -> Option<&BankAccountAuthorization> {
        self.authorizations.get(&id)
    }

    pub fn has_authorization(&self, account: AccountId, principal: BankPrincipalId) -> bool {
        self.authorizations
            .values()
            .any(|candidate| candidate.account() == account && candidate.principal() == principal)
    }

    /// The principals granted a pending payment's approval workflow: every
    /// Approver on its source account while the payment awaits a decision.
    pub fn payment_approval_grantees(
        &self,
        payment: &BusinessPayment,
    ) -> BTreeSet<BankPrincipalId> {
        if payment.status() != PaymentStatus::ApprovalRequired {
            return BTreeSet::new();
        }
        self.authorizations
            .values()
            .filter(|authorization| {
                authorization.account() == payment.source()
                    && authorization.role() == CustomerRole::Approver
            })
            .map(|authorization| authorization.principal())
            .collect()
    }

    pub fn primary_account(&self, principal: BankPrincipalId) -> Option<AccountId> {
        self.primary_personal_accounts.get(&principal).copied()
    }

    pub fn business_account(&self, business: BusinessId) -> Option<AccountId> {
        self.business_accounts.get(&business).copied()
    }

    pub fn institution_cash_account(&self, institution: InstitutionId) -> Option<AccountId> {
        self.institution_cash_accounts.get(&institution).copied()
    }

    pub fn is_known_institution(&self, institution: InstitutionId) -> bool {
        self.institutions.contains(&institution)
    }

    pub fn is_known_principal(&self, principal: BankPrincipalId) -> bool {
        self.principals.contains(&principal)
    }

    pub fn is_known_business(&self, business: BusinessId) -> bool {
        self.businesses.contains(&business)
    }

    pub fn is_reversed(&self, journal: JournalEntryId) -> bool {
        self.reversed_journals.contains(&journal)
    }

    pub(crate) fn insert_account(&mut self, account: BankAccount) {
        match (account.personal_owner(), account.business_owner()) {
            (Some(principal), None) => {
                self.primary_personal_accounts
                    .insert(principal, account.id());
            }
            (None, Some(business)) => {
                self.business_accounts.insert(business, account.id());
            }
            (None, None) => {
                self.institution_cash_accounts
                    .insert(account.institution(), account.id());
            }
            (Some(_), Some(_)) => unreachable!("account ownership is exclusive"),
        }
        self.accounts.insert(account.id(), account);
    }

    pub(crate) fn append_projected_journal(&mut self, entry: BankJournalEntry) {
        self.journal.push(entry);
    }

    pub(crate) fn append_proposed_journal(
        &mut self,
        entry: BankJournalEntry,
    ) -> Result<(), super::BankProposalDenial> {
        for posting in entry.postings() {
            if let Some(revision) = self.projected_account_revisions.get_mut(&posting.account()) {
                *revision = revision
                    .next()
                    .ok_or(super::BankProposalDenial::ArithmeticOverflow)?;
            }
        }
        self.journal.push(entry);
        Ok(())
    }

    pub(crate) fn insert_payment(&mut self, payment: BusinessPayment) {
        self.payments.insert(payment.id(), payment);
    }

    pub(crate) fn replace_payment(&mut self, payment: BusinessPayment) {
        self.payments.insert(payment.id(), payment);
    }

    pub(crate) fn insert_authorization(&mut self, authorization: BankAccountAuthorization) {
        self.authorizations
            .insert(authorization.id(), authorization);
    }

    pub(crate) fn remove_authorization(
        &mut self,
        id: AccountAuthorizationId,
    ) -> Option<BankAccountAuthorization> {
        self.authorizations.remove(&id)
    }

    pub(crate) fn mark_reversed(&mut self, journal: JournalEntryId) {
        self.reversed_journals.insert(journal);
    }

    pub fn journal_entry(&self, id: JournalEntryId) -> Option<&BankJournalEntry> {
        self.journal.iter().find(|entry| entry.id() == id)
    }

    pub(crate) fn has_valid_topology(&self) -> bool {
        self.account_references_are_valid()
            && self.ownership_indexes_are_valid()
            && self.payment_and_authorization_references_are_valid()
            && self.reversal_references_are_valid()
    }

    fn account_references_are_valid(&self) -> bool {
        self.accounts.values().all(|account| {
            self.institutions.contains(&account.institution())
                && account
                    .personal_owner()
                    .is_none_or(|owner| self.principals.contains(&owner))
                && account
                    .business_owner()
                    .is_none_or(|business| self.businesses.contains(&business))
        })
    }

    fn ownership_indexes_are_valid(&self) -> bool {
        let personal = self
            .primary_personal_accounts
            .iter()
            .all(|(principal, account)| {
                self.accounts
                    .get(account)
                    .is_some_and(|value| value.personal_owner() == Some(*principal))
            });
        let business = self.business_accounts.iter().all(|(business, account)| {
            self.accounts
                .get(account)
                .is_some_and(|value| value.business_owner() == Some(*business))
        });
        let cash = self
            .institution_cash_accounts
            .iter()
            .all(|(institution, account)| {
                self.accounts.get(account).is_some_and(|value| {
                    value.institution() == *institution
                        && value.kind() == crate::schema::AccountKind::InstitutionCash
                })
            });
        personal && business && cash
    }

    fn payment_and_authorization_references_are_valid(&self) -> bool {
        self.payments.values().all(|payment| {
            self.businesses.contains(&payment.business())
                && self.accounts.contains_key(&payment.source())
                && self.accounts.contains_key(&payment.destination())
                && self.principals.contains(&payment.initiator())
                && payment
                    .deciding_principal()
                    .is_none_or(|principal| self.principals.contains(&principal))
        }) && self.authorizations.values().all(|authorization| {
            self.accounts.contains_key(&authorization.account())
                && self.principals.contains(&authorization.principal())
        })
    }

    fn reversal_references_are_valid(&self) -> bool {
        self.reversed_journals
            .iter()
            .all(|id| self.journal_entry(*id).is_some())
    }
}

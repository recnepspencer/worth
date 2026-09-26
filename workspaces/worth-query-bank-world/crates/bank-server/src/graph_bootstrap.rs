use bank_domain::model::AccountJournalRevision;
use bank_domain::proposals::BankSnapshot;
use bank_domain::schema::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};

use crate::{BankBusinessOwnerSeed, BankEmployeeAssignmentSeed};

mod estate;
mod journal;
mod payment;

pub(crate) fn bind_bank_world_with_estate(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
    owners: &[BankBusinessOwnerSeed],
    employees: &[BankEmployeeAssignmentSeed],
    estate_world: Option<&bank_domain::estate::BankEstateWorld>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bind_bank_world_with_revisions(graph, snapshot, owners, employees, |_, derived| derived)?;
    if let Some(estate_world) = estate_world {
        estate::bind_estate_world(graph, estate_world)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn bind_bank_world_with_revision_override(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
    owners: &[BankBusinessOwnerSeed],
    employees: &[BankEmployeeAssignmentSeed],
    account: bank_domain::model::AccountId,
    replacement: AccountJournalRevision,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bind_bank_world_with_revisions(graph, snapshot, owners, employees, |candidate, derived| {
        if candidate == account {
            replacement
        } else {
            derived
        }
    })
}

fn bind_bank_world_with_revisions(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
    owners: &[BankBusinessOwnerSeed],
    employees: &[BankEmployeeAssignmentSeed],
    revision: impl Fn(bank_domain::model::AccountId, AccountJournalRevision) -> AccountJournalRevision,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bind_institutions(graph, snapshot)?;
    bind_businesses(graph, snapshot)?;
    bind_accounts(graph, snapshot, revision)?;
    bind_authorizations(graph, snapshot)?;
    bind_employees(graph, employees)?;
    payment::bind_payments(graph, snapshot)?;
    journal::bind_journal(graph, snapshot)?;
    bind_ownership_relations(graph, snapshot, owners)?;
    bind_authorization_relations(graph, snapshot)?;
    bind_employee_relations(graph, employees)?;
    payment::bind_payment_relations(graph, snapshot)
}

fn bind_institutions(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for institution in snapshot.institutions() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Institution::reference(),
                entity_key(institution_key(institution.get())),
            )
            .field(InstitutionIdentityField::reference(), institution),
        )?;
    }
    Ok(())
}

fn bind_businesses(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for business in snapshot.businesses() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Business::reference(),
                entity_key(business_key(business.get())),
            )
            .field(BusinessIdentityField::reference(), business),
        )?;
    }
    Ok(())
}

fn bind_accounts(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
    revision: impl Fn(bank_domain::model::AccountId, AccountJournalRevision) -> AccountJournalRevision,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for account in snapshot.accounts() {
        let posting_count = snapshot
            .journal()
            .iter()
            .flat_map(|journal| journal.postings())
            .filter(|posting| posting.account() == account.id())
            .count();
        let revision = revision(
            account.id(),
            AccountJournalRevision::from_posting_count(
                u64::try_from(posting_count).expect("bank posting count fits a typed revision"),
            ),
        );
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Account::reference(),
                entity_key(account_key(account.id())),
            )
            .field(AccountIdentity::reference(), account.id())
            .field(
                AccountDisplayName::reference(),
                account.display_name().clone(),
            )
            .field(Kind::reference(), account.kind())
            .field(AccountingRevision::reference(), revision)
            .field(Status::reference(), account.status()),
        )?;
    }
    Ok(())
}

fn bind_authorizations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for authorization in snapshot.authorizations() {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                AccountAuthorization::reference(),
                entity_key(authorization_key(authorization.id())),
            )
            .field(
                AccountAuthorizationIdentity::reference(),
                authorization.id(),
            )
            .field(AuthorizationRole::reference(), authorization.role()),
        )?;
    }
    Ok(())
}

fn bind_employees(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    employees: &[BankEmployeeAssignmentSeed],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for employee in employees {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                EmployeeAssignment::reference(),
                entity_key(employee_key(employee.id().get())),
            )
            .field(EmployeeAssignmentIdentityField::reference(), employee.id())
            .field(AssignmentRole::reference(), employee.role()),
        )?;
    }
    Ok(())
}

fn bind_ownership_relations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
    owners: &[BankBusinessOwnerSeed],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for account in snapshot.accounts() {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            InstitutionAccount::reference(),
            format!("institution-account:{}", account.id().canonical_text()),
            entity_key(institution_key(account.institution().get())),
            entity_key(account_key(account.id())),
        ))?;
        if account.kind() == AccountKind::InstitutionCash {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                InstitutionCashAccount::reference(),
                format!("institution-cash-account:{}", account.id().canonical_text()),
                entity_key(institution_key(account.institution().get())),
                entity_key(account_key(account.id())),
            ))?;
        }
        if let Some(owner) = account.personal_owner() {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                PersonalOwner::reference(),
                format!("personal-owner:{}", account.id().canonical_text()),
                entity_key(principal_key(owner.get())),
                entity_key(account_key(account.id())),
            ))?;
        }
        if let Some(business) = account.business_owner() {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                BusinessAccount::reference(),
                format!("business-account:{}", account.id().canonical_text()),
                entity_key(business_key(business.get())),
                entity_key(account_key(account.id())),
            ))?;
        }
    }
    for owner in owners {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            BusinessOwner::reference(),
            format!(
                "business-owner:{}:{}",
                owner.business().get(),
                owner.principal().get()
            ),
            entity_key(business_key(owner.business().get())),
            entity_key(principal_key(owner.principal().get())),
        ))?;
    }
    Ok(())
}

fn bind_authorization_relations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    snapshot: &BankSnapshot,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for authorization in snapshot.authorizations() {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            AccountAuthorizedUser::reference(),
            format!("authorized-user:{}", authorization.id().canonical_text()),
            entity_key(principal_key(authorization.principal().get())),
            entity_key(authorization_key(authorization.id())),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            AuthorizationAccount::reference(),
            format!(
                "authorization-account:{}",
                authorization.id().canonical_text()
            ),
            entity_key(authorization_key(authorization.id())),
            entity_key(account_key(authorization.account())),
        ))?;
    }
    Ok(())
}

fn bind_employee_relations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BankSchema>,
    employees: &[BankEmployeeAssignmentSeed],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for employee in employees {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            InstitutionEmployee::reference(),
            format!("institution-employee:{}", employee.id().get()),
            entity_key(institution_key(employee.institution().get())),
            entity_key(employee_key(employee.id().get())),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            AssignmentPrincipal::reference(),
            format!("assignment-principal:{}", employee.id().get()),
            entity_key(employee_key(employee.id().get())),
            entity_key(principal_key(employee.principal().get())),
        ))?;
    }
    Ok(())
}

fn entity_key<Schema, Entity>(value: String) -> WorthQueryApplicationEntityKey<Schema, Entity> {
    WorthQueryApplicationEntityKey::new(value)
        .expect("bank keys are bounded canonical numeric identities")
}

pub(crate) fn principal_key(id: u64) -> String {
    format!("bank-principal:{id}")
}

fn institution_key(id: u64) -> String {
    format!("bank-institution:{id}")
}

fn business_key(id: u64) -> String {
    format!("bank-business:{id}")
}

pub(crate) fn account_key(id: bank_domain::model::AccountId) -> String {
    format!("bank-account:{}", id.canonical_text())
}

pub(crate) fn authorization_key(id: bank_domain::model::AccountAuthorizationId) -> String {
    format!("bank-authorization:{}", id.canonical_text())
}

fn employee_key(id: u64) -> String {
    format!("bank-employee:{id}")
}

pub(super) fn payment_key(id: bank_domain::model::PaymentId) -> String {
    format!("bank-payment:{}", id.canonical_text())
}

pub(crate) fn journal_key(id: bank_domain::model::JournalEntryId) -> String {
    format!("bank-journal:{}", id.canonical_text())
}

pub(crate) fn posting_key(id: bank_domain::model::PostingId) -> String {
    format!("bank-posting:{}", id.canonical_text())
}

pub(super) fn payment_workflow_grant_key(
    payment: bank_domain::model::PaymentId,
    principal: bank_domain::model::BankPrincipalId,
) -> String {
    format!(
        "approved-payment-workflow-grant:{}:{}",
        payment.canonical_text(),
        principal.get()
    )
}

pub(super) fn approval_key(payment_id: bank_domain::model::PaymentId) -> String {
    format!("bank-payment-approval:{}", payment_id.canonical_text())
}

pub use bank_domain::schema::{AccountStatus, AccountingRevision, Status};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
pub use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};

pub use super::*;

pub fn assert_warm_canonical_work_is_zero(
    phases: worth_query_host::facade::domain::WorthQueryCanonicalWorkPhases,
) {
    let zero = worth_query_host::facade::domain::WorthQueryCanonicalWorkEvidence::default();
    assert!(phases.installation().basis_preparations() > 0);
    assert!(phases.installation().canonical_encoded_bytes() > 0);
    assert!(phases.admission().basis_preparations() > 0);
    assert!(phases.admission().canonical_encoded_bytes() > 0);
    assert_eq!(phases.execution(), zero);
    assert_eq!(phases.provider_commit(), zero);
    assert_eq!(phases.projection(), zero);
    assert_eq!(phases.live_delivery(), zero);
    assert_eq!(phases.retry_resolution(), zero);
    assert_eq!(phases.recovery_inspection(), zero);
    assert_eq!(phases.publication(), zero);
}

pub fn account_summary(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> bank_domain::reads::AccountSummary {
    fixture
        .world
        .runtime
        .query(queries::account_summary(account))
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("account summary must be query-visible")
        .into_rows()
        .into_iter()
        .next()
        .expect("the requested account must be visible")
}

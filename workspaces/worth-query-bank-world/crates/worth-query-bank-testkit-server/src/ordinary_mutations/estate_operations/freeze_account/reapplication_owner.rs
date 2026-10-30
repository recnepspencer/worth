//! A shared EstateAction codec keeps exact freeze meaning in the ordinary source owner.
pub use super::{
    correction::{freeze_and_close, limits},
    estate_account_status,
    fixture::{revocable_freeze_world, FreezeFixture},
    foreign_account_status,
};
pub use crate::facade::support::request_scope;
pub use bank_domain::schema::{AccountStatus, BankSchema};
pub use worth_query_host::facade::{application_entry::*, primary_graph::*};

pub fn restore_and_close(
    fixture: &FreezeFixture,
    source: &WorthQueryRecordedCorrectionSource,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let prepared = fixture
        .world
        .runtime
        .prepare_estate_freeze_correction(&specialist, &scope, &source.coordinates(), key, limits())
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh restoration prepares");
    };
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("fresh restoration completes");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("fresh restoration receipt");
    };
    let receipt = receipt.clone();
    assert!(completed.close(&request).is_ok());
    assert_eq!(estate_account_status(fixture), AccountStatus::Open);
    receipt
}

pub fn choose<'a>(
    fixture: &'a FreezeFixture,
    source: &WorthQueryRecordedCorrectionSource,
    reversal: &WorthQueryApplicationCommitReceipt,
) -> WorthQuerySelectedCorrectionAlternative<'a, BankSchema> {
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let selected = request
        .corrections()
        .select_recorded(source, limits())
        .unwrap();
    let mut page = request
        .corrections()
        .alternatives(&selected, limits(), std::num::NonZeroUsize::MIN)
        .unwrap();
    let mut entries = page.entries().to_vec();
    while let Some(continuation) = page.continuation() {
        let next = request
            .corrections()
            .continue_alternatives(
                &selected,
                limits(),
                std::num::NonZeroUsize::MIN,
                continuation,
            )
            .unwrap();
        entries.extend_from_slice(next.entries());
        page = next;
    }
    let chosen = entries
        .iter()
        .find(|entry| entry.performed_commit() == reversal.commit_reference())
        .expect("the exact historical restoration remains in canonical alternatives");
    page.select(chosen, limits()).unwrap()
}

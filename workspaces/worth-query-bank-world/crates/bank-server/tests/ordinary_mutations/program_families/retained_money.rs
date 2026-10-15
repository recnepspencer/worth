//! Existing retained funding/reversal declarations deliver genuine original World observations.
use super::*;
pub(super) fn fund_opening(
    fixture: &fixture::OrdinaryReadFixture,
    teller: &bank_server::BankAuthenticatedPrincipal,
    created_personal: bank_domain::model::AccountId,
) {
    let scope = request_scope();
    let request = fixture.world.runtime.request(&teller, &scope);
    let source = fixture
        .world
        .runtime
        .mutate(mutations::apply_opening_funding(ApplyOpeningFunding {
            institution: fixture.institution,
            account: created_personal,
            amount: Money::from_minor(3_000).unwrap(),
        }))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("opening-funding"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared retained money: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        source.advance(&request, std::num::NonZeroUsize::new(128).unwrap())
    else {
        panic!("fresh retained money completion")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: Some(retained),
        ..
    } = done.evidence()
    else {
        panic!("authentic retained commit")
    };
    assert!(receipt.emitted_effect_count() > 0);
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("retained close: {cause:?}"));
}

pub(super) fn reverse_deposit(
    fixture: &fixture::OrdinaryReadFixture,
    teller: &bank_server::BankAuthenticatedPrincipal,
    journal: bank_domain::model::JournalEntryId,
) {
    let scope = request_scope();
    let request = fixture.world.runtime.request(&teller, &scope);
    let source = fixture
        .world
        .runtime
        .mutate(mutations::reverse_journal(ReverseJournal {
            institution: id(InstitutionId::new, 1),
            journal,
            reason: ReversalReason::OperatorCorrection,
        }))
        .as_principal(&teller)
        .controls(BankMutationControls::new(scope.clone(), key("reverse")))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared retained money: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        source.advance(&request, std::num::NonZeroUsize::new(128).unwrap())
    else {
        panic!("fresh retained money completion")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: Some(retained),
        ..
    } = done.evidence()
    else {
        panic!("authentic retained commit")
    };
    assert!(receipt.emitted_effect_count() > 0);
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("retained close: {cause:?}"));
}

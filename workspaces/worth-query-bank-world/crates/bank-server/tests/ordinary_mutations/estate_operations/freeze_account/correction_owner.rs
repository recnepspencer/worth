use super::{
    correction::{freeze_and_close, limits},
    estate_account_status,
    fixture::revocable_freeze_world,
};
use crate::support::request_scope;
use bank_domain::schema::AccountStatus;
use worth_query_host::facade::application_entry::*;

#[test]
fn source_only_freeze_correction_closes_and_replays_through_its_original_owner() {
    let fixture = revocable_freeze_world("recorded-freeze-source-owner");
    let source = freeze_and_close(&fixture);
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let prepared = fixture
        .world
        .runtime
        .prepare_estate_freeze_correction(
            &specialist,
            &scope,
            &source.coordinates(),
            "source-owned-restoration",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh source-only restoration prepares");
    };
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("source-only correction completes");
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: None,
        ..
    } = completed.evidence()
    else {
        panic!("source-only mode carries the performed receipt");
    };
    let receipt = receipt.clone();
    assert_eq!(
        receipt.aftermath_causality().unwrap().recorded_source(),
        &source
    );
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    assert!(completed.close(&request).is_ok());

    let replay = fixture
        .world
        .runtime
        .prepare_estate_freeze_correction(
            &specialist,
            &scope,
            &source.coordinates(),
            "source-owned-restoration",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Continued(
        WorthQuerySourceOnlyAdvance::Completed(replayed),
    ) = replay
    else {
        panic!("source-only retry recovers the original completion directly");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt: recovered,
        retained: None,
        ..
    } = replayed.evidence()
    else {
        panic!("recovered source-only completion has its original receipt");
    };
    assert_eq!(recovered.commit_reference(), receipt.commit_reference());
    assert_eq!(
        recovered.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert!(replayed.close(&request).is_ok());
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
}

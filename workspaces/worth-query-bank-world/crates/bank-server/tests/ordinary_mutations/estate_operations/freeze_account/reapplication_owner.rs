//! A shared EstateAction codec keeps exact freeze meaning in the ordinary source owner.
use super::{
    correction::{freeze_and_close, limits},
    estate_account_status,
    fixture::{revocable_freeze_world, FreezeFixture},
    foreign_account_status,
};
use crate::support::request_scope;
use bank_domain::schema::{AccountStatus, BankSchema};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
fn shared_codec_reapplication_closes_replays_and_uses_current_source_capability() {
    let fixture = revocable_freeze_world("shared-codec-freeze-reapplication");
    let source = freeze_and_close(&fixture);
    let first_reverse = restore_and_close(&fixture, &source, "shared-codec-restore");
    let alternative = choose(&fixture, &source, &first_reverse);
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let prepared = fixture
        .world
        .runtime
        .prepare_estate_freeze_reapplication(
            &specialist,
            &scope,
            &alternative,
            "shared-codec-reapply",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("shared raw input is admitted through its selected freeze binding");
    };
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("the original source-completion action must complete");
    };
    assert_eq!(
        completed.original_input(),
        &fixture.action(fixture.estate_account)
    );
    let WorthQuerySourceCompletion::Fresh {
        receipt,
        retained: None,
        ..
    } = completed.evidence()
    else {
        panic!(
            "source-only reapplication carries fresh publication without retained-read delivery"
        );
    };
    let receipt = receipt.clone();
    let fact = receipt.aftermath_causality().unwrap();
    assert_eq!(fact.role(), WorthQueryAftermathCausalRole::RecordedReapply);
    assert_eq!(fact.recorded_source(), &source);
    assert_eq!(fact.correction_lineage().root(), &source);
    assert_eq!(
        fact.correction_lineage()
            .selected_alternative()
            .unwrap()
            .relational(),
        first_reverse.commit_reference()
    );
    assert_eq!(
        fact.correction_lineage()
            .active_reversal()
            .unwrap()
            .relational(),
        first_reverse.commit_reference()
    );
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
    assert!(completed.close(&request).is_ok());

    let replay = fixture
        .world
        .runtime
        .prepare_estate_freeze_reapplication(
            &specialist,
            &scope,
            &alternative,
            "shared-codec-reapply",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Continued(
        WorthQuerySourceOnlyAdvance::Completed(replayed),
    ) = replay
    else {
        panic!(
            "exact retry recovers the source owner before re-evaluating the already frozen account"
        );
    };
    assert_eq!(
        replayed.original_input(),
        &fixture.action(fixture.estate_account)
    );
    let WorthQuerySourceCompletion::Recovered {
        receipt: recovered,
        retained: None,
        ..
    } = replayed.evidence()
    else {
        panic!("the original shared-codec completion must retain recovered provenance");
    };
    assert_eq!(recovered.commit_reference(), receipt.commit_reference());
    assert_eq!(
        recovered.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert!(replayed.close(&request).is_ok());

    let second_reverse = restore_and_close(
        &fixture,
        &receipt.correction_source(),
        "shared-codec-restore-reapplication",
    );
    let historical = choose(&fixture, &source, &first_reverse);
    assert_ne!(
        first_reverse.commit_reference(),
        second_reverse.commit_reference()
    );
    fixture.revoke_freeze_permission();
    let refused = fixture.world.runtime.prepare_estate_freeze_reapplication(
        &specialist,
        &scope,
        &historical,
        "shared-codec-revoked",
        limits(),
    );
    assert!(
        matches!(refused, Err(WorthQueryApplicationCorrectionDenial::Mutation(ref denial))
        if matches!(denial.kind(), WorthQueryApplicationRequestMutationDenialKind::Authorization(WorthQueryOperationAuthorizationDenialKind::Refused(_)))),
        "historical raw input must pass the original source capability under the current grant"
    );
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
}

fn restore_and_close(
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

fn choose<'a>(
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

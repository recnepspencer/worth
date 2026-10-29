pub(crate) use worth_query_bank_worlds::facade::server::estate_operations::freeze_account::correction::*;

#[test]
fn closed_freeze_restores_the_recorded_status_and_exact_retry_preserves_its_receipt() {
    let fixture = revocable_freeze_world("recorded-freeze-restore");
    let source = freeze_and_close(&fixture);
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let selected = request
        .corrections()
        .select_recorded(&source, limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("restore-freeze")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("current capability admits the recorded inverse");
    };
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("recorded freeze restoration commits: {outcome:?}");
    };
    assert_eq!(
        receipt.aftermath_causality().unwrap().recorded_source(),
        &source
    );
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);

    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("restore-freeze")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = retry
    else {
        panic!("retry recovers the performed inverse before re-evaluating Frozen");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(
        replayed.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );

    // The canonical reversal already consumed this exact source occurrence.
    // A new command cannot run the inverse handler for it again.
    let moved = request
        .corrections()
        .reverse(&selected)
        .idempotency("another-restoration")
        .limits(limits())
        .prepare();
    match moved {
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceAlreadyReversed,
        )) => (),
        Err(cause) => panic!("restored source must retain its canonical reversal: {cause:?}"),
        Ok(_) => panic!("a different command cannot restore the same moved source again"),
    }
    assert_eq!(estate_account_status(&fixture), AccountStatus::Open);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
}

#[test]
fn selected_freeze_source_does_not_preserve_revoked_capability_authority() {
    let fixture = revocable_freeze_world("recorded-freeze-revoked");
    let source = freeze_and_close(&fixture);
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let selected = request
        .corrections()
        .select_recorded(&source, limits())
        .unwrap();
    fixture.revoke_freeze_permission();
    let refused = request
        .corrections()
        .reverse(&selected)
        .idempotency("revoked-restoration")
        .limits(limits())
        .prepare();
    assert!(
        matches!(refused, Err(WorthQueryApplicationCorrectionDenial::Mutation(ref denial))
        if matches!(denial.kind(), WorthQueryApplicationRequestMutationDenialKind::Authorization(WorthQueryOperationAuthorizationDenialKind::Refused(_)))),
        "selected historical truth cannot carry a revoked freeze grant into correction"
    );
    assert_eq!(estate_account_status(&fixture), AccountStatus::Frozen);
    assert_eq!(foreign_account_status(&fixture), AccountStatus::Open);
}

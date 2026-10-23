//! Failed publication preserves the original inverse, current admission and close owner.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted,
        WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
    },
};

#[test]
fn unpublished_correction_keeps_original_custody_through_refusal_recovery_and_close() {
    let world = fixture::World::new(9);
    let (source, _) = world.send(30, "custody-original");
    world.send(7, "custody-intervening");
    let before = world.activity(world.source);
    let destination_before = world.activity(world.destination);
    let scope = request_scope();
    let request = world.native.runtime.request(&world.teller, &scope);
    // Recovery custody and optional source-history retention have distinct owners.
    let selected_source = request
        .corrections()
        .select_recorded(&source, limits())
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source.coordinates(),
            "custody-inverse",
            limits(),
        )
        .unwrap()
    else {
        panic!("first correction prepares");
    };
    world.native.runtime.fail_next_durable_append_for_test();
    let WorthQuerySourceOnlyAdvance::Progress(operation) = prepared.publish().unwrap() else {
        panic!("real failed durable append must preserve inverse custody");
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(world.balances(), (63, 37));
    assert_eq!(world.activity(world.source), before);
    assert_eq!(world.activity(world.destination), destination_before);

    let unauthorized = world.native.runtime.request(&world.owner, &scope);
    let WorthQuerySourceOnlyAdvance::Refused {
        operation,
        denial:
            WorthQuerySourceOnlyDenial::Recovery(WorthQueryApplicationRecoveryRequestDenial::Request(
                WorthQueryApplicationRequestMutationDenial::Authorization(cause),
            )),
    } = operation.advance(&unauthorized, None)
    else {
        panic!("source author cannot acquire teller correction authority through recovery");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
        )
    );
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(world.activity(world.source), before);
    assert_eq!(world.activity(world.destination), destination_before);

    let WorthQuerySourceOnlyAdvance::Progress(operation) = operation.advance(&request, None) else {
        panic!("original correction owner performs native publication recovery");
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RecoveryPerformed
    );
    let WorthQuerySourceOnlyAdvance::Progress(operation) = operation.advance(&request, None) else {
        panic!("recovered correction resolves its original transformed key");
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RetainedDeliveryPending
    );
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request, None)
    else {
        panic!("original correction receipt supplies retained delivery");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: Some(retained),
        publication: Some(_),
        ..
    } = completed.evidence()
    else {
        panic!("native recovery evidence");
    };
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        receipt.aftermath_causality().unwrap().recorded_source(),
        Some(&source)
    );
    let recovered_commit = receipt.correction_source();
    assert_eq!(world.balances(), (93, 7));
    assert_eq!(world.activity(world.source).len(), before.len() + 1);
    assert_eq!(
        world.activity(world.destination).len(),
        destination_before.len() + 1
    );

    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    let cancelled_scope = WorthQueryRequestScope::new(None, cancel.token());
    let cancelled = world
        .native
        .runtime
        .request(&world.teller, &cancelled_scope);
    let (completed, denial) = completed
        .close(&cancelled)
        .err()
        .expect("cancelled close returns original owner");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    assert!(completed.close(&request).is_ok());

    let source_after = world.activity(world.source);
    let destination_after = world.activity(world.destination);
    let WorthQueryOperationCorrectionPreparation::Continued(WorthQuerySourceOnlyAdvance::Progress(
        replay,
    )) = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source.coordinates(),
            "custody-inverse",
            limits(),
        )
        .unwrap()
    else {
        panic!("same inverse identity replays recovered evidence");
    };
    let WorthQuerySourceOnlyAdvance::Completed(replay) = replay.advance(&request, None) else {
        panic!("retained replay");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: Some(retained),
        ..
    } = replay.evidence()
    else {
        panic!("replay delivers recovered correction evidence and its retained observation");
    };
    assert_eq!(receipt.correction_source(), recovered_commit);
    assert_eq!(
        receipt.aftermath_causality().unwrap().recorded_source(),
        Some(&source)
    );
    assert_eq!(retained.selected_commit(), recovered_commit.world());
    assert_eq!(world.activity(world.source), source_after);
    assert_eq!(world.activity(world.destination), destination_after);
    assert!(replay.close(&request).is_ok());
    drop(selected_source);
}

#[test]
fn stale_prepared_correction_is_terminal_and_cannot_retry_as_an_ordinary_reversal() {
    let world = fixture::World::new(9);
    let (source, _) = world.send(30, "stale-original");
    let scope = request_scope();
    let request = world.native.runtime.request(&world.teller, &scope);
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = world
        .native
        .runtime
        .prepare_send_money_correction(
            &world.teller,
            &scope,
            &source.coordinates(),
            "stale-inverse",
            limits(),
        )
        .unwrap()
    else {
        panic!("prepare at exact current target");
    };
    world.send(7, "stale-target-movement");
    let before = world.activity(world.source);
    let destination_before = world.activity(world.destination);
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = prepared.publish().unwrap() else {
        panic!("stale correction retains terminal custody without an ordinary retry");
    };
    assert!(
        matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause))) if cause.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale)
    );
    assert_eq!(world.balances(), (63, 37));
    assert_eq!(world.activity(world.source), before);
    assert_eq!(world.activity(world.destination), destination_before);
    assert!(terminal.close(&request).is_ok());
}

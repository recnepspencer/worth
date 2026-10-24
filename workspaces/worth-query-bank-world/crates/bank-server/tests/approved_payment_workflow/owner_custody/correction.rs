//! A real rail owner settles before an independently admitted journal compensation.
use super::super::{
    assertions::{key, require_completed},
    fixture,
    ready_payment::ReadyPaymentWorld,
};
use bank_domain::schema::PaymentStatus;
use bank_external_rail::test_control::FaultScript;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[path = "correction/fixture.rs"]
mod support;

fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN)
}

fn require_pending(
    result: Result<
        bank_server::BankApprovedPaymentCorrectionPreparation<'_>,
        WorthQueryApplicationCorrectionDenial,
    >,
) {
    let Err(denial) = result else {
        panic!("authentic unsettled custody must refuse before an inverse candidate");
    };
    assert!(
        matches!(&denial,
        WorthQueryApplicationCorrectionDenial::Derivation(WorthQueryCorrectionDerivationDenial::Workflow(cause))
            if cause.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody),
        "authentic unsettled custody must return its owning refusal: {denial:?}"
    );
}

#[test]
fn correction_waits_for_original_rail_owner_then_preserves_completed_settlement() {
    let fixture = fixture::ordinary_read_world_with_approval_authentication(
        "correction-original-rail-owner",
        super::super::authentication::approval_configuration(),
    );
    support::grant_teller_payment_disclosure(&fixture);
    let ready = ReadyPaymentWorld::from_fixture(fixture, FaultScript::CommitThenLoseResponse);
    let route = ready.install_synchronous_completion();
    let before_payment = support::balances(&ready);
    let runtime = &ready.fixture.world.runtime;
    let request = runtime
        .request(&ready.principal, &ready.scope)
        .on_branch(ready.instance.branch());
    let operation_key = key("approved-payment:operation:perform");
    let operation = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .retain_apply(
            &ready.operation,
            ready.authority.clone(),
            operation_key.clone(),
        )
        .unwrap();
    let mut operation = match operation.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(operation) => operation,
        _ => panic!("the lost real rail response must retain the original owner"),
    };
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let token = ready.rail.attempts()[0].token().try_into().unwrap();
    assert!(runtime
        .observe_payment_rail_completion(&route, token)
        .is_none());

    // Inspect the actual same-key receipt while retaining the original operation owner.
    let source = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .advanced()
        .execute_operation(
            ready.instance.clone(),
            &ready.operation,
            ready.authority.clone(),
            &operation_key,
        )
        .unwrap();
    let bank_server::BankApprovedPaymentApplyOutcome::Performed(source) = source else {
        panic!("same-key inspection must return the original committed occurrence");
    };
    assert!(!source.newly_committed());
    assert_eq!(
        ready.rail.attempts().len(),
        1,
        "receipt observation cannot redispatch"
    );
    let source_coordinates = source.correction_source();
    let teller = ready.fixture.authenticate(fixture::TELLER);
    let teller_request = runtime
        .request(&teller, &ready.scope)
        .on_branch(ready.instance.branch());
    // Keep the original optional history through later workflow commits and close.
    // The serialized coordinates and operation cleanup owner do not retain it.
    let selected_source = teller_request
        .corrections()
        .select_recorded(&source_coordinates, limits())
        .unwrap();
    let after_payment = support::balances(&ready);
    assert_ne!(after_payment, before_payment);
    let dispatches_before = ready.rail.attempts().len();
    require_pending(runtime.prepare_approved_payment_correction(
        &teller,
        &ready.scope,
        &source_coordinates.coordinates(),
        "approved-payment:journal-compensation",
        limits(),
    ));
    assert_eq!(support::balances(&ready), after_payment);
    assert_eq!(ready.rail.attempts().len(), dispatches_before);

    ready
        .rail
        .under(FaultScript::Succeed, Duration::from_millis(150));
    let completed = loop {
        match operation.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => operation = next,
            WorthQueryWorkflowOperationAdvance::Completed(completed) => break completed,
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("original real rail recovery refused: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                let _original_owner = terminal;
                panic!("original real rail recovery terminated");
            }
        }
    };
    let terminal = runtime
        .observe_payment_rail_completion(&route, token)
        .unwrap();
    assert_eq!(terminal.original_world_commit(), source_coordinates.world());
    // Completed inbound alone does not impersonate workflow settlement.
    require_pending(runtime.prepare_approved_payment_correction(
        &teller,
        &ready.scope,
        &source_coordinates.coordinates(),
        "approved-payment:journal-compensation",
        limits(),
    ));
    assert_eq!(support::balances(&ready), after_payment);
    let accepted = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .accept_owned_applied(
            ready.instance.clone(),
            ready.authority.clone(),
            completed,
            &key("approved-payment:correction:accept-owner"),
        )
        .unwrap_or_else(|(_owner, cause)| {
            panic!("authentic completed owner settlement: {cause:?}")
        });
    require_completed(accepted.progress, "apply");
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("original owner closes normally"));
    for (step, operation_key) in [
        (
            "await-inbound",
            "approved-payment:correction:consume-inbound",
        ),
        ("completed", "approved-payment:correction:end"),
    ] {
        require_completed(
            runtime
                .approved_business_payment(&ready.principal, &ready.scope)
                .advance(
                    ready.instance.clone(),
                    ready.authority.clone(),
                    &key(operation_key),
                )
                .unwrap(),
            step,
        );
    }

    let prepared = runtime
        .prepare_approved_payment_correction(
            &teller,
            &ready.scope,
            &source_coordinates.coordinates(),
            "approved-payment:journal-compensation",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("settled source prepares its installed journal compensation");
    };
    assert_eq!(support::balances(&ready), after_payment);
    let WorthQuerySourceOnlyAdvance::Completed(completed) = prepared.publish().unwrap() else {
        panic!("ordinary teller compensation publishes with retained delivery");
    };
    let WorthQuerySourceCompletion::Fresh {
        receipt: inverse,
        retained: Some(retained),
        ..
    } = completed.evidence()
    else {
        panic!("fresh compensation owns its exact retained publication");
    };
    let inverse = inverse.clone();
    assert_eq!(
        retained.selected_commit(),
        inverse.committed_product_publication().composite_commit()
    );
    assert_eq!(support::balances(&ready), before_payment);
    assert_eq!(
        inverse.aftermath_causality().unwrap().recorded_source(),
        &source_coordinates
    );
    assert!(completed.close(&teller_request).is_ok());
    let replay = runtime
        .prepare_approved_payment_correction(
            &teller,
            &ready.scope,
            &source_coordinates.coordinates(),
            "approved-payment:journal-compensation",
            limits(),
        )
        .unwrap();
    let WorthQueryOperationCorrectionPreparation::Continued(WorthQuerySourceOnlyAdvance::Progress(
        replay,
    )) = replay
    else {
        panic!("exact compensation retry retains the original inverse delivery");
    };
    let WorthQuerySourceOnlyAdvance::Completed(replay) = replay.advance(&teller_request, None)
    else {
        panic!("exact compensation retry delivers the original inverse");
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: Some(retained),
        ..
    } = replay.evidence()
    else {
        panic!("replay retains its original publication evidence");
    };
    assert_eq!(receipt.commit_reference(), inverse.commit_reference());
    assert_eq!(
        retained.selected_commit(),
        inverse.committed_product_publication().composite_commit()
    );
    assert!(replay.close(&teller_request).is_ok());
    drop(selected_source);
    assert_eq!(support::balances(&ready), before_payment);
    let retained = runtime
        .observe_payment_rail_completion(&route, token)
        .unwrap();
    assert_eq!(
        retained.original_world_commit(),
        terminal.original_world_commit()
    );
    assert_eq!(
        retained.completion_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(retained.completion_attempt(), terminal.completion_attempt());
    assert_eq!(ready.rail.attempts().len(), 2);
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let payment = request
        .query(bank_domain::queries::payment(ready.fixture.payment))
        .execute()
        .unwrap();
    assert_eq!(payment.rows()[0].status(), PaymentStatus::Committed);
    assert_eq!(
        payment.rows()[0].deciding_principal(),
        Some(fixture::principal_id(fixture::APPROVER))
    );
    let cancelled = super::cancel(&ready, "approved-payment:correction:cancel-completed");
    assert!(matches!(cancelled,
        Err(bank_server::BankApprovedPaymentWorkflowError::InstanceCancellation(
            WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(WorkflowInstancePreparationDenial::Attempt(ref denial))))
                if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted));
}

//! Original Bank workflow custody crosses request frames without retaining authorization.
use super::{assertions::key, ready_payment::ReadyPaymentWorld, support};
use bank_external_rail::test_control::FaultScript;
use worth_query_host::facade::{admission::authenticated_principal::*, application_entry::*};
#[test]
fn owning_bank_payment_refuses_cancellation_then_commits_once() {
    let ready = ReadyPaymentWorld::new("owning-workflow-payment", FaultScript::Succeed);
    let runtime = &ready.fixture.world.runtime;
    let owner = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .retain_apply(
            &ready.operation,
            ready.authority.clone(),
            key("approved-payment:owned:perform"),
        )
        .expect("actual declared Bank action");
    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    let stopped = WorthQueryRequestScope::new(None, cancel.token());
    let owner = match owner.advance(
        &runtime
            .request(&ready.principal, &stopped)
            .on_branch(ready.instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, .. } => operation,
        _ => panic!("pre-effect cancellation retains original slot"),
    };
    assert!(ready.rail.attempts().is_empty());
    let fresh = support::request_scope();
    let completed = match owner.advance(
        &runtime
            .request(&ready.principal, &fresh)
            .on_branch(ready.instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Completed(done) => done,
        _ => panic!("lawful fresh Bank effect"),
    };
    assert!(
        matches!(completed.evidence(),WorthQuerySourceCompletion::Fresh{receipt,..} if receipt.emitted_effect_count()==1)
    );
    assert_eq!(ready.rail.completed_effect_count(), 1);
    completed
        .close(
            &runtime
                .request(&ready.principal, &fresh)
                .on_branch(ready.instance.branch()),
        )
        .unwrap_or_else(|_| panic!("real native close"));
}

#[test]
fn owning_bank_payment_retains_original_outbox_handle_through_fresh_refusal() {
    let ready = ReadyPaymentWorld::new("owning-workflow-outbox", FaultScript::DisappearMidDispatch);
    let runtime = &ready.fixture.world.runtime;
    let owner = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .retain_apply(
            &ready.operation,
            ready.authority.clone(),
            key("approved-payment:owned:outbox"),
        )
        .expect("actual original operation");
    let request = runtime
        .request(&ready.principal, &ready.scope)
        .on_branch(ready.instance.branch());
    let owner = match owner.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(owner) => owner,
        _ => panic!("lost dispatch retains native outbox custody"),
    };
    assert_eq!(ready.rail.attempts().len(), 1);
    assert_eq!(ready.rail.completed_effect_count(), 0);
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let owner = match owner.advance(
        &runtime
            .request(&ready.principal, &stopped)
            .on_branch(ready.instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Refused { operation, .. } => operation,
        _ => panic!("opening refusal retains original handle before taking it"),
    };
    assert_eq!(ready.rail.attempts().len(), 1);
    ready
        .rail
        .under(FaultScript::Succeed, std::time::Duration::from_millis(150));
    let fresh = support::request_scope();
    let completed = match owner.advance(
        &runtime
            .request(&ready.principal, &fresh)
            .on_branch(ready.instance.branch()),
    ) {
        WorthQueryWorkflowOperationAdvance::Completed(done) => done,
        _ => panic!("fresh native redispatch consumes original handle"),
    };
    assert_eq!(ready.rail.attempts().len(), 2);
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let dispatches = ready.rail.production_dispatches();
    assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
    assert_eq!(dispatches[0].payload, dispatches[1].payload);
    let (completed, _) = runtime
        .approved_business_payment(&ready.principal, &fresh)
        .accept_owned_applied(
            ready.instance.clone(),
            ready.authority.clone(),
            completed,
            &key("approved-payment:owned:accept-without-inbound"),
        )
        .err()
        .expect("rail-local completion is not authenticated inbound workflow completion");
    assert!(matches!(
        runtime
            .approved_business_payment(&ready.principal, &fresh)
            .advance(
                ready.instance.clone(),
                ready.authority.clone(),
                &key("approved-payment:owned:still-awaiting")
            )
            .unwrap(),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
    completed
        .close(
            &runtime
                .request(&ready.principal, &fresh)
                .on_branch(ready.instance.branch()),
        )
        .unwrap_or_else(|_| panic!("close actual effect cleanup, not workflow settlement"));
}

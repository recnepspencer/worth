//! Limits count real starts, and refusing a later start rolls the complete edit back.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::ApplicationProducerRoundLimits;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

fn budget(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let original = (
        world::model(&host, host.current_world()),
        world::outputs(&host, host.current_world()),
    );
    let before = failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, publications) = observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .producer_round_limits(ApplicationProducerRoundLimits::new(
                NonZeroUsize::new(64).unwrap(),
                NonZeroUsize::MIN,
            ))
            .without_source()
            .idempotency(&"bounded-two-family-edit".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        refusal,
    )) = outcome
    else {
        panic!("the second actual start must refuse the whole edit: {outcome:?}")
    };
    let cause = refusal
        .coupled_producer_denial()
        .expect("the original producer refusal");
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerAttemptRoundBudgetExceeded {
            source: original.0.native_id,
            started: 1,
            limit: 1,
        }
    );
    assert!(cause.subject().contains("owned-outputs."), "{cause:?}");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "no handler contact is permitted after the one admitted start"
    );
    assert_eq!(
        publications.published_rows.len() + publications.omitted_publications,
        0
    );
    assert_eq!(
        (
            world::model(&host, host.current_world()),
            world::outputs(&host, host.current_world())
        ),
        original
    );
    failure::assert_model_unchanged(&host, &before);

    let before = host.producer_contacts_on_this_thread_for_test();
    let (receipt, publications) = observe_maintenance_work(8, || {
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"unrestricted-two-family-edit".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        outcome.receipt().unwrap().commit_reference().clone()
    });
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        before + 2,
        "explicit coupled writers still run once each"
    );
    assert_eq!(
        publications.published_rows.len() + publications.omitted_publications,
        1
    );
    let outputs = world::outputs(&host, host.current_world());
    assert_eq!((outputs.0.value, outputs.1.value), (20, 1002));
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (replay, publications) = observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .producer_round_limits(ApplicationProducerRoundLimits::new(
                NonZeroUsize::MIN,
                NonZeroUsize::MIN,
            ))
            .idempotency(&"unrestricted-two-family-edit".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    assert!(matches!(
        replay,
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)
    ));
    assert_eq!(
        replay.recovered_receipt().unwrap().commit_reference(),
        &receipt
    );
    assert_eq!(
        publications.published_rows.len() + publications.omitted_publications,
        0
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(world::outputs(&host, host.current_world()), outputs);
}
#[test]
fn an_attempt_round_limit_refuses_before_the_second_handler_without_effect() {
    budget(false);
}
#[test]
fn worker_only_attempt_round_limit_refuses_before_the_second_handler_without_effect() {
    budget(true);
}

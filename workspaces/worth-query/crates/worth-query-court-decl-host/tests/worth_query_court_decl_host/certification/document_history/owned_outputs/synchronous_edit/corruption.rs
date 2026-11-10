//! Corruption changes a real prepared writer; the ordinary Native fence refuses it.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};

fn refuses_foreign_writer(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    let source = world::model(host, host.current_world());
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = failure::checkpoint(host);
    let _fault = host
        .runtime()
        .corrupt_next_coupled_participant_for_test(true);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"corrupt-synchronous-edit".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a foreign publication must refuse the entire edit: {outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("the Native publication refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::ForeignCoupledOwner)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the actual first writer ran"
    );
    failure::assert_model_unchanged(host, &before);
}

#[test]
fn a_foreign_prepared_writer_cannot_publish_the_source_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_foreign_writer(&world::install());
}
#[test]
fn worker_only_foreign_prepared_writer_cannot_publish_the_source_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_foreign_writer(&world::install_worker_only());
}

fn replayed_continuation(worker: bool) {
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
    let source = world::model(&host, host.current_world());
    let before = failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let _fault = host.runtime().replay_next_coupled_continuation_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"replayed-real-continuation".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a consumed Native continuation must refuse before publication")
    };
    let error = denial
        .authored_retirement_error()
        .expect("Native continuation refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::StaleCoupledContinuation)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_real_native_continuation_is_consumed_exactly_once() {
    replayed_continuation(false);
}
#[test]
fn worker_only_real_native_continuation_is_consumed_exactly_once() {
    replayed_continuation(true);
}

/// A continued round with an effect would be the route's second producing round.
fn continued_effect_refuses(worker: bool, changed_patch: bool) {
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
    let source = world::model(&host, host.current_world());
    let before = failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let _fault = if changed_patch {
        host.runtime()
            .continue_next_coupled_route_with_a_changed_patch_for_test()
    } else {
        host.runtime()
            .continue_next_coupled_route_with_its_effect_for_test()
    };
    let (outcome, work) = worth_relational::facade::inspection::observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"continued-effectful-round".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a second effectful round of one route must refuse before publication")
    };
    let error = denial
        .authored_retirement_error()
        .expect("Native continued-round refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::CoupledContinuationProvenancePostponed { issue: 119 })
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "only the actual first writer ran"
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_continued_round_with_an_effect_refuses_before_publication() {
    continued_effect_refuses(false, false);
}
#[test]
fn worker_only_continued_round_with_an_effect_refuses_before_publication() {
    continued_effect_refuses(true, false);
}
#[test]
fn a_continued_round_whose_patch_changes_a_value_refuses_before_publication() {
    continued_effect_refuses(false, true);
}
#[test]
fn worker_only_continued_round_whose_patch_changes_a_value_refuses_before_publication() {
    continued_effect_refuses(true, true);
}

/// A continued round that applies nothing leaves the candidate exactly as it was.
fn continued_unchanged_round_is_invisible(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let edited = |continued: bool| {
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
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let _fault = continued.then(|| {
            host.runtime()
                .continue_next_coupled_route_unchanged_for_test()
        });
        let (outcome, work) =
            worth_relational::facade::inspection::observe_maintenance_work(8, || {
                request
                    .advanced()
                    .mutate(Intent {
                        key: "a".into(),
                        value: 2,
                    })
                    .without_source()
                    .idempotency(&"continued-unchanged-round".to_owned())
                    .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap()
            });
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "an unchanged continued round must not refuse the edit: {outcome:?}"
        );
        (
            host.producer_contacts_on_this_thread_for_test() - contacts,
            work.published_rows.len() + work.omitted_publications,
            world::model(&host, host.current_world()),
            world::outputs(&host, host.current_world()),
            courts::native_head(&host),
        )
    };
    let plain = edited(false);
    let continued = edited(true);
    assert_eq!(plain.0, 2);
    assert_eq!(plain.1, 1);
    assert_eq!(
        continued, plain,
        "contacts, the one publication, every row and the Native head match the edit without a continued round"
    );
}
#[test]
fn a_continued_round_that_applies_nothing_leaves_the_edit_unchanged() {
    continued_unchanged_round_is_invisible(false);
}
#[test]
fn worker_only_continued_round_that_applies_nothing_leaves_the_edit_unchanged() {
    continued_unchanged_round_is_invisible(true);
}

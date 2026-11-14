//! First publications and later Preserves are synchronous consequences of the edit.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let initial = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"initial-synchronous-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            initial,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{initial:?}"
    );
    let first_head = courts::native_head(&host);
    assert_eq!(
        first_head.1,
        [before.0],
        "the source and two Initials publish once"
    );
    assert_eq!(
        first_head.0,
        initial.receipt().unwrap().commit_reference().commit_id.0
    );
    let original = world::outputs(&host, host.current_world());
    assert_eq!((original.0.value, original.1.value), (20, 1002));
    assert_eq!(world::model(&host, host.current_world()).value, 2);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let preserve = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"preserve-after-synchronous-initial".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            preserve,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{preserve:?}"
    );
    let head = courts::native_head(&host);
    assert_eq!(
        head.1,
        [first_head.0],
        "the two Preserves add one Native step"
    );
    let outputs = world::outputs(&host, host.current_world());
    assert_eq!((outputs.0.value, outputs.1.value), (30, 1003));
    assert_eq!(
        (outputs.0.native_id, outputs.1.native_id),
        (original.0.native_id, original.1.native_id)
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    assert_eq!(courts::native_head(&host), head, "readiness adds no commit");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    let replay = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"initial-synchronous-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::RecoveredCommitted(replayed) = replay else {
        panic!("{replay:?}")
    };
    assert!(initial
        .receipt()
        .unwrap()
        .matches_recorded_original(&replayed));
    assert_eq!(
        replayed.commit_reference(),
        initial.receipt().unwrap().commit_reference()
    );
    assert_eq!(world::model(&host, host.current_world()).value, 3);
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
}
#[test]
fn initial_families_then_preserves_each_share_the_authored_commit() {
    run(false);
}
#[test]
fn worker_only_initial_families_then_preserves_each_share_the_authored_commit() {
    run(true);
}

fn refused_initial(value: u64) -> u64 {
    if value == 2 {
        u64::MAX
    } else {
        second::derived(value)
    }
}
fn failure(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Program>(
        limits,
        1,
        None,
        world::Configuration {
            reconciliation_indices: false,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            envelope_records: false,
            second_derivation: Some(refused_initial),
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"refused-initial-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!(
            "an Initial refusal cannot publish: receipt {:?}",
            outcome.receipt().map(|receipt| receipt.commit_reference())
        );
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the real Initial denial");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert!(
        producer.subject().contains("owned-outputs.second.initial"),
        "{producer:?}"
    );
    assert_eq!(producer.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the first Initial really executed before the second refused"
    );
    assert_eq!(courts::native_head(&host), head);
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn an_initial_refusal_discards_the_source_and_the_earlier_initial() {
    failure(false);
}
#[test]
fn worker_only_initial_refusal_discards_the_source_and_the_earlier_initial() {
    failure(true);
}

fn cold(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let (checkpoint, outputs) = {
        let host = if worker {
            world::install_worker_only()
        } else {
            world::install()
        };
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"cold-initial-edit".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
        (
            failure::checkpoint(&host),
            world::outputs(&host, host.current_world()),
        )
    };
    let host = world::restore_program_checkpoint::<program::Program>(checkpoint, worker, false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(world::outputs(&host, host.current_world()), outputs);
    assert_eq!(world::model(&host, host.current_world()).value, 2);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn coupled_initials_reopen_with_equal_custody_and_contact_free_readiness() {
    cold(false);
}
#[test]
fn worker_only_coupled_initials_reopen_with_equal_custody_and_contact_free_readiness() {
    cold(true);
}

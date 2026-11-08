//! A generic entry cannot drop an installed required tree or hide its old receipt.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let bare = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"required-source-gate".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        cause,
    )) = bare
    else {
        panic!("an entry without the real required handoff must refuse")
    };
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationCommitDenialKind::RequiredOutputPreparationMissing
    );
    let detail = cause
        .detail()
        .expect("the refusal must name the actual missing required-source handoff");
    assert!(
        detail.contains("owned-outputs.edit-model.v1") && detail.contains("required-output"),
        "{cause:?}"
    );
    let performed = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"required-source-gate".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = performed else {
        panic!("the same fresh key and real handoff must perform")
    };
    assert_eq!(courts::native_head(&host).1, [head.0]);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        30
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1003
    );
    let complete = courts::native_head(&host);
    let replay = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"required-source-gate".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) = replay else {
        panic!("the exact recorded receipt precedes missing fresh preparation")
    };
    assert_eq!(
        receipt.commit_reference(),
        done.receipt().commit_reference()
    );
    let drift = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 4,
        })
        .without_source()
        .idempotency(&"required-source-gate".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        drift,
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
    ));
    assert_eq!(courts::native_head(&host), complete);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let (_, _, mut outputs) = done
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            outputs.advance(&host, &request).unwrap()
        {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(courts::native_head(&host), complete);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
}
#[test]
fn a_required_source_needs_its_real_handoff_after_recorded_replay() {
    exercise(false)
}
#[test]
fn worker_only_a_required_source_needs_its_real_handoff_after_recorded_replay() {
    exercise(true)
}

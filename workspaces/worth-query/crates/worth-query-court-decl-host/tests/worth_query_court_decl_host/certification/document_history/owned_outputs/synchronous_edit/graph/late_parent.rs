//! A later contribution cannot invalidate a verified parent and still publish the edit.
use super::*;
mod program;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_other_model::<program::Program>(limits, 2, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let initial_contacts = host.producer_contacts_on_this_thread_for_test();
    let first = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 4,
        })
        .without_source()
        .idempotency(&"late-parent-initial".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = first else {
        panic!("same-value child write must preserve the initial parent support")
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        initial_contacts + 2
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
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        initial_contacts + 2
    );
    drop(outputs);
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        7
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1004
    );
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 8,
        })
        .without_source()
        .idempotency(&"late-parent-refused".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("changed verified-parent support must refuse the whole edit")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("the final candidate read-support owner refuses");
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert!(
        failure.detail().contains("decision-input")
            && failure.detail().contains("verified output support")
            && failure.detail().contains("MemberValue"),
        "{failure:?}"
    );
    let source = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    assert_eq!(
        failure.affected_records(),
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the healthy original parent stays unopened"
    );
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_later_child_cannot_invalidate_the_verified_parent_before_publication() {
    exercise(false)
}
#[test]
fn worker_only_a_later_child_cannot_invalidate_the_verified_parent_before_publication() {
    exercise(true)
}

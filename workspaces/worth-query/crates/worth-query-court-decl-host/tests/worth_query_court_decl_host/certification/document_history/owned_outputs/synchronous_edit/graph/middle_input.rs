//! A parent that ran without effect still refuses when a later writer moves its input.
//! Publishing that journey needs same-route receipt replacement, postponed by D1483.
use super::*;
mod program;
use worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind as Kind;
use worth_relational::facade::{inspection::observe_maintenance_work, transactions::RecordRef};
fn limits(worker: bool) -> WorthQueryApplicationLimits {
    if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    }
}
fn parity(request: &family::Request<'_, '_, '_>) -> query::Row {
    request
        .query(query::first_output::Read("parity:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn source_edit(label: &str, value: u64) -> (edit::Intent, String) {
    (
        edit::Intent {
            key: "a".into(),
            value,
        },
        format!("middle-input-{label}-{value}"),
    )
}

/// The parent's second run is a real execution whose output does not change.
fn parent_alone_runs_without_effect(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_value::<program::Alone>(limits(worker), 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut born = None;
    for (value, label) in [(4, "birth"), (6, "unchanged")] {
        let (intent, key) = source_edit(label, value);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (outcome, work) = observe_maintenance_work(8, || {
            request
                .advanced()
                .mutate(intent)
                .without_source()
                .idempotency(&key)
                .execute_performed::<program::Alone, program::AloneTree>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ),
            "the parent alone must perform its {label} edit"
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 1,
            "the changed source runs the real parent handler"
        );
        assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
        let row = parity(&request);
        assert_eq!(row.value, 5, "an even source keeps parity 0 plus member 5");
        assert_eq!(*born.get_or_insert(row.native_id), row.native_id);
    }
}
#[test]
fn a_parity_parent_alone_executes_again_without_changing_its_output() {
    parent_alone_runs_without_effect(false)
}
#[test]
fn worker_only_parity_parent_alone_executes_again_without_changing_its_output() {
    parent_alone_runs_without_effect(true)
}

fn later_writer_refuses(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_value::<program::Program>(limits(worker), 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (intent, key) = source_edit("birth", 4);
    let first = request
        .advanced()
        .mutate(intent)
        .without_source()
        .idempotency(&key)
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = first else {
        panic!("the child's same-value member write must preserve the parent's support")
    };
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
    drop(outputs);
    assert_eq!(parity(&request).value, 5);
    let source = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (intent, key) = source_edit("moved", 6);
    let (outcome, work) = observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(intent)
            .without_source()
            .idempotency(&key)
            .execute_performed::<program::Program, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("a later writer moving the unchanged parent's input must refuse the whole edit")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("the final candidate decision-support owner refuses");
    assert_eq!(failure.kind(), Kind::CoupledReadSupportChanged);
    assert!(
        failure.detail().contains("decision support") && failure.detail().contains("MemberValue"),
        "{failure:?}"
    );
    assert_eq!(failure.affected_records(), [RecordRef::Entity(source)]);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the parent ran without effect and the child ran before the refusal"
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
    assert_eq!(parity(&request).value, 5);
}
#[test]
fn a_later_writer_cannot_move_the_input_of_a_parent_that_ran_without_effect() {
    later_writer_refuses(false)
}
#[test]
fn worker_only_later_writer_cannot_move_the_input_of_a_parent_that_ran_without_effect() {
    later_writer_refuses(true)
}

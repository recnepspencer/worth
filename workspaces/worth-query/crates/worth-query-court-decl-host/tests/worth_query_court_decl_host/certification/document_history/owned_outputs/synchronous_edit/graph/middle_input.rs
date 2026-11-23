//! A parent that ran without effect, whose input a later writer moves, reopens
//! and re-derives (PR C §1.5): its successor round folds into its chain, so the
//! route publishes one receipt with the re-derived value.
use super::*;
mod program;
use worth_relational::facade::inspection::observe_maintenance_work;
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

/// The sequential oracle: `parity:a` derived on demand from the committed
/// world settles as already current, with no producer contact.
fn settles_current(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Program>,
    request: &family::Request<'_, '_, '_>,
) {
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut operation = request
        .demand(family::parity_input::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for _ in 0..128 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("{denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    panic!("on-demand parity:a did not settle")
}

fn later_writer_rederives(worker: bool) {
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
    let born = parity(&request).native_id;
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
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the moved input re-derives the parent and the edit performs"
    );
    // The parent ran without effect (parity 0 + m 5), the child wrote m = 6 + 1,
    // and the parent's successor round re-derived parity 0 + 7.
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3,
        "the parent, the child, then the parent's successor round"
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
    assert_ne!(courts::native_head(&host), head);
    let row = parity(&request);
    assert_eq!(row.value, 7);
    assert_eq!(row.native_id, born, "the chain re-derives the same output");
    settles_current(&host, &request);
    assert_eq!(parity(&request).value, 7);
}
#[test]
fn a_later_writer_moving_the_input_of_a_parent_that_ran_without_effect_rederives_it() {
    later_writer_rederives(false)
}
#[test]
fn worker_only_later_writer_moving_the_input_of_a_parent_that_ran_without_effect_rederives_it() {
    later_writer_rederives(true)
}

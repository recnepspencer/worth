//! A later contribution cannot invalidate a verified parent and still publish the edit.
use super::*;
mod derived;
mod program;
#[macro_use]
mod staged;
mod alias;
mod cross_write;
mod outgoing;
mod prerequisite;
mod pending_refusal;
mod pending_retry;
mod prior_effect;
mod selection;
mod selection_swap;
mod settlement;
mod shared_read;
mod superseded_window;
mod unowned_reader;
/// Commits `b = 4` and settles its outputs: `first:a` is then a verified
/// parent derived from model `a` and member `m`, and `second:b` its child.
fn settle_initial(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Program>,
    request: &family::Request<'_, '_, '_>,
) {
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
            host,
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
        .start_required_outputs(host, request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            outputs.advance(host, request).unwrap()
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
}
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
    settle_initial(&host, &request);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 8,
        })
        .without_source()
        .idempotency(&"late-parent-rederived".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(_) = outcome else {
        panic!(
            "the stale verified parent is derived again in the edit: {:?}",
            _execution.wave_events_for_test()
        )
    };
    // `decision-write(b)` writes m = b + 1 = 9; the verified parent read m = 5,
    // so qualified use derives it again from m = 9 (D3): 2 + 9.
    assert_rederived_first(&request);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the child derives, and the stale parent derives again"
    );
    assert_ne!(courts::native_head(&host), head);
}
/// `first:a` committed derived from m = 9: model `a` 2 plus m 9.
fn assert_rederived_first(request: &family::Request<'_, '_, '_>) {
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2 + 9
    );
}
/// A later child stales the verified parent it was selected through. Selection
/// is not consumption, so qualified use reopens the parent through normal
/// admission and the edit commits it derived from the settled member (Q1, D3).
#[test]
fn a_later_child_stales_the_verified_parent_and_it_derives_again() {
    exercise(false)
}
#[test]
fn worker_only_a_later_child_stales_the_verified_parent_and_it_derives_again() {
    exercise(true)
}
/// The same change as one authored stage of an ordered edit (#244): the
/// stage's child `decision-write(b)` changes `m`, which the verified parent
/// `first:a` read. That child is selected by a discovery that reads `first:a`,
/// but it has no prerequisite supplier and derives from model `b` alone.
/// Selection is not consumption, so qualified use reopens the stale verified
/// parent and the edit commits it derived from m = 9, the value one commit per
/// stage reads on demand (Q1, D3). The consuming variant is `prerequisite`.
#[test]
fn an_authored_stage_whose_discovered_child_stales_its_verified_parent_derives_it_again() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_initial(&host, &request);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-own-producer".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_rederived_first(&request);
    assert_ne!(courts::native_head(&host), head);
}

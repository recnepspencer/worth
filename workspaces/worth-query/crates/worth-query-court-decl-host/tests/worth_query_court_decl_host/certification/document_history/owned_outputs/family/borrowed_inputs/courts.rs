//! Literal borrowed values distinguish dependency custody from owned-edge reach.
use super::super::super::synchronous_edit::{
    self, envelope_fixture as fixture, interleaved::created,
};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};

type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;

fn first(request: &Request<'_, '_, '_>) -> query::Row {
    request
        .query(query::first_output::Read("borrowed:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn child(request: &Request<'_, '_, '_>, key: &str) -> query::Row {
    request
        .query(query::child_output::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn install() -> Host {
    world::install_program(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
    )
}
fn initialize(host: &Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut create = request
        .edit_envelope()
        .idempotency("borrowed-add-n".to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    create
        .push_stage::<ApplicationNoOutputGraph, _>(created::Intent {
            key: "n".into(),
            previous: None,
            value: 7,
        })
        .unwrap();
    let outcome = create
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    let mut edit = request
        .edit_envelope()
        .idempotency("borrowed-initialize".to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::model::Intent {
        key: "a".into(),
        value: 1,
        key_derived_target: false,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(first(&request).value, 1201);
    assert_eq!(child(&request, "from-first:borrowed:a").value, 120100);
    assert_eq!(child(&request, "downstream:m").value, 500);
    assert_eq!(child(&request, "downstream:n").value, 700);
}

fn exercise(cold: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    initialize(&host);
    let host = if cold {
        let checkpoint = synchronous_edit::checkpoint(&host);
        drop(host);
        world::restore_program_checkpoint::<program::Program>(checkpoint, false, false)
    } else {
        host
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = first(&request);
    let downstream = child(&request, "from-first:borrowed:a");
    let unaffected = child(&request, "downstream:n");
    let model = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let members = request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap();
    let roots = [
        ("model", model),
        (
            "m",
            members.rows()[0]
                .members
                .iter()
                .find(|row| row.key == "m")
                .unwrap()
                .native_id,
        ),
        (
            "n",
            members.rows()[0]
                .members
                .iter()
                .find(|row| row.key == "n")
                .unwrap()
                .native_id,
        ),
        ("header", before.native_id),
    ];
    let root_contacts =
        roots.map(|(_, root)| host.producer_contacts_at_root_on_this_thread_for_test(root));
    let head = synchronous_edit::native_head(&host).0;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut retire = request
        .edit_envelope()
        .idempotency("borrowed-retire-m".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    retire
        .push_stage::<program::Tree, _>(authored_member::retire::Intent)
        .unwrap();
    let (outcome, native_work) =
        worth_relational::facade::inspection::observe_maintenance_work(128, || {
            retire
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
    let Outcome::Committed(done) = outcome else {
        panic!("{outcome:?}")
    };
    let receipt = done.receipt().clone();
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        2,
        "only the collector and its transitive consumer produce; n is a current read; values=({}, {}); roots={:?}; symbol_rounds={:?}; omitted_events={}",
        first(&request).value,
        child(&request, "from-first:borrowed:a").value,
        roots.iter().zip(root_contacts).map(|((name, root), before)| (*name, before, host.producer_contacts_at_root_on_this_thread_for_test(*root))).collect::<Vec<_>>(),
        native_work.symbol_preparation,
        native_work.omitted_symbol_preparation_events,
    );
    assert_eq!(synchronous_edit::native_head(&host).1, [head]);
    assert_eq!(
        (first(&request).native_id, first(&request).value),
        (before.native_id, 701)
    );
    assert_eq!(
        (
            child(&request, "from-first:borrowed:a").native_id,
            child(&request, "from-first:borrowed:a").value
        ),
        (downstream.native_id, 70100)
    );
    assert_eq!(child(&request, "downstream:n"), unaffected);
    assert_eq!(
        request
            .query(query::members::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .members
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        ["n"]
    );

    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request.corrections().select(&receipt, limits).unwrap();
    let undo = request
        .corrections()
        .reverse(&source)
        .idempotency("borrowed-undo")
        .limits(limits)
        .edit_envelope::<authored_member::correction::UndoRetire, program::Program, program::Tree>(
            &host,
        )
        .unwrap()
        .ordered_authored()
        .unwrap();
    let outcome = undo
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        (first(&request).native_id, first(&request).value),
        (before.native_id, 1201)
    );
    assert_eq!(
        (
            child(&request, "from-first:borrowed:a").native_id,
            child(&request, "from-first:borrowed:a").value
        ),
        (downstream.native_id, 120100)
    );
    assert_eq!(child(&request, "downstream:n"), unaffected);
    let alternatives = request
        .corrections()
        .alternatives(&source, limits, NonZeroUsize::MIN)
        .unwrap();
    let alternative = alternatives
        .select(&alternatives.entries()[0], limits)
        .unwrap();
    let reapply = request
        .corrections()
        .reapply(&alternative)
        .idempotency("borrowed-reapply")
        .limits(limits)
        .edit_envelope::<authored_member::correction::UndoRetire, program::Program, program::Tree>(
            &host,
        )
        .unwrap()
        .ordered_authored()
        .unwrap();
    let outcome = reapply
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        (first(&request).native_id, first(&request).value),
        (before.native_id, 701)
    );
    assert_eq!(child(&request, "from-first:borrowed:a").value, 70100);
    assert_eq!(child(&request, "downstream:n"), unaffected);
}

#[test]
fn retirement_rebuilds_a_borrowed_only_owner_and_its_transitive_consumer() {
    exercise(false);
}
#[test]
fn cold_retirement_rebuilds_borrowed_owners_and_retains_recorded_reverse_and_reapply() {
    exercise(true);
}

#[test]
fn a_removed_live_input_has_no_surviving_consumer_when_it_is_later_retired() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    initialize(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = first(&request);
    let next = child(&request, "from-first:borrowed:a");
    let m = child(&request, "downstream:m");
    let n = child(&request, "downstream:n");
    let mut disconnect = request
        .edit_envelope()
        .idempotency("disconnect-live-input".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    disconnect
        .push_stage::<program::Tree, _>(disconnect::Intent)
        .unwrap();
    let outcome = disconnect
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        (first(&request).native_id, first(&request).value),
        (before.native_id, 701)
    );
    assert_eq!(
        (
            child(&request, "from-first:borrowed:a").native_id,
            child(&request, "from-first:borrowed:a").value
        ),
        (next.native_id, 70100)
    );
    assert_eq!(
        child(&request, "downstream:m"),
        m,
        "disconnection leaves the real parent publication live"
    );
    assert_eq!(child(&request, "downstream:n"), n);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let head = synchronous_edit::native_head(&host).0;
    let mut retire = request
        .edit_envelope()
        .idempotency("retire-unconsumed-input".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    retire
        .push_stage::<program::Tree, _>(authored_member::retire::Intent)
        .unwrap();
    let outcome = retire
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "no surviving consumer is enrolled from the removed input"
    );
    assert_eq!(synchronous_edit::native_head(&host).1, [head]);
    assert!(request
        .query(query::child_output::Read("downstream:m".into()))
        .execute()
        .is_err());
    assert_eq!(
        (first(&request).native_id, first(&request).value),
        (before.native_id, 701)
    );
    assert_eq!(
        (
            child(&request, "from-first:borrowed:a").native_id,
            child(&request, "from-first:borrowed:a").value
        ),
        (next.native_id, 70100)
    );
    assert_eq!(child(&request, "downstream:n"), n);
}

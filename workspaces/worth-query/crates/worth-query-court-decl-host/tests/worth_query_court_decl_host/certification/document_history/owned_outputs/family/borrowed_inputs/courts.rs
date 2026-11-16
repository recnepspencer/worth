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
    world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() },
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
    let outcome = retire
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Committed(done) = outcome else {
        panic!("{outcome:?}")
    };
    let receipt = done.receipt().clone();
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        2,
        "only the collector and its transitive consumer produce; n is a current read"
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

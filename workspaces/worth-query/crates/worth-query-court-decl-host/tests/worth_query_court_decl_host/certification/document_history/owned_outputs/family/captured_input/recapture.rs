//! Superseded planning must neither run an orphan nor lose another real edge.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};

#[test]
fn recapture_replaces_an_unconsumed_parent_with_the_new_actual_choice() {
    switch_after_parent(false);
}
#[test]
fn recapture_keeps_a_parent_still_required_by_another_input() {
    switch_after_parent(true);
}

/// A captured choice that drops a parent already derived in this edit (PR C
/// C4 r4, blocker 1, path B). `choice` (weight 1) drains on the seed port
/// before the control `a`, so its first choice `late` derives while the
/// child still waits on `a`. `a`'s append flips the choice to `d`: the
/// refresh clears the child's capture before any reselection runs. The
/// derived `late` is a selection it made, so its re-derivation, which no
/// longer selects `late`, refuses `SupersededSelectionDropped`. Red before
/// the fix: clearing forgot `late`, and the edit committed it as an orphan.
#[test]
fn a_recapture_that_drops_a_derived_parent_refuses() {
    super::program::CHOICE_FIRST.with(|flag| flag.set(true));
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, captured_inputs: true, captured_parent_switch: Some(false), ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("capture-derived-switch".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::model::Intent {
        key: "capture:child".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains("SupersededSelectionDropped"),
        "the derived first choice was dropped: {detail}; {events:#?}"
    );
    assert!(request
        .query(query::first_output::Read("first:carriage:late".into()))
        .execute()
        .is_err());
}
fn switch_after_parent(retain_old: bool) {
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, captured_inputs: true, captured_parent_switch: Some(retain_old), ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("capture-parent-switch".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::model::Intent {
        key: "capture:child".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    let output = request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .unwrap();
    // Once a's genuine output exists, the actual choice changes from late to d.
    // Source3 + zero-weight control a + 2*d40, plus a separately retained late50.
    assert_eq!(output.rows()[0].value, if retain_old { 133 } else { 83 });
    let d = request
        .query(query::first_output::Read("first:carriage:d".into()))
        .execute()
        .unwrap();
    assert_eq!(
        d.rows()[0].value,
        40,
        "the newly discovered parent must drain"
    );
    let old = request
        .query(query::first_output::Read("first:carriage:late".into()))
        .execute();
    if retain_old {
        assert_eq!(
            old.unwrap().rows()[0].value,
            50,
            "another real input still needs late"
        );
    } else {
        assert!(
            old.is_err(),
            "an unconsumed superseded parent is not a delivery root"
        );
    }
}

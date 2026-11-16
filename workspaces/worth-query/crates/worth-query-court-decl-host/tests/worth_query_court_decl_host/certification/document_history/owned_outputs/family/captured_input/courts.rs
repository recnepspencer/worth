//! Literal output checks distinguish full captured data from source-only reuse.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};

#[test]
fn a_born_output_consumes_every_seed_and_anchor_before_publication() {
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, captured_inputs: true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert!(request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .is_err());
    let mut edit = request
        .edit_envelope()
        .idempotency("captured-born".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    let token = edit
        .push_stage::<program::Tree, _>(fixture::model::Intent {
            key: "capture:child".into(),
            value: 3,
            key_derived_target: false,
        })
        .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Committed(done) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(done.result(&token).unwrap().value, 3);
    let rows = request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .unwrap();
    // Source 3 plus two ordinary seeds (10, 50), and anchor 40 with weight 2.
    assert_eq!(rows.rows()[0].value, 143);
    for (key, value) in [("a", 10), ("carriage:late", 50), ("carriage:d", 40)] {
        assert_eq!(
            request
                .query(query::first_output::Read(format!("first:{key}")))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            value
        );
    }
}

#[test]
fn capture_only_data_changes_survive_ordinary_input_cutoff_and_key_replay() {
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, captured_inputs: true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("capture-cutoff-original".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::model::Intent {
        key: "capture:child".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let initial = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(initial, Outcome::Committed(_)), "{initial:?}");
    let old = request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(old.value, 143);
    let mut edit = request
        .edit_envelope()
        .idempotency("capture-cutoff-change".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::member::Intent {
        key: "capture:anchor-d".into(),
        value: 4,
        key_derived_target: false,
    })
    .unwrap();
    let changed = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(changed, Outcome::Committed(_)), "{changed:?}");
    assert_eq!(
        request
            .query(query::model::Read("capture:child".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3,
        "ordinary producer input is unchanged"
    );
    let row = request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(
        row.native_id, old.native_id,
        "Preserve retains the genuine output entity"
    );
    assert_eq!(
        row.value, 223,
        "the captured anchor weight changed from 2 to 4"
    );
    let mut edit = request
        .edit_envelope()
        .idempotency("capture-cutoff-original".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(fixture::model::Intent {
        key: "capture:child".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let replay = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(replay, Outcome::RecoveredCommitted(_)),
        "the original full key must return its retained receipt"
    );
    let after = request
        .query(query::first_output::Read("captured:capture:child".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(
        after, row,
        "replay must not reconstruct a stale captured payload"
    );
}

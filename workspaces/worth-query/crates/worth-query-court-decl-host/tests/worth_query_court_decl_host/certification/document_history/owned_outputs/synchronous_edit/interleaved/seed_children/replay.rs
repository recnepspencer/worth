//! Replays the CAD seed edit traced on ef4a3304fb: a source stage writes the
//! seed of a parent whose child is reached only through an occurrence arc,
//! then a document stage that declares a read of the child's family. The
//! traced barrier at that stage held only the seed's own family; the rule
//! of 63eafee holds every family the source stage made available. The edit
//! must commit what two separate commits commit, or refuse atomically.
use super::*;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

type Host = WorthQueryProgramApplicationRuntime<Schema, OccurrenceProgram>;

fn world() -> Host {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() });
    assert!(execute(&host, "seed-world", Some(2), None).is_ok(), "the world commits");
    assert!(observe(&host).1.is_some(), "the occurrence arc places the child");
    host
}

/// `Ok(())` committed; `Err` refused atomically.
fn execute(host: &Host, key: &str, seed: Option<u64>, frontier: Option<u64>) -> Result<(), String> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope().idempotency(key.to_owned()))
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    if let Some(value) = seed {
        edit.push_stage::<OccurrenceWriteTree, _>(fixture::model::Intent {
            key: "a".into(),
            value,
            key_derived_target: false,
        })
        .unwrap();
    }
    if let Some(frontier) = frontier {
        edit.push_stage::<ApplicationNoOutputGraph, _>(frontier::Intent { frontier }).unwrap();
    }
    let before = native_head(host);
    let (outcome, writes) = observe_maintenance_work(32, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
    });
    if let Ok(Outcome::Committed(_)) = &outcome {
        return Ok(());
    }
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0, "a refusal published");
    assert_eq!(native_head(host), before, "a refusal moved the head");
    Err(format!("{outcome:?}").chars().take(1200).collect())
}

/// The seed, the occurrence-discovered child, and the frontier.
fn observe(host: &Host) -> (u64, Option<u64>, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    (
        request.query(query::model::Read("a".into())).execute().unwrap().rows()[0].value,
        (request.query(query::second_output::Read("second:a".into())).execute())
            .ok()
            .and_then(|rows| rows.rows().first().map(|row| row.value)),
        request.query(super::super::reading::Read).execute().unwrap().rows()[0].frontier,
    )
}

#[test]
fn a_seed_write_before_a_document_stage_never_commits_a_stale_occurrence_child() {
    let positioned = world();
    let outcome = execute(&positioned, "positioned", Some(4), Some(5));
    eprintln!("positioned: {outcome:?} -> {:?}", observe(&positioned));
    let sequential = world();
    execute(&sequential, "sequential:0", Some(4), None).expect("the seed write commits alone");
    execute(&sequential, "sequential:1", None, Some(5)).expect("the document stage commits alone");
    eprintln!("sequential: {:?}", observe(&sequential));
    if outcome.is_ok() {
        assert_eq!(observe(&positioned), observe(&sequential), "a positioned commit kept a stale child");
    }
}

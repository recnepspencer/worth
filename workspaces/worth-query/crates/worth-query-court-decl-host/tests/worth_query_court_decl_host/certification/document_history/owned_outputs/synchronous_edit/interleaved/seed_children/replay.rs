//! Replays the CAD seed edit traced on ef4a3304fb: a source stage writes the
//! seed of a parent whose child, and the child's join, are reached only
//! through occurrence arcs two hops from the seed; then a document stage reads
//! the join and records it. The traced barrier at that stage held only the
//! seed's own family; the rule of 63eafee holds every family the source stage
//! made available. The edit commits what two separate commits commit, or
//! refuses atomically with a typed cause.
use super::*;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryInvariantExecutionDenialKind,
};
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

type Host = WorthQueryProgramApplicationRuntime<Schema, OccurrenceProgram>;

fn world() -> Host {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() });
    assert!(
        execute(&host, "seed-world", Some(2), None).is_ok(),
        "the world commits"
    );
    let (_, child, join, _) = observe(&host);
    assert!(
        child.is_some() && join.is_some(),
        "the occurrence arcs place the child and its join: {child:?} {join:?}"
    );
    host
}

/// `Ok(())` committed; `Err` refused atomically, with its typed cause when it has one.
fn execute(
    host: &Host,
    key: &str,
    seed: Option<u64>,
    frontier: Option<u64>,
) -> Result<(), (Option<WorthQueryInvariantExecutionDenialKind>, String)> {
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
        edit.push_stage::<ApplicationNoOutputGraph, _>(frontier::Intent {
            frontier,
            grandchild: true,
        })
        .unwrap();
    }
    let before = native_head(host);
    let (outcome, writes) = observe_maintenance_work(32, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
    });
    if let Ok(Outcome::Committed(_)) = &outcome {
        return Ok(());
    }
    assert_eq!(
        writes.published_rows.len() + writes.omitted_publications,
        0,
        "a refusal published"
    );
    assert_eq!(native_head(host), before, "a refusal moved the head");
    let kind = match &outcome {
        Ok(Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause))) => cause
            .invariant_execution_failure()
            .map(|failure| failure.kind()),
        _ => None,
    };
    Err((kind, format!("{outcome:?}").chars().take(1200).collect()))
}

/// The seed, the occurrence-discovered child and its join, and the frontier.
fn observe(host: &Host) -> (u64, Option<u64>, Option<u64>, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    (
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        (request
            .query(query::second_output::Read("second:a".into()))
            .execute())
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value)),
        (request
            .query(query::member_public::Read("joined:first:a".into()))
            .execute())
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value)),
        request
            .query(super::super::reading::Read)
            .execute()
            .unwrap()
            .rows()[0]
            .frontier,
    )
}

#[test]
fn a_document_stage_reads_the_join_a_seed_discovers_two_occurrence_hops_away() {
    let sequential = world();
    execute(&sequential, "sequential:0", Some(4), None).expect("the seed write commits alone");
    execute(&sequential, "sequential:1", None, Some(5)).expect("the document stage commits alone");
    let expected = observe(&sequential);
    // The document stage recorded the join derived from the final seed.
    assert_eq!(Some(expected.3 - 5 * 10_000), expected.2, "{expected:?}");
    let positioned = world();
    match execute(&positioned, "positioned", Some(4), Some(5)) {
        // The join the seed's arcs discover must settle before the document
        // stage reads it: the edit commits what two commits commit.
        Ok(()) => assert_eq!(observe(&positioned), expected),
        // Otherwise it refuses atomically (asserted by `execute`) with a typed
        // cause, never an untyped failure.
        Err((kind, refusal)) => assert!(kind.is_some(), "an untyped refusal: {refusal}"),
    }
}

//! Journey 6: a filter-only field is part of the read's exact footprint.
use super::edit::{EditNotes, Edited};
use super::observe::Observe;
use super::queries::SearchQuery;
use super::world::Host;
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_host::facade::{application_entry::*, primary_graph::*};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn edit_notes(host: &Host, project: u64, task: u64, notes: &str, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(EditNotes {
            project,
            task,
            notes: notes.to_owned(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the notes edit reaches the installed program");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::<_, Edited>::Committed { .. }
    ));
}

/// Commits a no-op bound to `source`; refuses once anything it read changed.
fn observe(
    host: &Host,
    source: &WorthQueryObservedSource<SearchQuery>,
    key: &str,
) -> Result<(), WorthQueryApplicationRequestMutationDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Observe {
            project: 1,
            q: "bracket".into(),
        })
        .expect_source(source.clone())
        .idempotency(&key.to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .map(|_| ())
}

fn bracket_search(host: &Host) -> (Vec<u64>, WorthQueryObservedSource<SearchQuery>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .query(search_text(1, "bracket"))
        .execute()
        .expect("the bracket search reads");
    let keys = result.rows().iter().map(|row| row.key).collect();
    (keys, result.observed_sources()[0].clone())
}

#[test]
fn editing_an_excluded_rows_filter_only_field_updates_results_and_stales_the_source() {
    let host = install(&board());
    // Only task 5's title says "bracket"; task 4's notes are never projected.
    let (rows, source) = bracket_search(&host);
    assert_eq!(rows, [5]);
    observe(&host, &source, "current").expect("an unchanged source admits its observer");
    // A field outside the scope project is not part of this read.
    edit_notes(&host, 2, 6, "bracket spare", 1);
    observe(&host, &source, "unrelated").expect("another project's edit leaves it current");
    edit_notes(&host, 1, 4, "awaiting spare bracket", 2);
    assert_eq!(bracket_search(&host).0, [4, 5]);
    assert_stale(observe(&host, &source, "stale"));
}

#[test]
fn an_absent_filter_only_field_becoming_present_stales_the_source() {
    let host = install(&board());
    // Task 2 has no notes at all; its absence is part of the read.
    let (rows, source) = bracket_search(&host);
    assert_eq!(rows, [5]);
    edit_notes(&host, 1, 2, "bracket plate", 1);
    assert_eq!(bracket_search(&host).0, [2, 5]);
    assert_stale(observe(&host, &source, "absent-stale"));
}

#[track_caller]
fn assert_stale(result: Result<(), WorthQueryApplicationRequestMutationDenial>) {
    let Err(WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    )) = result
    else {
        panic!("an excluded row filter-only edit must stale the observed source: {result:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
}

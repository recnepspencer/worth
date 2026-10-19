//! Atomic topology through installed native actions, never fixture effect assembly.
use super::{
    child_actions::CreateTree,
    child_query,
    operator_identity::{authenticate_operator, request_scope},
    revision_query, row_query,
    world::{install, App},
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial as QueryDenial,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted, WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
fn input(row: &str) -> CreateTree {
    CreateTree {
        parent: "parent-a".into(),
        row: row.into(),
        physical: 101,
        electronic: 102,
        physical_note: Some("held".into()),
        electronic_note: None,
        detail: "physical detail".into(),
    }
}
fn execute(app: &App, input: CreateTree, key: u64) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, Allocation::SystemAllocation)
        .unwrap()
}
fn cells(app: &App, row: &str) -> Vec<child_query::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(child_query::Read(row.into()))
        .execute()
        .unwrap();
    let mut cells = result.rows().to_vec();
    cells.sort_by_key(|cell| cell.kind);
    cells
}
fn revision(app: &App) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
fn missing_row(app: &App, row: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(row_query::Read(row.into()))
        .execute();
    assert!(
        matches!(result,Err(QueryDenial::ScopeResolution(cause)) if cause.kind()==Resolution::Refused(Refusal::UnknownEntity))
    );
}
#[test]
fn one_candidate_creates_distinct_cells_nested_detail_and_computed_root_and_replays_once() {
    let app = install();
    let original = input("new-row");
    let receipt = match execute(&app, original.clone(), 500) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("{other:?}"),
    };
    let observed = cells(&app, "new-row");
    assert_eq!(observed.len(), 2);
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let mut archive = app
        .request(&principal, &scope)
        .query(child_query::ReadArchive("new-row".into()))
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    archive.sort_by_key(|cell| cell.kind);
    assert_eq!(
        archive, observed,
        "both relations must reach the same actual child identities, not copies"
    );
    assert_ne!(observed[0].entity, observed[1].entity);
    assert_eq!(
        (
            observed[0].id,
            observed[0].kind,
            observed[0].note.as_deref(),
            observed[0].detail.as_deref()
        ),
        (101, 1, Some("held"), Some("physical detail"))
    );
    assert_eq!(
        (
            observed[1].id,
            observed[1].kind,
            observed[1].note.as_deref(),
            observed[1].detail.as_deref()
        ),
        (102, 2, None, None)
    );
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("new-row".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        2
    );
    assert_eq!(revision(&app), 2);
    match execute(&app, original, 500) {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), receipt.outcome_identity())
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(revision(&app), 2);
    assert_eq!(
        cells(&app, "new-row")
            .iter()
            .map(|cell| cell.entity)
            .collect::<Vec<_>>(),
        observed.iter().map(|cell| cell.entity).collect::<Vec<_>>()
    );
}
#[test]
fn invalid_late_child_publishes_neither_parent_revision_nor_any_created_topology() {
    let app = install();
    let mut invalid = input("refused-row");
    invalid.electronic_note = Some("malformed".into());
    let result = execute(&app, invalid, 510);
    assert!(
        matches!(result,Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch{ref member}) if member=="electronic_note"),
        "{result:?}"
    );
    missing_row(&app, "refused-row");
    assert_eq!(revision(&app), 1);
}
#[test]
fn concurrent_parent_edit_stales_the_entire_prepared_creation_graph() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input("loser-row"))
        .without_source()
        .idempotency(&520);
    let candidate = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh operation"),
    };
    assert!(matches!(
        execute(&app, input("winner-row"), 521),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        candidate.commit(Allocation::SystemAllocation),
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    ));
    missing_row(&app, "loser-row");
    assert_eq!(cells(&app, "winner-row").len(), 2);
    assert_eq!(revision(&app), 2);
}

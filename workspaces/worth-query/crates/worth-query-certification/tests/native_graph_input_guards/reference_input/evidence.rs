//! No-source binding isolates native reference CAS from query-source freshness checks.
use super::super::{
    operator_identity::{authenticate_operator, request_scope},
    row_query,
    world::{install, App},
};
use super::{query, Assign};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
fn input(row: &str, selected: Option<&str>) -> Assign {
    Assign {
        row: row.into(),
        selected: selected.map(str::to_owned),
        encoding: super::declaration::Encoding::Normal,
    }
}
fn assign(
    app: &App,
    input: Assign,
    key: u64,
) -> Result<Outcome<WorthQueryOperationBodyDenial, ()>, RequestDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, Allocation::SystemAllocation)
}
fn read(app: &App, row: &str) -> Vec<query::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
#[test]
fn existing_reference_assign_replace_clear_and_exact_replay_preserve_target_identity() {
    let app = install();
    assert!(read(&app, "row-a").is_empty());
    let first = input("row-a", Some("parent-a"));
    let receipt = match assign(&app, first.clone(), 800).unwrap() {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("assign:{other:?}"),
    };
    let initial = read(&app, "row-a");
    assert_eq!(initial[0].0, "parent-a");
    for (id, selected) in [
        (801, Some("parent-a")),
        (802, Some("parent-b")),
        (803, None),
    ] {
        let result = assign(&app, input("row-a", selected), id).unwrap();
        assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
        let current = read(&app, "row-a");
        match selected {
            Some("parent-a") => assert_eq!(current, initial),
            Some(expected) => {
                assert_eq!(current[0].0, expected);
                assert_ne!(current[0].1, initial[0].1);
            }
            None => assert!(current.is_empty()),
        }
    }
    match assign(&app, first, 800).unwrap() {
        Outcome::AlreadyCommitted(recovered) => {
            assert_eq!(recovered.outcome_identity(), receipt.outcome_identity())
        }
        other => panic!("replay:{other:?}"),
    }
    assert!(
        read(&app, "row-a").is_empty(),
        "replay may not resurrect the old reference"
    );
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        7
    );
    // Sharing a selected target is allowed; incoming exclusivity is not invented.
    for (id, row) in [(804, "row-a"), (805, "row-sibling")] {
        assert!(matches!(
            assign(&app, input(row, Some("parent-a")), id).unwrap(),
            Outcome::Committed { .. }
        ));
    }
    assert_eq!(read(&app, "row-a"), read(&app, "row-sibling"));
}
#[test]
fn competing_reference_assignment_refuses_stale_selection_without_duplicate_edges() {
    for existing in [false, true] {
        let app = install();
        if existing {
            assert!(matches!(
                assign(&app, input("row-a", Some("parent-c")), 809).unwrap(),
                Outcome::Committed { .. }
            ));
        }
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let mut request = app
            .request(&principal, &scope)
            .advanced()
            .mutate(input("row-a", Some("parent-a")))
            .without_source()
            .idempotency(&810_u64);
        let prepared = match request
            .prepare_in_program(&app, Allocation::SystemAllocation)
            .unwrap()
        {
            Preparation::Prepared(prepared) => prepared,
            _ => panic!("fresh intent"),
        };
        let result = assign(&app, input("row-a", Some("parent-b")), 811).unwrap();
        assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
        assert!(matches!(
            prepared.commit(Allocation::SystemAllocation),
            Outcome::Commit(Uncommitted::Stale(_))
        ));
        let rows = read(&app, "row-a");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "parent-b");
    }
}
#[test]
fn unknown_reference_target_denies_without_clearing_existing_selection() {
    let app = install();
    assert!(matches!(
        assign(&app, input("row-a", Some("parent-a")), 820).unwrap(),
        Outcome::Committed { .. }
    ));
    match assign(&app, input("row-a", Some("absent-target")), 821) {
        Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) => assert_eq!(
            cause
                .downcast::<WorthQueryEntityResolutionDenial>()
                .unwrap()
                .kind(),
            Resolution::Refused(Refusal::UnknownEntity)
        ),
        other => panic!("expected native unknown target, got {other:?}"),
    }
    let rows = read(&app, "row-a");
    assert_eq!(rows[0].0, "parent-a");
    for (key, encoding) in [
        (822, super::declaration::Encoding::Missing),
        (823, super::declaration::Encoding::WrongFamily),
    ] {
        let mut bad = input("row-a", None);
        bad.encoding = encoding;
        assert!(
            matches!(assign(&app,bad,key).unwrap(),Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch{member}) if member=="selected")
        );
        assert_eq!(
            read(&app, "row-a"),
            rows,
            "invalid input may not clear the reference"
        );
    }
}

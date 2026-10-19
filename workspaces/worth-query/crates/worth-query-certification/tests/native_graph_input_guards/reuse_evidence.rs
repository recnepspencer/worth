//! Identity/currentness proof, not a second signing fixture or an NCR parity claim.
use super::{
    current_snapshot_query,
    operator_identity::{authenticate_operator, request_scope},
    reuse_actions::*,
    revision_query, row_query,
    world::{install, App},
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial},
    runtime::ExecutionAllocationPolicy as Allocation,
};
fn select(app: &App, input: Select, key: u64) -> Outcome<WorthQueryOperationBodyDenial, ()> {
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
fn identity(app: &App) -> impl Eq + std::fmt::Debug + use<> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(current_snapshot_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .1
}
fn values(app: &App) -> (u64, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let row = app
        .request(&principal, &scope)
        .query(row_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0;
    let revision = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0;
    (row, revision)
}
fn input(note: Option<&str>, row_value: u64) -> Select {
    Select {
        row: "row-a".into(),
        identity: "same-source".into(),
        value: 10,
        note: note.map(str::to_owned),
        label: "display".into(),
        row_value,
    }
}
#[test]
fn complete_content_reuses_actual_identity_but_independent_effects_and_replay_still_work() {
    let app = install();
    assert!(matches!(
        select(&app, input(None, 20), 400),
        Outcome::Committed { .. }
    ));
    let original = identity(&app);
    let request = input(None, 30);
    let receipt = match select(&app, request.clone(), 401) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("{other:?}"),
    };
    assert_eq!(original, identity(&app));
    assert_eq!(values(&app), (30, 21)); // Scope and parent-path writes are not skipped.
    match select(&app, request, 401) {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), receipt.outcome_identity())
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(values(&app), (30, 21));
    assert_eq!(original, identity(&app));
    // Only optional provenance changes; same source ID and selected value are not sufficient.
    assert!(matches!(
        select(&app, input(Some("source-revision-2"), 40), 402),
        Outcome::Committed { .. }
    ));
    let changed = identity(&app);
    assert_ne!(original, changed);
    assert!(matches!(
        select(&app, input(Some("source-revision-2"), 50), 403),
        Outcome::Committed { .. }
    ));
    assert_eq!(changed, identity(&app));
    assert!(matches!(
        select(&app, input(None, 60), 404),
        Outcome::Committed { .. }
    ));
    assert_ne!(changed, identity(&app));
    assert_eq!(values(&app), (60, 51));
}
#[test]
fn comparison_observes_absence_and_all_fields_even_after_an_earlier_mismatch() {
    for mismatch in [false, true] {
        let app = install();
        assert!(matches!(
            select(&app, input(None, 20), 410),
            Outcome::Committed { .. }
        ));
        let original = identity(&app);
        let scope = request_scope();
        let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
        let mut selected = input(None, 30);
        if mismatch {
            selected.identity = "new-source".into();
        }
        let mut request = app
            .request(&principal, &scope)
            .advanced()
            .mutate(selected)
            .without_source()
            .idempotency(&411);
        let prepared = match request
            .prepare_in_program(&app, Allocation::SystemAllocation)
            .unwrap()
        {
            Preparation::Prepared(candidate) => candidate,
            Preparation::Settled(_) => panic!("fresh request"),
        };
        let edit = app
            .request(&principal, &scope)
            .advanced()
            .mutate(EditNote {
                identity: "same-source".into(),
                note: Some("new-provenance".into()),
            })
            .without_source()
            .idempotency(&412)
            .execute_in_program(&app, Allocation::SystemAllocation)
            .unwrap();
        assert!(matches!(edit, Outcome::Committed { .. }), "{edit:?}");
        let result = prepared.commit(Allocation::SystemAllocation);
        assert!(
            matches!(
                result,
                Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
            ),
            "{result:?}"
        );
        assert_eq!(identity(&app), original);
        assert_eq!(values(&app), (20, 8));
    }
}
#[test]
fn selection_retargeting_stales_reuse_without_publishing_other_effects() {
    let app = install();
    assert!(matches!(
        select(&app, input(None, 20), 420),
        Outcome::Committed { .. }
    ));
    let original = identity(&app);
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input(None, 30))
        .without_source()
        .idempotency(&421);
    let prepared = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh request"),
    };
    assert!(matches!(
        select(&app, input(Some("new"), 40), 422),
        Outcome::Committed { .. }
    ));
    let changed = identity(&app);
    assert_ne!(original, changed);
    assert!(matches!(
        prepared.commit(Allocation::SystemAllocation),
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    ));
    assert_eq!(identity(&app), changed);
    assert_eq!(values(&app), (40, 21));
}
#[test]
fn installed_codec_rejection_cannot_reuse_or_replace_an_existing_selection() {
    let app = install();
    assert!(matches!(
        select(&app, input(None, 20), 430),
        Outcome::Committed { .. }
    ));
    let original = identity(&app);
    let refused = select(&app, input(Some("malformed"), 30), 431);
    assert!(
        matches!(
            refused,
            Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch { .. })
        ),
        "{refused:?}"
    );
    assert_eq!(identity(&app), original);
    assert_eq!(values(&app), (20, 8));
}

#[test]
fn absent_optional_fields_can_be_filled_by_literal_and_required_input_initializers() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let initial = app
        .request(&principal, &scope)
        .advanced()
        .mutate(super::retiring_actions::KeepSnapshot {
            row: "row-a".into(),
            identity: "same-source".into(),
            value: 10,
        })
        .without_source()
        .idempotency(&440)
        .execute_in_program(&app, Allocation::SystemAllocation)
        .unwrap();
    assert!(matches!(initial, Outcome::Committed { .. }), "{initial:?}");
    let before = identity(&app);
    // Keep omits optional Label/Kind. Reuse initializes Label from required input
    // and Kind from a literal; installed field presence, not initializer kind, wins.
    let replacement = select(&app, input(None, 20), 441);
    assert!(
        matches!(replacement, Outcome::Committed { .. }),
        "{replacement:?}"
    );
    assert_ne!(identity(&app), before);
    assert_eq!(values(&app), (20, 11));
    let after = identity(&app);
    assert!(matches!(
        select(&app, input(None, 30), 442),
        Outcome::Committed { .. }
    ));
    assert_eq!(identity(&app), after);
}

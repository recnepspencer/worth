//! A fresh native event publishes bookkeeping and evidence together; alias/stale attempts publish neither.
use super::super::{
    operator_identity::{authenticate_operator, block_on, request_scope},
    path_value::BumpRow,
    revision_query, row_query,
    world::install,
};
use super::{authentication, query, SignGraph, SignGraphAlias, SignScopeAlias};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted,
        WorthQuerySignedCreationPreparationDenial as SigningDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
#[test]
fn signed_parent_bookkeeping_is_atomic_and_original_replay_needs_no_new_event() {
    let mut app = install();
    let owner = authentication::install(&mut app);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let source = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SignGraph("row-a".into()))
        .expect_source(source.clone())
        .idempotency(&700_u64);
    let mut prepared = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh signing intent"),
    };
    assert_eq!(
        app.request(&principal, &scope)
            .query(revision_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        1
    );
    assert!(app
        .request(&principal, &scope)
        .query(query::ReadGraph("row-a".into()))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    let event = block_on(owner.authenticate(
        "verify",
        &principal,
        prepared.authentication_intent().unwrap().clone(),
        &scope,
    ))
    .unwrap();
    prepared.sign(&event).unwrap();
    let receipt = match prepared.commit(Allocation::SystemAllocation) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("signed commit:{other:?}"),
    };
    let history = app
        .request(&principal, &scope)
        .query(query::ReadGraph("row-a".into()))
        .execute()
        .unwrap();
    assert_eq!(history.rows().len(), 1);
    let audits = super::super::peer_evidence::journal(&app, "parent-a");
    assert_eq!(audits.len(), 1);
    assert_eq!(
        (audits[0].name.as_str(), audits[0].revision, audits[0].actor),
        ("signed", 2, 1)
    );
    let signed = &history.rows()[0];
    assert_eq!((signed.value, signed.revision, signed.actor), (7, 1, 1));
    assert_eq!(signed.company.as_deref(), Some("company-a"));
    assert_eq!(
        signed
            .provenance
            .dependencies()
            .iter()
            .map(|dependency| dependency.source_field.field())
            .collect::<Vec<_>>(),
        vec!["RowValue", "Company"]
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(revision_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        2
    );
    let later = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute_with_expression_context()
        .unwrap();
    assert!(matches!(
        app.request(&principal, &scope)
            .advanced()
            .mutate(BumpRow {
                row: "row-a".into(),
                value: 99
            })
            .expect_source(later.observed_sources()[0].clone())
            .idempotency(&701)
            .execute_in_program(&app, Allocation::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    let mut replay = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SignGraph("row-a".into()))
        .expect_source(source)
        .idempotency(&700_u64);
    match replay
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Settled(Outcome::AlreadyCommitted(recovered)) => {
            assert_eq!(recovered.outcome_identity(), receipt.outcome_identity())
        }
        _ => panic!("exact replay must settle before asking for another authentication event"),
    }
    assert_eq!(
        app.request(&principal, &scope)
            .query(query::ReadGraph("row-a".into()))
            .execute()
            .unwrap()
            .rows(),
        history.rows()
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(revision_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        3
    );
    assert_eq!(
        super::super::peer_evidence::journal(&app, "parent-a"),
        audits
    );
}
#[test]
fn signed_parent_race_preserves_only_the_winner_without_losing_evidence_or_revision() {
    let mut app = install();
    let owner = authentication::install(&mut app);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let source = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute_with_expression_context()
        .unwrap();
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SignGraph("row-a".into()))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&710_u64);
    let mut prepared = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh signing candidate"),
    };
    let event = block_on(owner.authenticate(
        "verify",
        &principal,
        prepared.authentication_intent().unwrap().clone(),
        &scope,
    ))
    .unwrap();
    prepared.sign(&event).unwrap();
    let sibling = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-sibling".into()))
        .execute_with_expression_context()
        .unwrap();
    assert!(matches!(
        app.request(&principal, &scope)
            .advanced()
            .mutate(BumpRow {
                row: "row-sibling".into(),
                value: 42
            })
            .expect_source(sibling.observed_sources()[0].clone())
            .idempotency(&711)
            .execute_in_program(&app, Allocation::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        prepared.commit(Allocation::SystemAllocation),
        Outcome::Commit(Uncommitted::Stale(_))
    ));
    assert!(super::super::peer_evidence::journal(&app, "parent-a").is_empty());
    assert_eq!(
        app.request(&principal, &scope)
            .query(revision_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        2
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        7
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-sibling".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        42
    );
    assert!(app
        .request(&principal, &scope)
        .query(query::ReadGraph("row-a".into()))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
}
#[test]
fn graph_and_scope_subjects_refuse_actual_alias_writes_before_an_event_is_requested() {
    let mut app = install();
    let _owner = authentication::install(&mut app);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    // This parent's unique row makes both cyclic paths select one actual endpoint,
    // so ambiguity cannot counterfeit the intended Subject-write refusal.
    let source = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-mismatch".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let mut graph = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SignGraphAlias("row-mismatch".into()))
        .expect_source(source.clone())
        .idempotency(&720_u64);
    assert!(matches!(
        graph.prepare_in_program(&app, Allocation::SystemAllocation),
        Err(RequestDenial::SignedCreation(
            SigningDenial::CandidateMismatch
        ))
    ));
    let mut scope_request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(SignScopeAlias("row-mismatch".into()))
        .expect_source(source)
        .idempotency(&721_u64);
    assert!(matches!(
        scope_request.prepare_in_program(&app, Allocation::SystemAllocation),
        Err(RequestDenial::SignedCreation(
            SigningDenial::CandidateMismatch
        ))
    ));
    assert_eq!(
        app.request(&principal, &scope)
            .query(revision_query::Read("row-mismatch".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        1
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-mismatch".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        7
    );
    assert!(app
        .request(&principal, &scope)
        .query(query::ReadGraph("row-mismatch".into()))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        app.request(&principal, &scope)
            .query(query::ReadScope("row-mismatch".into()))
            .execute()
            .unwrap()
            .rows()
            .iter()
            .map(|row| row.0)
            .collect::<Vec<_>>(),
        Vec::<u64>::new()
    );
}

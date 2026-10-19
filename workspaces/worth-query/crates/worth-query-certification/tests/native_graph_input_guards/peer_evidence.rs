//! Independent audit effects remain atomic and are not swallowed by snapshot reuse.
use super::{
    operator_identity::{authenticate_operator, request_scope},
    peer_audit,
    reuse_evidence::{identity, input, select, values},
    world::{install, App},
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub(super) fn journal(app: &App, parent: &str) -> Vec<peer_audit::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(peer_audit::Read(parent.into()))
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
#[test]
fn snapshot_reuse_does_not_suppress_parent_audit_and_replay_does_not_duplicate_it() {
    let app = install();
    assert!(matches!(
        select(&app, input(None, 20), 1000),
        Outcome::Committed { .. }
    ));
    let snapshot = identity(&app);
    let request = input(None, 30);
    let receipt = match select(&app, request.clone(), 1001) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("{other:?}"),
    };
    assert_eq!(identity(&app), snapshot);
    let audits = journal(&app, "parent-a");
    assert_eq!(audits.len(), 2);
    assert_ne!(audits[0].entity, audits[1].entity);
    assert!(audits
        .iter()
        .all(|a| a.detail.as_deref() == Some("independent")));
    let mut revisions = audits
        .iter()
        .map(|r| (r.name.as_str(), r.revision, r.actor))
        .collect::<Vec<_>>();
    revisions.sort();
    assert_eq!(
        revisions,
        vec![("same-source", 8, 1), ("same-source", 21, 1)]
    );
    assert!(journal(&app, "parent-b").is_empty());
    assert!(
        matches!(select(&app,request,1001),Outcome::AlreadyCommitted(replay) if replay.outcome_identity()==receipt.outcome_identity())
    );
    assert_eq!(journal(&app, "parent-a"), audits);
    assert!(matches!(
        select(&app, input(Some("changed-provenance"), 40), 1002),
        Outcome::Committed { .. }
    ));
    assert_ne!(identity(&app), snapshot);
    let retained = journal(&app, "parent-a");
    assert_eq!(retained.len(), 3);
    for audit in audits {
        assert!(retained.contains(&audit));
    }
    assert_eq!(values(&app), (40, 31));
}
#[test]
fn stale_or_invalid_input_publishes_neither_snapshot_nor_independent_audit() {
    let app = install();
    assert!(matches!(
        select(&app, input(None, 20), 1010),
        Outcome::Committed { .. }
    ));
    let snapshot = identity(&app);
    let before = journal(&app, "parent-a");
    assert!(matches!(
        select(&app, input(Some("malformed"), 30), 1011),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch { .. })
    ));
    assert_eq!(journal(&app, "parent-a"), before);
    assert_eq!(identity(&app), snapshot);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input(None, 30))
        .without_source()
        .idempotency(&1012);
    let prepared = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        _ => panic!("fresh candidate"),
    };
    assert!(matches!(
        select(&app, input(Some("winner"), 40), 1013),
        Outcome::Committed { .. }
    ));
    let authoritative = journal(&app, "parent-a");
    let winner = identity(&app);
    assert!(matches!(
        prepared.commit(Allocation::SystemAllocation),
        Outcome::Commit(Uncommitted::Stale(_))
    ));
    assert_eq!(journal(&app, "parent-a"), authoritative);
    assert_eq!(authoritative.len(), 2);
    assert_eq!(identity(&app), winner);
    assert_ne!(winner, snapshot);
    assert_eq!(values(&app), (40, 21));
}

//! External writes preserve the canonical producer records.
use super::{settlement::*, write_fence::Destroy, *};
fn refused(action: u8) {
    let app = world::install_two_levels();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    let before = checkpoint::capture(&app);
    let node = request.query(Read("root".into())).execute().unwrap().rows()[0].native_id;
    let member = request
        .query(realization_read::ReadRealization("root".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let history = checkpoint::envelopes(&before);
    let origin = history
        .iter()
        .filter_map(|e| e.get("required_output_origin"))
        .find(|origin| origin["meaning"]["source_scope"] == serde_json::to_value(node).unwrap())
        .unwrap();
    let publication = serde_json::from_value(origin["commit"]["commit_id"].clone()).unwrap();
    let edge = serde_json::from_value(
        origin["relations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| {
                r["target"] == serde_json::to_value(node).unwrap()
                    && r["source"] == serde_json::to_value(member).unwrap()
            })
            .unwrap()["id"]
            .clone(),
    )
    .unwrap();
    let outcome = request
        .advanced()
        .mutate(Destroy {
            key: "root".into(),
            action,
        })
        .idempotency(&44)
        .execute_in_program::<Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("non-custodial destruction committed: {outcome:?}");
    };
    let error = denial
        .authored_retirement_error()
        .expect("original typed Native refusal");
    assert_eq!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner: node }));
    use worth_relational::facade::transactions::RecordRef;
    let subject = match action {
        0 => RecordRef::Relation(edge),
        1 => RecordRef::Entity(member),
        2 => RecordRef::Entity(node),
        _ => unreachable!(),
    };
    let expected = if action == 2 {
        vec![subject]
    } else {
        vec![subject, RecordRef::Entity(node)]
    };
    assert_eq!(error.context.affected_records, expected);
    assert_eq!(
        checkpoint::envelopes(&checkpoint::capture(&app)),
        history,
        "refusal has no canonical effect"
    );
    assert_live(&request, &["root", "root.0", "root.1"]);
    // A later real Preserve still reads its intact predecessor, including the edge.
    resize(&app, &request, 2, 45);
    close(settle(&app, &request), &request);
}
#[test]
fn external_unlink_refuses_before_effect() {
    refused(0);
}
#[test]
fn owned_member_delete_without_output_roles_refuses_before_effect() {
    refused(1);
}
#[test]
fn source_delete_with_live_owned_edges_refuses_before_effect() {
    refused(2);
}

#[test]
fn independent_publication_keeps_ordinary_unlink_behavior() {
    let app = world::install(2, Arc::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    independent_generator_courts::settle_independent(&app, &request);
    let outcome = request
        .advanced()
        .mutate(Destroy {
            key: "root".into(),
            action: 0,
        })
        .idempotency(&46)
        .execute_in_program::<Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_live(&request, &["root", "root.0", "root.1"]);
}

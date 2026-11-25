//! External writes preserve the canonical producer records.
use super::{settlement::*, write_fence::ExternalWrite, *};
fn refused(action: u8) {
    let app = world::install_two_levels();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let settled = settle(&app, &request);
    let receipt = settled
        .settlement()
        .root_settlement()
        .application_commit_receipt()
        .expect("settled root producer commit receipt")
        .clone();
    close(settled, &request);
    let head_before = world::native_head(&app);
    let node = request.query(Read("root".into())).execute().unwrap().rows()[0].native_id;
    let member = request
        .query(realization_read::ReadRealization("root".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let publication = receipt.commit_reference().commit_id;
    let _ = write_fence::take_last_realizes_edge();
    let outcome = request
        .advanced()
        .mutate(ExternalWrite {
            key: "root".into(),
            action,
        })
        .idempotency(&44)
        .execute_in_program::<Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let edge = write_fence::take_last_realizes_edge()
        .expect("realizes relation read by write_fence handler");
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
        world::native_head(&app),
        head_before,
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
        .mutate(ExternalWrite {
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

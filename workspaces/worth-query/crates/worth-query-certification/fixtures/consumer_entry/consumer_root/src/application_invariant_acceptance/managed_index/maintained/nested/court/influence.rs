//! Whole projected influence includes descendants even when a candidate is rejected.
use super::*;
#[test]
fn descendant_changes_refresh_watch_and_refuse_populated_and_negative_expectations() {
    let (world, _, _, mut model, mut key) = setup();
    for (from, to) in [
        ("x", "a"),
        ("x", "b"),
        ("a", "z"),
        ("b", "z"),
        ("z", "c"),
        ("q", "x"),
        ("q", "p"),
    ] {
        link(&world, &mut model, &mut key, from, to, 1);
    }
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let pinned = request.retain_current_read().unwrap();
    let old_source = request
        .at(&pinned)
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let (initial, mut watch) = request
        .query(find("A", "match"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let original = initial.result_set_observation().unwrap().clone();
    assert!(original.has_verified_realized_footprint());
    let root = (initial.rows()[0].coordinate, initial.rows()[0].name.clone());
    let before = model.clone();
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "c".into(),
            value: "changed-tail".into(),
        },
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = request
        .query(find("A", "match"))
        .next_watch(&mut watch)
        .unwrap()
    else {
        panic!("the unchanged root prefix must still deliver its changed descendant");
    };
    assert_eq!(
        values(update.read().rows()),
        model.expected("A", "match", true)
    );
    assert_eq!(
        (
            update.read().rows()[0].coordinate,
            update.read().rows()[0].name.clone()
        ),
        root
    );
    assert_eq!(values(initial.rows()), before.expected("A", "match", true));
    assert!(matches!(
        request
            .query(find("A", "match"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    let selected = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    key += 1;
    let stale = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "A".into(),
            prefix: "match".into(),
            item: "x".into(),
            replacement: "incorrect".into(),
        })
        .expect_result_set(original)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = stale.expect_err("descendant content must invalidate the genuine retained Source")
    else {
        panic!("expected actual Source currentness denial");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &selected
    );
    assert_eq!(
        exact(
            request
                .at(&pinned)
                .query(query::ReadSource {
                    directory: "A".into()
                })
                .execute()
                .unwrap()
                .rows()
        ),
        exact(old_source.rows())
    );
    let index = request
        .query(find("A", "match"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(find("A", "match"))
        .rebuild_index(&index)
        .unwrap();
    assert_eq!(
        values(request.query(find("A", "match")).execute().unwrap().rows()),
        model.expected("A", "match", true)
    );

    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Enabled {
            item: "x".into(),
            value: false,
        },
    );
    let negative = request.query(find("A", "match")).execute().unwrap();
    assert!(negative.rows().is_empty());
    let negative = negative.into_result_set_observation().unwrap();
    assert!(negative.has_verified_realized_footprint());
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "z".into(),
            value: "changed-rejected-descendant".into(),
        },
    );
    let selected = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    key += 1;
    let stale = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "A".into(),
            prefix: "match".into(),
            item: "x".into(),
            replacement: "incorrect-negative".into(),
        })
        .expect_result_set(negative)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = stale.expect_err("a rejected candidate's full declared projection remains influence")
    else {
        panic!("expected actual negative Source currentness denial");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &selected
    );
}

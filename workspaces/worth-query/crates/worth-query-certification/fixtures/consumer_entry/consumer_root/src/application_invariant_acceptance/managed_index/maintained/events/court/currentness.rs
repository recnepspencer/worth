use super::*;
#[test]
fn direct_source_keeps_complete_support_when_history_changes_with_identical_rows() {
    let (world, _, _, mut key) = setup();
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
    // D-local y keeps the rendered roster fixed when its predecessor shortens.
    edit_history(
        &world,
        &mut key,
        history::Edit::Introduction {
            event: "D".into(),
            item: "y".into(),
            introduced: true,
            edition: 2,
        },
    );
    let old = request
        .query(query::ReadSource {
            directory: "D".into(),
        })
        .execute()
        .unwrap();
    let expected = old.result_set_observation().unwrap().clone();
    edit_history(
        &world,
        &mut key,
        history::Edit::Previous {
            event: "D".into(),
            previous: Some("A".into()),
            edition: 3,
        },
    );
    let current = request
        .query(query::ReadSource {
            directory: "D".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        current.rows(),
        old.rows(),
        "the rendered projection is deliberately identical"
    );
    let fresh = current.result_set_observation().unwrap().clone();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    key += 1;
    let input = || plain::Rename {
        directory: "D".into(),
        item: "x".into(),
        replacement: "fresh-name".into(),
    };
    let refused = request
        .advanced()
        .mutate(input())
        .expect_result_set(expected)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(denial),
    ) = refused.expect_err("full physical history remains part of the direct Source")
    else {
        panic!("wrong direct source refusal");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    key += 1;
    let result = request
        .advanced()
        .mutate(input())
        .expect_result_set(fresh)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        result.result().is_some(),
        "the fresh source is the lawful twin: {result:?}"
    );
}

#[test]
fn an_empty_derived_source_retains_its_negative_history_and_accepts_a_fresh_twin() {
    let (world, _, _, mut key) = setup();
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
    let empty = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    assert!(empty.rows().is_empty());
    let expected = empty.result_set_observation().unwrap().clone();
    let (_, mut watch) = request
        .query(find("B", ""))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    edit_history(
        &world,
        &mut key,
        history::Edit::Introduction {
            event: "B".into(),
            item: "z".into(),
            introduced: true,
            edition: 1,
        },
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) =
        request.query(find("B", "")).next_watch(&mut watch).unwrap()
    else {
        panic!("an empty history must retain its negative probes");
    };
    assert_eq!(names(update.read().rows()), set(&["z"]));
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    key += 1;
    let input = || plain::Rename {
        directory: "B".into(),
        item: "z".into(),
        replacement: "new-z".into(),
    };
    let refused = request
        .advanced()
        .mutate(input())
        .expect_result_set(expected)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(denial),
    ) = refused.expect_err("empty Source is a real negative observation")
    else {
        panic!("wrong empty source refusal");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    let fresh = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap()
        .into_result_set_observation()
        .unwrap();
    key += 1;
    let result = request
        .advanced()
        .mutate(input())
        .expect_result_set(fresh)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(result.result().is_some(), "{result:?}");
}

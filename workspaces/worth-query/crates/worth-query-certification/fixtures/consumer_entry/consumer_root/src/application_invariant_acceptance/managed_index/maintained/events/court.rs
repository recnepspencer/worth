//! Real published event membership, ordinary Query and maintained Parent share truth.
use super::super::super::super::authentication;
use super::super::host;
use super::{authoring, history, mutation, paths, plain, prefix, query, rule, schema};
use std::{collections::BTreeSet, sync::Arc};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
mod fixture;
use fixture::*;
#[test]
fn newest_event_reintroduction_and_inverse_queries_use_real_registered_sources() {
    let (world, _, records, mut key) = setup();
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
    for (event, wanted) in [
        ("A", vec!["x"]),
        ("B", vec![]),
        ("C", vec!["x", "y"]),
        ("D", vec!["x", "y"]),
        ("U", vec!["z"]),
    ] {
        let source = request
            .query(query::ReadSource {
                directory: event.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(names(source.rows()), set(&wanted));
        let parent = request.query(find(event, "")).execute().unwrap();
        assert_eq!(names(parent.rows()), set(&wanted));
    }
    let inverse = request
        .query(paths::Inverse { item: "x".into() })
        .execute()
        .unwrap();
    assert_eq!(
        inverse
            .rows()
            .iter()
            .map(|r| r.identity.as_str())
            .collect::<Vec<_>>(),
        vec!["A", "C", "D"]
    );
    let multiple = request
        .query(paths::RoundTrip {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(names(multiple.rows()), set(&["x", "y"]));
    records.records.lock().unwrap().clear();
    edit_history(
        &world,
        &mut key,
        history::Edit::Retirement {
            event: "C".into(),
            item: "x".into(),
            retired: true,
            edition: 1,
        },
    );
    assert_eq!(
        names(
            request
                .query(query::ReadSource {
                    directory: "D".into()
                })
                .execute()
                .unwrap()
                .rows()
        ),
        set(&["y"])
    );
    assert_eq!(
        request
            .query(paths::Inverse { item: "x".into() })
            .execute()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.identity.as_str())
            .collect::<Vec<_>>(),
        vec!["A"]
    );
    let observed = records.records.lock().unwrap();
    assert!(observed.iter().any(|r| r
        .before
        .as_ref()
        .is_some_and(|before| names(before) == set(&["x", "y"]))
        && names(&r.after) == set(&["y"])));
    drop(observed);
    edit_history(
        &world,
        &mut key,
        history::Edit::Retirement {
            event: "C".into(),
            item: "x".into(),
            retired: false,
            edition: 2,
        },
    );
    assert_eq!(
        names(
            request
                .query(query::ReadSource {
                    directory: "D".into()
                })
                .execute()
                .unwrap()
                .rows()
        ),
        set(&["x", "y"])
    );
}
#[test]
fn event_history_rewire_refreshes_watch_and_refuses_the_original_parent_expectation() {
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
    let pinned = request.retain_current_read().unwrap();
    let old = request
        .at(&pinned)
        .query(query::ReadSource {
            directory: "D".into(),
        })
        .execute()
        .unwrap();
    let (initial, mut watch) = request
        .query(find("D", ""))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(names(initial.rows()), set(&["x", "y"]));
    let expected = initial.result_set_observation().unwrap().clone();
    edit_history(
        &world,
        &mut key,
        history::Edit::Previous {
            event: "D".into(),
            previous: Some("A".into()),
            edition: 2,
        },
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) =
        request.query(find("D", "")).next_watch(&mut watch).unwrap()
    else {
        panic!("real event rewire must deliver");
    };
    assert_eq!(names(update.read().rows()), set(&["x"]));
    assert_eq!(names(old.rows()), set(&["x", "y"]));
    assert!(matches!(
        request.query(find("D", "")).next_watch(&mut watch).unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    key += 1;
    let refused = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "D".into(),
            prefix: "".into(),
            item: "x".into(),
            replacement: "must-not-publish".into(),
        })
        .expect_result_set(expected)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(denial),
    ) = refused.expect_err("history movement must stale this real observation")
    else {
        panic!("wrong source refusal");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
}

mod controls;
mod currentness;

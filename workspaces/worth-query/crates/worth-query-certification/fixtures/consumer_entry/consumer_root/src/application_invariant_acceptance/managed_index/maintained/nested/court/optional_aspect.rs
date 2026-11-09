//! Presence-only edits must reach providers, full Source influence and watches.
use super::*;
use std::sync::{atomic::Ordering, Mutex};
struct Update {
    before: Vec<query::ItemRow>,
    after: Vec<query::ItemRow>,
    changed: Vec<(query::ItemRow, query::ItemRow)>,
}

#[test]
fn root_and_child_optional_aspects_maintain_presence_fields_and_complete_influence() {
    let (world, calls, records, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "x", "a", 1);
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
    let (initial, mut watch) = request
        .query(find("A", "match"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(values(initial.rows()), model.expected("A", "match", true));
    assert_eq!(initial.rows()[0].badge, None);
    assert_eq!(initial.rows()[0].children[0].badge, None);
    let builds = calls.builds.load(Ordering::SeqCst);
    // The first present parent has an absent optional child. Removing the parent
    // later leaves every surviving value, coordinate and relation unchanged.
    for item in ["x", "a"] {
        for value in [
            Some((true, None)),
            Some((false, None)),
            Some((false, Some(true))),
            Some((false, None)),
            None,
        ] {
            let previous = model.clone();
            let original = request
                .query(find("A", "match"))
                .execute()
                .unwrap()
                .into_result_set_observation()
                .unwrap();
            let observed_updates = Arc::new(Mutex::new(Vec::new()));
            let observed = Arc::clone(&observed_updates);
            *calls.source_updates.lock().unwrap() =
                Some(Box::new(move |before, after, changed| {
                    observed.lock().unwrap().push(Update {
                        before,
                        after,
                        changed,
                    });
                }));
            records.records.lock().unwrap().clear();
            publish(
                &world,
                &mut model,
                &mut key,
                Edit::Badge {
                    item: item.into(),
                    value,
                },
            );
            calls.source_updates.lock().unwrap().take();
            let updates = std::mem::take(&mut *observed_updates.lock().unwrap());
            assert!(
                !updates.is_empty(),
                "each aspect transition reaches provider update"
            );
            let affected: std::collections::BTreeSet<_> = records
                .records
                .lock()
                .unwrap()
                .iter()
                .map(|record| record.scope)
                .collect();
            assert_eq!(
                affected,
                [model.scope("A"), model.scope("B")].into_iter().collect()
            );
            for update in updates {
                // Both shared scopes contain x, and both must receive its changed tree.
                assert!(
                    !update.changed.is_empty(),
                    "presence-only removal remains a Source change"
                );
                let directory = if update.after.iter().any(|row| row.identity == "y") {
                    "B"
                } else {
                    "A"
                };
                assert_eq!(
                    values(&update.before),
                    previous.expected(directory, "", false)
                );
                assert_eq!(values(&update.after), model.expected(directory, "", false));
                for (before, after) in &update.changed {
                    assert_ne!(
                        before, after,
                        "presence-only removal differs in provider before/after"
                    );
                }
                let ordinary = request
                    .query(query::ReadSource {
                        directory: directory.into(),
                    })
                    .execute()
                    .unwrap();
                assert_eq!(
                    exact(&update.after),
                    exact(ordinary.rows()),
                    "provider receiver equals complete ordinary Source read"
                );
            }
            for record in records.records.lock().unwrap().iter() {
                let directory = model.directory(record.scope);
                assert_eq!(
                    values(record.before.as_ref().unwrap()),
                    previous.expected(directory, "", false)
                );
                assert_eq!(values(&record.after), model.expected(directory, "", false));
                let ordinary = request
                    .query(query::ReadSource {
                        directory: directory.into(),
                    })
                    .execute()
                    .unwrap();
                assert_eq!(exact(&record.after), exact(ordinary.rows()));
            }
            let WorthQueryManagedIndexWatchOutcome::Delivered(update) = request
                .query(find("A", "match"))
                .next_watch(&mut watch)
                .unwrap()
            else {
                panic!("aspect presence is complete watch influence");
            };
            assert_eq!(
                values(update.read().rows()),
                model.expected("A", "match", true)
            );
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
            ) = stale.expect_err("presence-only removal invalidates the retained complete Source")
            else {
                panic!("expected actual Source currentness denial");
            };
            assert_eq!(
                cause.kind(),
                WorthQueryApplicationAttemptDenialKind::SourceChanged,
                "presence-only removal invalidates the retained complete Source"
            );
        }
    }
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "warm presence edits use provider update"
    );
}

#[test]
fn removing_a_child_aspect_invalidates_negative_complete_source_influence() {
    let (world, _, _, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "x", "a", 1);
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Badge {
            item: "a".into(),
            value: Some((true, None)),
        },
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
    let negative = request.query(find("A", "match")).execute().unwrap();
    assert!(negative.rows().is_empty());
    let negative = negative.into_result_set_observation().unwrap();
    assert!(negative.has_verified_realized_footprint());
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Badge {
            item: "a".into(),
            value: None,
        },
    );
    key += 1;
    let stale = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "A".into(),
            prefix: "match".into(),
            item: "x".into(),
            replacement: "incorrect".into(),
        })
        .expect_result_set(negative)
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = stale.expect_err("absent child aspect changes even a rejected candidate's full Source")
    else {
        panic!("expected actual negative Source currentness denial");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
}

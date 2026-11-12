//! A real published successor borrows only its declared parent's immutable root.
use super::*;
use std::sync::atomic::Ordering;

#[test]
fn ordered_birth_cannot_use_a_predecessor_changed_in_the_same_candidate() {
    let (world, calls, _, mut key) = setup();
    let builds = calls.builds.load(Ordering::SeqCst);
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            event: "E".into(),
            previous: "A".into(),
            introduced: None,
            retired: None,
            rename: Some(("x".into(), "new-prefix".into())),
        },
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
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
    let indexed = request.query(find("E", "")).execute().unwrap();
    let cold = request
        .query(query::ReadSource {
            directory: "E".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(indexed.rows(), cold.rows());
    assert_eq!(indexed.rows()[0].name, "new-prefix");
}

#[test]
fn ordered_birth_applies_only_its_step_and_preserves_the_predecessor() {
    let (world, calls, _, mut key) = setup();
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            rename: None,
            event: "E".into(),
            previous: "A".into(),
            introduced: Some(("e".into(), "match-e".into())),
            retired: Some("x".into()),
        },
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "birth must not cold-build either scope"
    );
    assert_eq!(calls.updates.load(Ordering::SeqCst), updates + 1);
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
    for (event, wanted) in [("A", set(&["x"])), ("E", set(&["e"]))] {
        let plain = request
            .query(query::ReadSource {
                directory: event.into(),
            })
            .execute()
            .unwrap();
        let index = request.query(find(event, "")).execute().unwrap();
        assert_eq!(names(plain.rows()), wanted);
        assert_eq!(names(index.rows()), wanted);
    }
}

#[test]
fn refused_birth_releases_its_edit_and_preserves_the_retained_predecessor() {
    let (world, calls, _, mut key) = setup();
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
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let before = request.query(find("A", "")).execute().unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    key += 1;
    let refused = request
        .advanced()
        .mutate(history::Edit::Append {
            rename: None,
            event: "E".into(),
            previous: "A".into(),
            introduced: Some(("e".into(), "forbidden".into())),
            retired: Some("x".into()),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        refused.result().is_none(),
        "forbidden step must refuse: {refused:?}"
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        names(request.query(find("A", "")).execute().unwrap().rows()),
        names(before.rows())
    );
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            rename: None,
            event: "E".into(),
            previous: "A".into(),
            introduced: Some(("e".into(), "match-e".into())),
            retired: Some("x".into()),
        },
    );
    assert_eq!(
        names(request.query(find("E", "")).execute().unwrap().rows()),
        set(&["e"])
    );
}

#[test]
fn inherited_row_changes_update_both_scopes_without_rebuilding_after_birth() {
    let (world, calls, _, mut key) = setup();
    let builds = calls.builds.load(Ordering::SeqCst);
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            event: "E".into(),
            previous: "A".into(),
            introduced: None,
            retired: None,
            rename: None,
        },
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "E must inherit A's row"
    );
    publish(
        &world,
        &mut key,
        crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit::Name {
            item: "x".into(),
            value: "changed-inherited-row".into(),
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
    for event in ["A", "E"] {
        let indexed = request.query(find(event, "")).execute().unwrap();
        let direct = request
            .query(query::ReadSource {
                directory: event.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(indexed.rows(), direct.rows());
        assert_eq!(indexed.rows().len(), 1);
        assert_eq!(indexed.rows()[0].identity, "x");
        assert_eq!(indexed.rows()[0].name, "changed-inherited-row");
    }
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "both scopes must update their retained rows"
    );
}

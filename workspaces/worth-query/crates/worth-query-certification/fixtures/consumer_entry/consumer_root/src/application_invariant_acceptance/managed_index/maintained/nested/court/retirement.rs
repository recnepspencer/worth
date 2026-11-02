//! Physical retirement/rewiring and exact domain refusal preserve complete trees.
use super::*;
fn require_violation<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the descendant rule must refuse: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("setup, unavailable Source and request failure cannot counterfeit a violation: {denial:?}");
    };
    assert_eq!(identity.rule_id.as_str(), "NoForbiddenProjectedName");
}
#[test]
fn retired_descendants_do_not_reattach_by_text_identity_and_new_edges_keep_actual_records() {
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
    let original = request.query(find("A", "match")).execute().unwrap();
    let old = original.rows()[0].children[0].children[0].coordinate;
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Unlink {
            from: "a".into(),
            to: "z".into(),
        },
    );
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Unlink {
            from: "b".into(),
            to: "z".into(),
        },
    );
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Retire { item: "z".into() },
    );
    let no_leaf = request.query(find("A", "match")).execute().unwrap();
    assert_eq!(values(no_leaf.rows()), model.expected("A", "match", true));
    assert!(no_leaf.rows()[0]
        .children
        .iter()
        .all(|node| node.children.is_empty()));
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Item {
            directory: "catalog".into(),
            identity: "z".into(),
            name: "shared-leaf".into(),
            root: false,
            incarnation: 2,
        },
    );
    assert_eq!(
        exact(request.query(find("A", "match")).execute().unwrap().rows()),
        exact(no_leaf.rows()),
        "matching text cannot recreate retired edges"
    );
    link(&world, &mut model, &mut key, "a", "z", 2);
    link(&world, &mut model, &mut key, "b", "z", 2);
    let replacement = request.query(find("A", "match")).execute().unwrap();
    assert_eq!(
        values(replacement.rows()),
        model.expected("A", "match", true)
    );
    assert_ne!(
        replacement.rows()[0].children[0].children[0].coordinate,
        old,
        "a replacement has an actually different full physical identity"
    );
    assert!(
        replacement.rows()[0]
            .children
            .iter()
            .all(|node| node.children[0].children.is_empty()),
        "old z->c belongs to the retired physical z, not its text lookalike"
    );
    link(&world, &mut model, &mut key, "z", "c", 2);
    let restored = request.query(find("A", "match")).execute().unwrap();
    assert_eq!(values(restored.rows()), model.expected("A", "match", true));
    let source = request
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(exact(restored.rows()), exact(source.rows()));
}
#[test]
fn forbidden_leaf_reaches_exact_proposed_rule_and_refuses_publication_atomically() {
    let (world, _, records, mut model, mut key) = setup();
    for (from, to) in [("x", "a"), ("a", "z"), ("z", "c"), ("q", "x"), ("q", "p")] {
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
    let before = request.query(find("A", "match")).execute().unwrap();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    records.records.lock().unwrap().clear();
    key += 1;
    let bad = Edit::Name {
        item: "c".into(),
        value: "forbidden".into(),
    };
    let result = request
        .advanced()
        .mutate(bad.clone())
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    require_violation(result);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        exact(request.query(find("A", "match")).execute().unwrap().rows()),
        exact(before.rows())
    );
    let mut proposed = model.clone();
    proposed.apply(&bad);
    {
        let observed = records.records.lock().unwrap();
        assert!((1..=2).contains(&observed.len()));
        for record in observed.iter() {
            let directory = model.directory(record.scope);
            assert_eq!(
                values(record.before.as_ref().unwrap()),
                model.expected(directory, "", false)
            );
            assert_eq!(
                values(&record.after),
                proposed.expected(directory, "", false)
            );
        }
    }
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "c".into(),
            value: "allowed".into(),
        },
    );
    assert_eq!(
        values(request.query(find("A", "match")).execute().unwrap().rows()),
        model.expected("A", "match", true)
    );
}

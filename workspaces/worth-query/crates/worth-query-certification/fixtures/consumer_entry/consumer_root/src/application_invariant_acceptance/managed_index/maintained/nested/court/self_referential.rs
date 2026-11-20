//! A root row projecting a self-referential relation to another root row in
//! the same scope must survive a relation-only rewrite on that target.
use super::*;
use std::collections::BTreeSet;

fn row_coordinate(world: &host::Host, directory: &str, identity: &str) -> (u32, u64, u32) {
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
    let rows = request
        .query(query::ReadSource {
            directory: directory.into(),
        })
        .execute()
        .unwrap();
    rows.rows()
        .iter()
        .find(|row| row.identity == identity)
        .unwrap_or_else(|| panic!("{identity} is a live row in {directory}"))
        .coordinate
}

fn scope_observed(
    records: &rule::Calls,
    model: &Model,
    directory: &str,
) -> [BTreeSet<(u32, u64, u32)>; 2] {
    let target = model.scope(directory);
    let observed = records.records.lock().unwrap();
    let matching: Vec<_> = observed
        .iter()
        .filter(|record| record.scope == target)
        .collect();
    assert_eq!(matching.len(), 1, "expected one observation in {directory}");
    [
        matching[0].observed[0].iter().copied().collect(),
        matching[0].observed[1].iter().copied().collect(),
    ]
}

fn assert_scope_matches(
    records: &rule::Calls,
    model: &Model,
    directory: &str,
    expected_observed: [BTreeSet<(u32, u64, u32)>; 2],
    message: &str,
) {
    assert_eq!(
        scope_observed(records, model, directory),
        expected_observed,
        "{message}"
    );
    let observed = records.records.lock().unwrap();
    let record = observed
        .iter()
        .find(|record| record.scope == model.scope(directory))
        .unwrap();
    assert_eq!(values(&record.after), model.expected(directory, "", false));
}

#[test]
fn self_referential_root_projection_allows_unrelated_relation_only_rewrite_and_keeps_record_pullback(
) {
    let (world, _, records, mut model, mut key) = setup();
    // Place `a` as a root row in scope A beside `x`, and link a 4-hop chain
    // `x -> a -> b -> z -> c`. Both `x` and `a` are live rows in A: `x`
    // projects `a` (with fields) through `Children` up to `z`, while `a`
    // projects `b -> z -> c`.
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Share {
            directory: "A".into(),
            item: "a".into(),
        },
    );
    for (from, to) in [("x", "a"), ("a", "b"), ("b", "z"), ("z", "c")] {
        link(&world, &mut model, &mut key, from, to, 1);
    }
    let x = row_coordinate(&world, "A", "x");
    let a = row_coordinate(&world, "A", "a");

    // 1. Relation-only unlink of `z -> c`: changes `a`'s 3-hop projection
    // without touching `a`'s entity record or `x`'s 3-hop projection.
    records.records.lock().unwrap().clear();
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Unlink {
            from: "z".into(),
            to: "c".into(),
        },
    );
    assert_scope_matches(
        &records,
        &model,
        "A",
        [BTreeSet::from([a]), BTreeSet::from([a])],
        "only `a` changed; `x` must not be required or falsely refused",
    );

    // 2. Scalar field write on `a`: `a` is an entity-record change, so both
    // `a` and `x` (which projects `a.name`) must be pulled back and updated.
    records.records.lock().unwrap().clear();
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "a".into(),
            value: "branch-a-updated".into(),
        },
    );
    assert_scope_matches(
        &records,
        &model,
        "A",
        [BTreeSet::from([a, x]), BTreeSet::from([a, x])],
        "scalar edit on `a` pulls back both `a` and `x`",
    );

    // 3. Lifecycle retirement of `a`: `a` leaves scope A and `x` updates to
    // drop `a` from its projected children.
    records.records.lock().unwrap().clear();
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Retire { item: "a".into() },
    );
    assert_scope_matches(
        &records,
        &model,
        "A",
        [BTreeSet::from([a, x]), BTreeSet::from([x])],
        "retiring `a` removes `a` and updates `x`",
    );
}

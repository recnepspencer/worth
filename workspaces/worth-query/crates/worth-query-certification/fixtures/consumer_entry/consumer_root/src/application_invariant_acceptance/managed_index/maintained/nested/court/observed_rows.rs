//! A Source-scoped rule reads the rows Native observed changing its scope, by
//! entity identity, through each admitted view.
use super::*;
type Coordinate = (u32, u64, u32);

/// Each recorded invocation: its directory and the identities of the rows
/// the committed and proposed views lent, by the proposed and committed rows.
fn observed(records: &rule::Calls, model: &Model) -> Vec<(String, [Vec<Coordinate>; 2])> {
    let mut observed: Vec<_> = records
        .records
        .lock()
        .unwrap()
        .iter()
        .map(|record| {
            (
                model.directory(record.scope).to_owned(),
                record.observed.clone(),
            )
        })
        .collect();
    observed.sort();
    observed
}
/// The coordinate of `identity` among a scope's committed rows.
fn row(rows: &[query::ItemRow], identity: &str) -> Coordinate {
    rows.iter()
        .find(|row| row.identity == identity)
        .unwrap_or_else(|| panic!("{identity} is a row"))
        .coordinate
}
fn source(world: &host::Host, directory: &str) -> Vec<query::ItemRow> {
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
    rows.rows().to_vec()
}

#[test]
fn a_descendant_field_change_lends_each_scope_the_root_rows_that_project_it() {
    let (world, _, records, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "x", "a", 1);
    let x = row(&source(&world, "A"), "x");
    records.records.lock().unwrap().clear();
    let rename = Edit::Name {
        item: "a".into(),
        value: "renamed-branch".into(),
    };
    publish(&world, &mut model, &mut key, rename);
    // `a` is no root row: it reaches A and B (which shares x) only as x's
    // projected child, so each lends x, in both views, and D lends nothing.
    assert_eq!(
        observed(&records, &model),
        [
            ("A".to_owned(), [vec![x], vec![x]]),
            ("B".to_owned(), [vec![x], vec![x]]),
        ]
    );
}

#[test]
fn a_root_field_change_lends_only_that_row_in_its_own_scope() {
    let (world, _, records, mut model, mut key) = setup();
    let w = row(&source(&world, "D"), "w");
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "w".into(),
            value: "renamed-unrelated".into(),
        },
    );
    assert_eq!(
        observed(&records, &model),
        [("D".to_owned(), [vec![w], vec![w]])]
    );
}

/// A retired root is lent only where it is visible, the committed view, and
/// its replacement under the same identity text is another entity, lent only
/// by the proposed view of the commit that makes it.
#[test]
fn a_retired_root_and_its_same_identity_replacement_are_distinct_entities() {
    let (world, _, records, mut model, mut key) = setup();
    let old = row(&source(&world, "D"), "w");
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Retire { item: "w".into() },
    );
    assert_eq!(
        observed(&records, &model),
        [("D".to_owned(), [vec![old], vec![]])]
    );
    records.records.lock().unwrap().clear();
    let replacement = Edit::Item {
        directory: "D".into(),
        identity: "w".into(),
        name: "match-unrelated".into(),
        root: true,
        incarnation: 2,
    };
    publish(&world, &mut model, &mut key, replacement);
    let new = row(&source(&world, "D"), "w");
    assert_ne!(new, old, "the replacement is a new entity");
    assert_eq!(
        observed(&records, &model),
        [("D".to_owned(), [vec![], vec![new]])]
    );
}

/// A prepared handle kept from one invocation is refused in any other:
/// another scope of the same proposal and another scope of a later one fail
/// the invocation check; the same scope in a later proposal fails the
/// observation check. Each invocation's own handle reads its rows.
#[test]
fn a_prepared_handle_from_another_scope_or_proposal_is_refused() {
    let (world, _, records, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "x", "a", 1);
    let x = row(&source(&world, "A"), "x");
    let w = row(&source(&world, "D"), "w");
    records
        .probe_stale
        .store(true, std::sync::atomic::Ordering::SeqCst);
    records.records.lock().unwrap().clear();
    let rename = |item: &str, value: &str| Edit::Name {
        item: item.into(),
        value: value.into(),
    };
    // Scopes A and B: B's invocation tries A's handle.
    publish(&world, &mut model, &mut key, rename("a", "renamed-branch"));
    // Scope D of the next proposal tries B's handle.
    publish(
        &world,
        &mut model,
        &mut key,
        rename("w", "renamed-unrelated"),
    );
    // Scope D again, one proposal later, tries its own earlier handle.
    publish(&world, &mut model, &mut key, rename("w", "renamed-again"));
    let reads = records.stale_reads.lock().unwrap().clone();
    let refusal = |read: &Result<usize, String>| {
        let cause = read.as_ref().expect_err("a kept handle is refused");
        if cause.contains("ForeignObservation") {
            "observation"
        } else if cause.contains("ForeignSource") {
            "invocation"
        } else {
            panic!("an unexpected refusal: {cause}")
        }
    };
    assert_eq!(
        reads.iter().map(refusal).collect::<Vec<_>>(),
        ["invocation", "invocation", "observation"]
    );
    // The kept handle cannot mint its scope either, by the same checks.
    let scopes = records.stale_scopes.lock().unwrap().clone();
    let scope_refusal = |read: &Result<(u32, u64, u32), String>| {
        refusal(&read.as_ref().map(|_| 0).map_err(Clone::clone))
    };
    assert_eq!(
        scopes.iter().map(scope_refusal).collect::<Vec<_>>(),
        ["invocation", "invocation", "observation"]
    );
    // The receiving invocations' own handles read their rows.
    assert_eq!(
        observed(&records, &model),
        [
            ("A".to_owned(), [vec![x], vec![x]]),
            ("B".to_owned(), [vec![x], vec![x]]),
            ("D".to_owned(), [vec![w], vec![w]]),
            ("D".to_owned(), [vec![w], vec![w]]),
        ]
    );
}

/// Each invocation mints its own scope as a typed Directory: the entity is
/// the scope Native invoked, evaluation reads its own field through it, and
/// a mint under the Item binding, not the Source's scope kind, is refused.
#[test]
fn the_observed_scope_is_the_invocations_directory_and_is_readable_in_evaluation() {
    let (world, _, records, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "x", "a", 1);
    records.records.lock().unwrap().clear();
    records.scopes.lock().unwrap().clear();
    let rename = Edit::Name {
        item: "a".into(),
        value: "renamed-branch".into(),
    };
    publish(&world, &mut model, &mut key, rename);
    let scopes = records.scopes.lock().unwrap();
    let mut minted: Vec<_> = scopes
        .iter()
        .map(|record| {
            assert_eq!(
                record.minted, record.scope,
                "the minted entity is the scope"
            );
            let refused = record
                .wrong_kind
                .as_ref()
                .expect_err("an Item scope is refused");
            assert!(refused.contains("WrongEntityKind"), "{refused}");
            model.directory(record.scope).to_owned()
        })
        .collect();
    minted.sort();
    assert_eq!(minted, ["A", "B"]);
    let mut read: Vec<_> = records
        .records
        .lock()
        .unwrap()
        .iter()
        .map(|record| {
            (
                model.directory(record.scope).to_owned(),
                record.scope_name.clone(),
            )
        })
        .collect();
    read.sort();
    assert_eq!(
        read,
        [
            ("A".to_owned(), Some("A".to_owned())),
            ("B".to_owned(), Some("B".to_owned())),
        ]
    );
}

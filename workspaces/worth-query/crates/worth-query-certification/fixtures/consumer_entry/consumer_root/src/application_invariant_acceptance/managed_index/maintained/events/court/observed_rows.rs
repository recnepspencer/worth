//! An ordered birth lends only rows that are members of the born scope: an
//! Item born beside it in the same candidate, never linked, is no row of it.
use super::*;

#[test]
fn an_ordered_birth_lends_its_introduced_row_and_never_a_disconnected_same_kind_birth() {
    let (world, _, records, mut key) = setup();
    records.observed.lock().unwrap().clear();
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            event: "E".into(),
            previous: "A".into(),
            introduced: Some(("e".into(), "match-e".into())),
            retired: None,
            rename: None,
            stray: Some(("s".into(), "match-stray".into())),
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
    // The independent membership: E's ordinary Source rows.
    let source = request
        .query(query::ReadSource {
            directory: "E".into(),
        })
        .execute()
        .unwrap();
    let members: BTreeSet<_> = source.rows().iter().map(|row| row.coordinate).collect();
    let introduced = source
        .rows()
        .iter()
        .find(|row| row.identity == "e")
        .expect("the introduced row is a member")
        .coordinate;
    let observed = records.observed.lock().unwrap();
    let proposed: BTreeSet<_> = observed
        .iter()
        .flat_map(|(_, [_, proposed])| proposed.iter().copied())
        .collect();
    assert_eq!(proposed, BTreeSet::from([introduced]));
    assert!(proposed.is_subset(&members));
    assert!(
        observed
            .iter()
            .all(|(_, [committed, _])| committed.is_empty()),
        "the born scope has no committed member: {observed:?}"
    );
}

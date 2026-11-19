//! An ordered birth lends exactly the rows its own step makes members of the
//! born scope: an Item born beside it and never linked is no row of it, and
//! an existing Item its new link introduces is one.
use super::*;

/// E's ordinary Source rows: the independent membership.
fn source(world: &host::EventHost, event: &str) -> Vec<query::ItemRow> {
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
            directory: event.into(),
        })
        .execute()
        .unwrap();
    rows.rows().to_vec()
}
fn append(event: &str, previous: &str) -> history::Edit {
    history::Edit::Append {
        event: event.into(),
        previous: previous.into(),
        introduced: None,
        retired: None,
        rename: None,
        stray: None,
        adopt: None,
    }
}

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
            adopt: None,
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
        .filter(|(index, ..)| *index == "ordered")
        .flat_map(|(_, _, [_, proposed])| proposed.iter().copied())
        .collect();
    assert_eq!(proposed, BTreeSet::from([introduced]));
    assert!(proposed.is_subset(&members));
    assert!(
        observed
            .iter()
            .all(|(_, _, [committed, _])| committed.is_empty()),
        "the born scope has no committed member: {observed:?}"
    );
}

/// A new event's link introduces an Item that already lives, born unlinked by
/// an earlier event and so absent from its predecessor's Source; the Item's
/// own record is untouched.
#[test]
fn an_ordered_birth_lends_an_existing_row_its_new_link_introduces() {
    let (world, _, records, mut key) = setup();
    let mut born = append("E1", "A");
    if let history::Edit::Append { stray, .. } = &mut born {
        *stray = Some(("s".into(), "match-stray".into()));
    }
    edit_history(&world, &mut key, born);
    assert!(source(&world, "E1").iter().all(|row| row.identity != "s"));
    records.observed.lock().unwrap().clear();
    let mut adopting = append("E2", "E1");
    if let history::Edit::Append { adopt, .. } = &mut adopting {
        *adopt = Some("s".into());
    }
    edit_history(&world, &mut key, adopting);
    let rows = source(&world, "E2");
    let adopted = rows
        .iter()
        .find(|row| row.identity == "s")
        .expect("the adopted Item is a member")
        .coordinate;
    let observed = records.observed.lock().unwrap();
    let proposed: BTreeSet<_> = observed
        .iter()
        .filter(|(index, ..)| *index == "ordered")
        .flat_map(|(_, _, [_, proposed])| proposed.iter().copied())
        .collect();
    assert_eq!(proposed, BTreeSet::from([adopted]));
}

/// C2 inherits A's row x, sharing A's entry, which names A's scope. A
/// redundant introduction of x at C2 makes maintenance rematerialize x under
/// C2 with the same facts: it is not lent. Renaming x then is, in C2.
#[test]
fn an_inherited_row_rematerialized_with_the_same_facts_is_not_lent() {
    let (world, _, records, mut key) = setup();
    edit_history(&world, &mut key, append("C2", "A"));
    let x = source(&world, "C2")
        .iter()
        .find(|row| row.identity == "x")
        .expect("C2 inherits x")
        .coordinate;
    let lent = |records: &rule::Calls| {
        records
            .observed
            .lock()
            .unwrap()
            .iter()
            .filter(|(index, ..)| *index == "ordered")
            .flat_map(|(_, _, [committed, proposed])| committed.iter().chain(proposed).copied())
            .collect::<BTreeSet<_>>()
    };
    records.observed.lock().unwrap().clear();
    edit_history(
        &world,
        &mut key,
        history::Edit::Introduction {
            event: "C2".into(),
            item: "x".into(),
            introduced: true,
            edition: 1,
        },
    );
    assert!(
        !lent(&records).contains(&x),
        "a rematerialized inherited row with equal facts is unchanged"
    );
    records.observed.lock().unwrap().clear();
    let rename = crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit::Name {
        item: "x".into(),
        value: "match-renamed".into(),
    };
    publish(&world, &mut key, rename);
    assert!(lent(&records).contains(&x), "a field change is lent");
}

//! An ordered successor mints its own scope through `observed_scope`, never
//! its predecessor's, and evaluation reads the minted Directory.
use super::*;

#[test]
fn an_ordered_successor_mints_its_own_scope_and_reads_it_in_evaluation() {
    let (world, _, records, mut key) = setup();
    records.scopes.lock().unwrap().clear();
    edit_history(
        &world,
        &mut key,
        history::Edit::Append {
            event: "E".into(),
            previous: "A".into(),
            introduced: Some(("e".into(), "match-e".into())),
            retired: None,
            rename: None,
            stray: None,
            adopt: None,
        },
    );
    let scopes = records.scopes.lock().unwrap();
    for (scope, minted, name) in scopes.iter() {
        assert_eq!(minted, scope, "{name:?} minted another scope");
    }
    // E, born after A in this proposal with no committed counterpart, mints
    // and reads itself; its predecessor A is not what it receives.
    let born: Vec<_> = scopes
        .iter()
        .filter_map(|(_, _, name)| name.as_deref())
        .collect();
    assert!(
        born.contains(&"E"),
        "E is invoked and reads itself: {born:?}"
    );
    assert!(
        !born.contains(&"A"),
        "the unchanged predecessor is not invoked: {born:?}"
    );
}

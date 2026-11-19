//! Explicit initialization of an ordered chain lends each seeded scope its
//! delta relative to its predecessor: the chain root every member, the
//! successor only its own introduction, never the inherited row.
use super::*;

#[test]
fn an_initialized_ordered_chain_lends_each_seeded_scope_its_own_step() {
    let records = Arc::new(rule::Calls::default());
    let world = host::install_events_seeded(Arc::clone(&records));
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
    let rows = |directory: &str| {
        request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap()
            .rows()
            .iter()
            .map(|row| (row.coordinate, row.identity.clone()))
            .collect::<Vec<_>>()
    };
    let (a, e) = (rows("A"), rows("E"));
    let identities = |rows: &[(rule::Coordinate, String)]| {
        let mut held: Vec<_> = rows.iter().map(|(_, identity)| identity.clone()).collect();
        held.sort_unstable();
        held
    };
    assert_eq!(identities(&a), ["x"], "A holds x");
    assert_eq!(identities(&e), ["e", "x"], "E holds e and inherits x");
    let identity = |coordinate: &rule::Coordinate| {
        e.iter()
            .find(|(row, _)| row == coordinate)
            .map(|(_, identity)| identity.clone())
            .unwrap_or_else(|| panic!("{coordinate:?} is a seeded row"))
    };
    let mut lent: Vec<[Vec<String>; 2]> = records
        .observed
        .lock()
        .unwrap()
        .iter()
        .filter(|(index, ..)| *index == "ordered")
        .map(|(_, _, sides)| {
            sides
                .clone()
                .map(|side| side.iter().map(identity).collect())
        })
        .collect();
    lent.sort();
    // Only E holds e, so the loan of e is E's; A, the chain root, lends x.
    assert_eq!(
        lent,
        [
            [vec![], vec!["e".to_owned()]],
            [vec![], vec!["x".to_owned()]],
        ],
        "each seeded scope lends its own step"
    );
}

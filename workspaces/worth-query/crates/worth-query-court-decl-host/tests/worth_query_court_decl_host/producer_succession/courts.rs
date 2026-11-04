//! Literal branch rosters and generation observations through real producer settlement.
use super::{settlement::*, *};
#[test]
fn generator_shrink_retires_the_deleted_childs_owned_publications() {
    let app = world::install_two_levels();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 1);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1"]);
    assert_live(&request, &["root", "root.0"]);
}
#[test]
fn three_level_shrink_retires_exactly_the_removed_branch_and_regrows_fresh() {
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let old = child_id(&first, "root.1");
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 2);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
    resize(&app, &request, 2, 3);
    let regrown = settle(&app, &request);
    assert_ne!(child_id(&regrown, "root.1"), old);
    assert_eq!(child_id(&regrown, "root.0"), sibling);
    close(regrown, &request);
    assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
    let selected = selections.lock().unwrap();
    let regrown = selected
        .iter()
        .filter(|s| s.key == "root.1")
        .collect::<Vec<_>>();
    assert_eq!(regrown.len(), 2);
    assert!(regrown
        .iter()
        .all(|s| s.tier == 1 && !s.preserve && !s.prior));
}

//! Real generator cascade with Native-owned private memberships.
use super::*;
#[test]
fn generator_cascade_derives_private_membership_deletions_and_regrows_fresh() {
    let app = world::install(2, Selections::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settlement::settle(&app, &request);
    let old = settlement::child_id(&first, "root.1");
    let sibling = settlement::child_id(&first, "root.0");
    settlement::close(first, &request);
    settlement::resize(&app, &request, 1, 50);
    let shrunk = settlement::settle(&app, &request);
    assert_eq!(settlement::child_id(&shrunk, "root.0"), sibling);
    settlement::close(shrunk, &request);
    settlement::assert_absent(&request, &["root.1", "root.1.0"]);
    settlement::assert_live(&request, &["root", "root.0", "root.0.0"]);
    settlement::resize(&app, &request, 2, 51);
    let regrown = settlement::settle(&app, &request);
    assert_ne!(settlement::child_id(&regrown, "root.1"), old);
    assert_eq!(settlement::child_id(&regrown, "root.0"), sibling);
    settlement::close(regrown, &request);
    settlement::assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
}

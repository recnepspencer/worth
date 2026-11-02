use super::split_support::*;
use super::*;
#[test]
fn sibling_sources_have_distinct_realizations_and_immediate_parents() {
    // Each subject: Initial only. root has two children; each child has one leaf.
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let settled = settle(&app, &request);
    assert_ne!(child_id(&settled, "root.0"), child_id(&settled, "root.1"));
    for key in ["root", "root.0", "root.1", "root.0.0", "root.1.0"] {
        sequence(&selections, key, &[(false, false)]);
    }
    for key in ["root.0", "root.1"] {
        let source = request.query(Read(key.into())).execute().unwrap();
        assert_eq!(source.rows()[0].generated_by, "root");
        assert_eq!(value(&request, key), 12);
        let leaf = request.query(Read(format!("{key}.0"))).execute().unwrap();
        assert_eq!(leaf.rows()[0].generated_by, key);
        assert_eq!(value(&request, &format!("{key}.0")), 13);
    }
    close(settled, &request);
}
#[test]
fn corrected_child_input_preserves_realization_identity_and_recomputes_its_leaf() {
    // Initial on all three subjects. Authored root input 1 -> 8 makes its owner
    // correct child input 2 -> 9, then child corrects leaf 3 -> 10: Preserve each.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let first = settle(&app, &request);
    let child = child_id(&first, "root.0");
    edit(&app, &request, "root", 8, 1, 101);
    assert!(
        first.deliver_current(&request).is_err(),
        "old settlement cannot adopt a corrected parent input"
    );
    let corrected = settle(&app, &request);
    assert_eq!(child_id(&corrected, "root.0"), child);
    assert_eq!(value(&request, "root.0"), 19);
    assert_eq!(value(&request, "root.0.0"), 20);
    for key in ["root", "root.0", "root.0.0"] {
        sequence(&selections, key, &[(false, false), (true, true)]);
    }
    close(first, &request);
    close(corrected, &request);
}
#[test]
fn parent_membership_growth_preserves_existing_children_and_initializes_new_subjects() {
    // root: Initial -> Preserve. Unchanged child0/leaf0 reuse their Initial.
    // New child1 and leaf1: Initial.
    let selections = Selections::default();
    let app = world::install(1, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let first = settle(&app, &request);
    let child = child_id(&first, "root.0");
    edit(&app, &request, "root", 1, 2, 102);
    assert!(first.deliver_current(&request).is_err());
    let grown = settle(&app, &request);
    assert_eq!(child_id(&grown, "root.0"), child);
    assert_ne!(child_id(&grown, "root.1"), child);
    sequence(&selections, "root", &[(false, false), (true, true)]);
    for key in ["root.0", "root.0.0"] {
        sequence(&selections, key, &[(false, false)]);
    }
    for key in ["root.1", "root.1.0"] {
        sequence(&selections, key, &[(false, false)]);
    }
    assert_eq!(value(&request, "root.1.0"), 13);
    close(first, &request);
    close(grown, &request);
}
#[test]
fn retirement_and_restore_reissue_complete_child_publications() {
    // root/child0/leaf0: Initial only, reused. Retired child1/leaf1: Initial -> Initial.
    // Each retirement covers one complete publication: leaf realization, then
    // child realization plus its generated leaf source. The parent-owned child source stays.
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let first = settle(&app, &request);
    let retired = child_id(&first, "root.1");
    control(
        &app,
        &request,
        source_control::Control {
            key: "root.1.0".into(),
            change: source_control::Change::RetirePublication,
        },
        103,
    );
    control(
        &app,
        &request,
        source_control::Control {
            key: "root.1".into(),
            change: source_control::Change::RetirePublication,
        },
        104,
    );
    for key in ["root.1", "root.1.0"] {
        assert_eq!(
            request.query(Read(key.into())).execute().is_ok(),
            key == "root.1"
        );
        assert!(request
            .query(realization_read::ReadRealization(key.into()))
            .execute()
            .is_err());
    }
    assert!(first.deliver_current(&request).is_err());
    let restored = settle(&app, &request);
    assert_ne!(child_id(&restored, "root.1"), retired);
    assert_eq!(value(&request, "root.1.0"), 13);
    for key in ["root.1", "root.1.0"] {
        sequence(&selections, key, &[(false, false), (false, false)]);
    }
    for key in ["root", "root.0", "root.0.0"] {
        sequence(&selections, key, &[(false, false)]);
    }
    close(first, &request);
    close(restored, &request);
}

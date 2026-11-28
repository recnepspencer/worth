use super::*;

#[test]
fn cas_refuses_a_sibling_and_a_skipped_parent_before_materialization() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let initial = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let cell = product_cell(initial.clone(), &catalog, &fixture);
    let first = fixture::install_ordinary(&mut fixture, &catalog, &root);
    let first_snapshot = fixture::successor_snapshot(&initial, Arc::clone(&first));
    cell.compare_and_publish(
        &observation(&cell, &fixture, &catalog),
        fixture::product_head_protection(&fixture, &catalog, first_snapshot.clone()),
    )
    .unwrap();

    let expected = observation(&cell, &fixture, &catalog);
    let sibling = fixture::install_ordinary(&mut fixture, &catalog, &root);
    let next = fixture::install_ordinary(&mut fixture, &catalog, &first);
    let grandchild = fixture::install_ordinary(&mut fixture, &catalog, &next);
    for commit in [sibling, grandchild] {
        let candidate = fixture::successor_snapshot(&first_snapshot, commit);
        let loss = cell
            .replace_expected_permitted(
                &expected,
                &candidate,
                &mut (),
                |_| panic!("invalid lineage must refuse before materialization"),
                None,
                None,
            )
            .unwrap_err();
        assert_eq!(
            loss.denial,
            ProductBranchReferenceCellDenial::FastForwardRequired
        );
        assert_eq!(loss.observed_head(), &first_snapshot);
        assert_eq!(cell.atomic_snapshot(), first_snapshot);
    }

    let candidate = fixture::successor_snapshot(&first_snapshot, next);
    cell.compare_and_publish(
        &expected,
        fixture::product_head_protection(&fixture, &catalog, candidate.clone()),
    )
    .unwrap();
    assert_eq!(cell.atomic_snapshot(), candidate);
}

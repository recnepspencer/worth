use super::*;

#[test]
fn unique_point_edits_refuse_actual_aliases_before_mutation() {
    let fixture = Fixture::new();
    let mut map = fixture.map(&[2, 1, 3]);
    let snapshot = map.clone();
    let ids = identities(&map);
    let original: BTreeMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
    let cell = fixture.cell(0);
    let cell_ids = cell_identities(&cell);
    let held = fixture.budget.charged_memory_bytes();
    assert_eq!(
        map.set_existing_unique(&2, 90),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    );
    assert!(matches!(
        map.remove_retaining_unique(&2),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    ));
    let (cause, cell) = map.insert_vacant_prepared(cell).unwrap_err();
    assert_eq!(cause, ExecutionOrderedMapUniqueDenial::SharedStorage);
    assert_eq!(cell_identities(&cell), cell_ids);
    assert_map(&map, &original);
    assert_eq!(identities(&map), ids);
    assert_eq!(snapshot, map);
    assert_eq!(fixture.budget.charged_memory_bytes(), held);
    drop(snapshot);

    let child = Arc::clone(map.root.as_ref().unwrap().left.as_ref().unwrap());
    assert!(Arc::get_mut(map.root.as_mut().unwrap()).is_some());
    assert_eq!(
        map.set_existing_unique(&1, 90),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    );
    assert!(matches!(
        map.remove_retaining_unique(&1),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    ));
    let (cause, cell) = map.insert_vacant_prepared(cell).unwrap_err();
    assert_eq!(cause, ExecutionOrderedMapUniqueDenial::SharedStorage);
    assert_eq!(cell_identities(&cell), cell_ids);
    assert_map(&map, &original);
    assert_eq!(identities(&map), ids);
    drop(child);

    let entry = Arc::clone(&map.root.as_ref().unwrap().entry);
    assert!(Arc::get_mut(map.root.as_mut().unwrap()).is_some());
    assert_eq!(
        map.set_existing_unique(&2, 90),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    );
    assert!(matches!(
        map.remove_retaining_unique(&2),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    ));
    assert_map(&map, &original);
    assert_eq!(identities(&map), ids);
    drop(entry);

    let occupied = fixture.cell(2);
    let occupied_ids = cell_identities(&occupied);
    let (cause, occupied) = map.insert_vacant_prepared(occupied).unwrap_err();
    assert_eq!(cause, ExecutionOrderedMapUniqueDenial::Occupied);
    assert_eq!(cell_identities(&occupied), occupied_ids);
    map.insert_vacant_prepared(cell).unwrap();
    assert_eq!(map.set_existing_unique(&1, 90), Ok(Some(7)));
    drop(occupied);
    drop(map);
    assert_eq!(fixture.budget.charged_memory_bytes(), 0);
}

#[test]
fn unique_removal_checks_off_path_rotation_owners() {
    for (keys, root_key, removed_key, pivot, left_heavy) in [
        (vec![4, 2, 6, 1, 3], 4, 6, false, true),
        (vec![4, 2, 6, 3], 4, 6, true, true),
        (vec![2, 0, 4, 3, 5], 2, 0, false, false),
        (vec![2, 0, 4, 3], 2, 0, true, false),
    ] {
        let fixture = Fixture::new();
        let mut map = fixture.map(&keys);
        assert_eq!(map.root.as_ref().unwrap().entry.key, root_key);
        let root = map.root.as_ref().unwrap();
        let child = if left_heavy { &root.left } else { &root.right }
            .as_ref()
            .unwrap();
        let inner = if left_heavy {
            &child.right
        } else {
            &child.left
        };
        let alias = Arc::clone(if pivot {
            inner.as_ref().unwrap()
        } else {
            child
        });
        let ids = identities(&map);
        let expected: BTreeMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
        let held = fixture.budget.charged_memory_bytes();
        assert!(matches!(
            map.remove_retaining_unique(&removed_key),
            Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
        ));
        assert_map(&map, &expected);
        assert_eq!(identities(&map), ids);
        assert_eq!(fixture.budget.charged_memory_bytes(), held);
        drop(alias);
        let removed = map.remove_retaining_unique(&removed_key).unwrap().unwrap();
        let mut expected = expected;
        expected.remove(&removed_key);
        assert_map(&map, &expected);
        drop(removed);
        drop(map);
        assert_eq!(fixture.budget.charged_memory_bytes(), 0);
    }
}

#[test]
fn unique_two_child_removal_checks_successor_path_before_effects() {
    let fixture = Fixture::new();
    let mut map = fixture.map(&[4, 2, 6, 1, 3, 5, 7]);
    let root = map.root.as_ref().unwrap();
    assert_eq!(root.entry.key, 4);
    let original_pair = [
        (0, Arc::as_ptr(root) as usize),
        (1, Arc::as_ptr(&root.entry) as usize),
    ];
    let successor = Arc::clone(root.right.as_ref().unwrap().left.as_ref().unwrap());
    let before = identities(&map);
    let expected = map.iter().map(|(k, v)| (*k, *v)).collect();
    assert!(matches!(
        map.remove_retaining_unique(&4),
        Err(ExecutionOrderedMapUniqueDenial::SharedStorage)
    ));
    assert_eq!(identities(&map), before);
    assert_map(&map, &expected);
    drop(successor);
    let retired = map.remove_retaining_unique(&4).unwrap().unwrap();
    assert_eq!(cell_identities(&retired), original_pair);
    drop(retired);
    drop(map);
    assert_eq!(fixture.budget.charged_memory_bytes(), 0);
}

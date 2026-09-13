use super::{PersistentHashMap, PersistentHashMapStorage};

#[test]
fn overlay_only_insert_remove_churn_retains_no_historical_tombstones() {
    for churn_count in [64_u64, 4_096, 65_536] {
        let mut source = PersistentHashMap::<u64, u64>::new();
        source.insert(0, 7);
        let mut fork = source.fork_persistent();
        assert!(source.ptr_eq(&fork), "scale {churn_count} must share");

        for key in 1..=churn_count {
            assert_eq!(fork.get_mut(&key), None);
            assert_eq!(fork.insert(key, key), None);
            assert_eq!(fork.remove(&key), Some(key));
            assert_eq!(fork.get_mut(&key), None);
        }

        assert_eq!(fork.len(), 1);
        assert_eq!(fork.get(&0), Some(&7));
        let PersistentHashMapStorage::ForkShared { changes, .. } = &fork.storage else {
            panic!("fork must retain shared storage");
        };
        assert!(
            changes.is_empty(),
            "scale {churn_count} overlay-only churn must erase its delta"
        );
        assert_eq!(source.get(&0), Some(&7));
        assert_eq!(source.len(), 1);
    }
}

#[test]
fn logical_clone_preserves_shared_storage_and_isolation() {
    let mut source: PersistentHashMap<u64, u64> = (0..128).map(|key| (key, key)).collect();
    let mut current = source.fork_persistent();
    current.insert(7, 700);
    let baseline = current.clone();

    assert!(current.ptr_eq(&baseline));
    current.insert(7, 7_000);
    current.remove(&3);
    assert_eq!(source.get(&7), Some(&7));
    assert_eq!(baseline.get(&7), Some(&700));
    assert_eq!(baseline.get(&3), Some(&3));
    assert_eq!(current.get(&7), Some(&7_000));
    assert_eq!(current.get(&3), None);
}

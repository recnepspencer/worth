use super::PersistentHashMap;

#[test]
fn equal_contents_have_key_order_bytes_despite_insertion_and_fork_history() {
    let forward: PersistentHashMap<u64, u64> = (0..32).map(|key| (key, key)).collect();
    let reverse: PersistentHashMap<u64, u64> = (0..32).rev().map(|key| (key, key)).collect();
    let mut shared = forward.clone();
    let mut fork = shared.fork_persistent();
    assert_eq!(fork.remove(&7), Some(7));
    fork.insert(7, 7);
    fork.insert(31, 99);
    fork.insert(31, 31);
    assert_eq!(forward, reverse);
    assert_eq!(forward, fork);

    let bytes = serde_json::to_vec(&forward).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&reverse).unwrap());
    assert_eq!(bytes, serde_json::to_vec(&fork).unwrap());
    let ordered = format!(
        "{{{}}}",
        (0..32)
            .map(|key| format!("\"{key}\":{key}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(bytes, ordered.as_bytes());

    // The previous writer could emit any order; its map encoding still reads.
    let old_order = format!(
        "{{{}}}",
        (0..32)
            .rev()
            .map(|key| format!("\"{key}\":{key}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    let decoded: PersistentHashMap<u64, u64> = serde_json::from_str(&old_order).unwrap();
    assert_eq!(decoded, forward);
}

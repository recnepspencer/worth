//! A missing interval start must still find a predecessor held in an internal node.
use super::{entry_handle, SharedKey};
use std::collections::BTreeMap;

#[test]
fn borrowed_predecessors_match_reference_across_internal_boundaries() {
    let reference: BTreeMap<u64, u64> = (0..8_192).map(|key| (key * 8 + 1, key)).collect();
    let map: im::OrdMap<_, _> = reference
        .iter()
        .map(|(&key, &value)| (SharedKey::new(key), value))
        .collect();
    for query in 0..65_536 {
        if reference.contains_key(&query) {
            continue;
        }
        let expected = reference
            .range(..query)
            .next_back()
            .map(|(&key, &value)| (key, value));
        let actual = entry_handle::previous_after_exact_miss(&map, &query)
            .map(|(key, &value)| (*key.as_key(), value));
        assert_eq!(actual, expected, "query {query}");
    }
}

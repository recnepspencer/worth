use super::DigestIndex;
use proptest::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn commitment_matches_sorted_content_reconstruction(
        actions in prop::collection::vec((any::<u128>(), prop::option::of(any::<[u8;32]>())), 0..150),
    ) {
        let mut index = DigestIndex::default();
        let mut expected = BTreeMap::new();
        for (key, value) in actions {
            let retained = index.clone();
            let before = retained.digest();
            index.set(key, value);
            if let Some(value) = value { expected.insert(key, value); } else { expected.remove(&key); }
            prop_assert_eq!(index.digest(), reference(&expected.iter().map(|(&k, &v)| (k,v)).collect::<Vec<_>>()));
            prop_assert_eq!(retained.digest(), before);
        }
        let mut reversed = DigestIndex::default();
        for (&key, &value) in expected.iter().rev() { reversed.set(key, Some(value)); }
        prop_assert_eq!(reversed.digest(), index.digest());
        for key in expected.keys() { index.set(*key, None); }
        prop_assert_eq!(index.digest(), reference(&[]));
        prop_assert_eq!(index.allocation_bytes(), 0);
    }
}

// Independent sorted-key reconstruction; no production node, insertion, or
// balancing helper participates in the expected commitment.
fn reference(entries: &[(u128, [u8; 32])]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.content-index.v1\0");
    match entries.len() {
        0 => hash.update([0]),
        1 => {
            hash.update([1]);
            hash.update(entries[0].0.to_be_bytes());
            hash.update(entries[0].1);
        }
        _ => {
            let bit = (entries[0].0 ^ entries.last().unwrap().0).leading_zeros();
            let split = entries.partition_point(|(key, _)| key & (1 << (127 - bit)) == 0);
            hash.update([2]);
            hash.update(bit.to_be_bytes());
            hash.update(reference(&entries[..split]));
            hash.update(reference(&entries[split..]));
        }
    }
    hash.finalize().into()
}

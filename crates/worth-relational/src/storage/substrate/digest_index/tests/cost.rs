use super::*;
use crate::execution::phase_observation::observe_maintenance_work;

#[test]
fn batched_dirty_paths_hash_shared_prefixes_once() {
    let mut predecessor = DigestIndex::default();
    for key in 0..2048 {
        predecessor.set(key, Some([1; 32])).unwrap();
    }
    predecessor.digest();
    let edit = |eager| {
        let mut next = predecessor.clone();
        for key in 0..512 {
            next.set(key, Some([2; 32])).unwrap();
            if eager {
                std::hint::black_box(next.digest());
            }
        }
        next.digest()
    };
    let (batched, batch_work) = observe_maintenance_work(0, || edit(false));
    let (eager, eager_work) = observe_maintenance_work(0, || edit(true));
    assert_eq!(batched, eager);
    assert!(
        batch_work.content_index_hash_bytes * 2 < eager_work.content_index_hash_bytes,
        "shared ancestors must be coalesced: {batch_work:?} vs {eager_work:?}"
    );
}

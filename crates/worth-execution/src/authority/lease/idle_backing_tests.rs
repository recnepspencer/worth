//! Idle-ledger ownership; no process-global authority or allocator tolerance.
use super::*;
use worth_foundational::ExecutionBudget;

fn authority() -> ExecutionAuthority {
    ExecutionAuthority {
        inner: Arc::new(AuthorityInner {
            config: ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::MIN,
                charged_memory_bytes: Some(1_024),
            },
            pool: None,
            equivalences: EquivalenceRegistry::new([]).unwrap(),
            ledger: Mutex::new(Ledger::default()),
        }),
    }
}

fn request() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(1_024), None),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn assert_idle(authority: &ExecutionAuthority) {
    let ledger = authority.inner.ledger.lock().unwrap();
    assert_eq!((ledger.active_workers, ledger.charged_memory_bytes), (0, 0));
    assert!(ledger.nodes.is_empty());
    assert_eq!(ledger.nodes.capacity(), 0, "idle backing must be released");
}

#[test]
fn retained_root_survives_child_churn_until_the_final_hold_releases_backing() {
    let authority = authority();
    let root = authority.request_lease(request()).unwrap();
    let root_hold = root.reserve_memory(17).unwrap();
    let holds = (0..64)
        .map(|_| root.child(request()).unwrap().reserve_memory(1).unwrap())
        .collect::<Vec<_>>();
    let capacity = authority.inner.ledger.lock().unwrap().nodes.capacity();
    assert!(capacity >= 65);
    assert_eq!(root.charged_memory_bytes(), 81);
    drop(holds);
    assert_eq!(root.charged_memory_bytes(), 17);
    {
        let ledger = authority.inner.ledger.lock().unwrap();
        assert_eq!(ledger.nodes.len(), 1);
        assert_ne!(ledger.nodes.capacity(), 0);
    }
    drop(root_hold);
    assert_idle(&authority);
    // The authority and lease remain usable after an idle transition.
    let mut next = root.reserve_memory(23).unwrap();
    next.resize(0).unwrap();
    assert_idle(&authority);
    drop(next);
    assert_idle(&authority);
}

#[test]
fn zero_rebind_releases_only_idle_backing_and_preserves_live_workers() {
    let authority = authority();
    let root = authority.request_lease(request()).unwrap();
    let child = root.child(request()).unwrap();
    let mut memory = root.reserve_memory(7).unwrap();
    memory.take_over(Some(&child), None, 0).unwrap();
    assert_idle(&authority);

    let worker = root.reserve_worker_entry().unwrap();
    let mut retained = child.reserve_retained_memory(11).unwrap();
    root.rebind_retained_memory(&mut retained, 0).unwrap();
    {
        let ledger = authority.inner.ledger.lock().unwrap();
        assert_eq!((ledger.active_workers, ledger.charged_memory_bytes), (1, 0));
        assert_eq!(ledger.nodes.len(), 1);
    }
    drop(retained);
    drop(worker);
    assert_idle(&authority);

    let mut retained = child.reserve_retained_memory(13).unwrap();
    root.rebind_retained_memory(&mut retained, 0).unwrap();
    assert_idle(&authority);
}

#[test]
fn an_empty_tombstone_table_retires_even_when_reported_capacity_is_zero() {
    use std::hash::BuildHasher;
    // This is the ledger's physical map protocol, not a fabricated lease.
    // Choose real keys using this map's own randomized hasher, so the occupied
    // buckets form one SwissTable probe cluster independently of its random seed.
    let mut ledger = Ledger::default();
    ledger.nodes.reserve(28);
    assert_eq!(ledger.nodes.capacity(), 28);
    let mut groups = [[0_u64; 28]; 32];
    let mut counts = [0_usize; 32];
    let keys = (0_u64..=28 * 32)
        .find_map(|key| {
            let bucket = ledger.nodes.hasher().hash_one(key) as usize & 31;
            groups[bucket][counts[bucket]] = key;
            counts[bucket] += 1;
            (counts[bucket] == 28).then_some(groups[bucket])
        })
        .expect("pigeonhole bound guarantees a cluster for every randomized seed");
    for key in keys {
        ledger.nodes.insert(key, NodeUsage::default());
    }
    for key in keys {
        assert!(ledger.nodes.remove(&key).is_some());
    }
    assert!(ledger.nodes.is_empty());
    assert_eq!(
        ledger.nodes.capacity(),
        0,
        "deleted buckets exhaust logical insertion room without freeing the allocated table"
    );
    let backing = ledger
        .take_unused_nodes()
        .expect("physical backing must leave the idle owner even with zero reported capacity");
    assert!(backing.is_empty());
    drop(backing);
}

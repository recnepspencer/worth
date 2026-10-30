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

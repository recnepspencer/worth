use super::super::*;
use worth_foundational::ExecutionBudget;

fn one_worker_authority() -> ExecutionAuthority {
    ExecutionAuthority {
        inner: Arc::new(AuthorityInner {
            config: ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::MIN,
                charged_memory_bytes: None,
            },
            pool: None,
            equivalences: EquivalenceRegistry::new([]).unwrap(),
            ledger: Mutex::new(Ledger::default()),
        }),
    }
}

/// An entry that finds every worker taken runs inline and charges its lease
/// nothing. Releasing the slot it could not take empties the lease's usage;
/// the inline entry's own release must not then expect that usage to exist.
#[test]
fn an_inline_entry_outlives_the_release_of_the_busy_worker_it_could_not_take() {
    let authority = one_worker_authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, Some(100)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let slot = lease.try_reserve_workers(1).unwrap();
    let inline = lease.reserve_entry(0).unwrap();
    assert_eq!(inline.process_workers, 0, "the only worker is taken");
    drop(slot);
    drop(inline);
    let ledger = authority.inner.ledger.lock().unwrap();
    assert_eq!((ledger.active_workers, ledger.charged_memory_bytes), (0, 0));
    assert!(ledger.nodes.is_empty(), "no released usage stays behind");
}

/// A zero-byte inline entry made while the only worker was taken releases
/// cleanly after that worker, and leaves the lease's memory fully reusable.
#[test]
fn zero_byte_inline_entry_outlives_the_worker_that_made_it_inline() {
    let lease = ExecutionResourceLease::serial(crate::tests::request(1, 1024, 100)).unwrap();
    let worker = lease.reserve_worker_entry().unwrap();
    let inline = lease.reserve_entry(0).unwrap();
    drop(worker);
    drop(inline);
    assert_eq!(lease.charged_memory_bytes(), 0);
    assert!(lease.reserve_memory(1024).is_ok());
}

use crate::tests::request;

#[test]
fn busy_zero_byte_entry_survives_the_charged_entry() {
    let lease = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    let charged = lease.reserve_entry(8).unwrap();
    let inline = lease.reserve_entry(0).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 8);
    drop(charged);
    assert_eq!(lease.charged_memory_bytes(), 0);
    drop(inline);
    let replacement = lease.reserve_entry(32).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 32);
    drop(replacement);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn busy_zero_byte_entry_can_rebind_after_the_charged_entry_leaves() {
    let lease = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    let charged = lease.reserve_entry(8).unwrap();
    let mut inline = lease.reserve_entry(0).unwrap();
    drop(charged);
    lease.rebind_retained_memory(&mut inline, 16).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 16);
    drop(inline);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

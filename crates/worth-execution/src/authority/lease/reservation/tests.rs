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

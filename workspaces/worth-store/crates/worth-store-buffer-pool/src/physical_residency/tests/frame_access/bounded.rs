use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::*;

fn bounded_key(identity: StableStoreIdentity, block: u64, limit: u32) -> PhysicalBoundedFrameKey {
    PhysicalBoundedFrameKey::new(
        identity,
        RecordArtifactFile::RootRoutingBlock {
            generation: 1,
            block,
        },
        NonZeroU32::new(limit).unwrap(),
    )
}

fn expect_bounded_fault(
    pool: &PhysicalResidencyPool,
    allocation: &OperationAllocationGrant,
    key: PhysicalBoundedFrameKey,
) -> PhysicalBoundedFrameFaultOwner {
    match pool.access_bounded_frame(allocation, key).unwrap() {
        PhysicalBoundedFrameAccess::Fault(owner) => owner,
        PhysicalBoundedFrameAccess::Hit(_) => panic!("expected bounded fault, found hit"),
        PhysicalBoundedFrameAccess::Coalesced(_) => {
            panic!("expected bounded fault owner, found waiter")
        }
    }
}

#[test]
fn bounded_fault_reserves_before_source_then_shrinks_and_hits_without_source() {
    let identity = store(21);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let key = bounded_key(identity, 1, 64);
    let admitted_metadata = pool.counters().metadata_bytes();
    let owner = expect_bounded_fault(&pool, &allocation, key);
    assert_eq!(pool.counters().resident_bytes(), 64);

    let lease = owner
        .load(
            |limit| {
                assert_eq!(limit, 64);
                Ok::<_, ()>(32)
            },
            |target| {
                target.fill(9);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    assert_eq!(lease.key().coordinate().length(), 32);
    assert_eq!(pool.counters().resident_bytes(), 32);
    drop(lease);

    let hit = match pool.access_bounded_frame(&allocation, key).unwrap() {
        PhysicalBoundedFrameAccess::Hit(lease) => lease,
        _ => panic!("resolved bounded frame must hit without source authority"),
    };
    assert_eq!(&*hit, &[9; 32]);
    assert_eq!(pool.counters().source_loads(), 1);
    assert_eq!(pool.counters().hits(), 1);
    assert_eq!(
        pool.counters().metadata_bytes(),
        admitted_metadata,
        "bounded fault, resolution, and hit cannot grow admitted metadata"
    );
}

#[test]
fn bounded_failure_shares_one_terminal_and_releases_reserved_limit() {
    let identity = store(23);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let key = bounded_key(identity, 1, 64);
    let owner = expect_bounded_fault(&pool, &allocation, key);
    let waiter = match pool.access_bounded_frame(&allocation, key).unwrap() {
        PhysicalBoundedFrameAccess::Coalesced(waiter) => waiter,
        _ => panic!("failed bounded overlap must attach one waiter"),
    };
    let owner_terminal = match owner.load(|_| Err::<u32, _>("length denied"), |_| Ok::<_, &str>(()))
    {
        Err(PhysicalFrameFaultError::Source { terminal, .. }) => terminal,
        _ => panic!("bounded source denial must retain its terminal"),
    };
    assert_eq!(waiter.wait().unwrap_err(), owner_terminal);
    let counters = pool.counters();
    assert_eq!(counters.resident_bytes(), 0);
    assert_eq!(counters.frame_entries(), 0);
    assert_eq!(counters.pin_leases(), 0);
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: bounded completion collision wakes an already waiting participant; source: worth-store-buffer-pool::physical_residency::tests::frame_access::bounded::bounded_completion_collision_wakes_an_already_waiting_participant; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn bounded_completion_collision_wakes_an_already_waiting_participant() {
    let identity = store(27);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(256, 3, 1, 64, 4)).unwrap());
    let allocation = allocation(&pool, READ_SCOPE);
    let artifact_key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    drop(
        expect_fault(&pool, &allocation, artifact_key)
            .load(|target| fill(target, 3))
            .unwrap(),
    );

    let bounded = bounded_key(identity, 1, 64);
    let owner = expect_bounded_fault(&pool, &allocation, bounded);
    let waiter = match pool.access_bounded_frame(&allocation, bounded).unwrap() {
        PhysicalBoundedFrameAccess::Coalesced(waiter) => waiter,
        _ => panic!("bounded collision participant must coalesce before completion"),
    };
    let (terminal_tx, terminal_rx) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        terminal_tx.send(waiter.wait()).unwrap();
    });
    for _ in 0..100_000 {
        if pool.inner.bounded_join_waiters() == 1 {
            break;
        }
        std::thread::yield_now();
    }
    assert_eq!(
        pool.inner.bounded_join_waiters(),
        1,
        "participant must be sleeping before the owner reaches terminal publication"
    );

    let owner_terminal = match owner.load(|_| Ok::<_, ()>(32), |target| fill(target, 4)) {
        Err(PhysicalFrameFaultError::Residency { terminal, denial }) => {
            assert_eq!(denial, PhysicalResidencyDenial::FrameAlreadyResident);
            terminal
        }
        _ => panic!("completion collision must reject the bounded owner"),
    };
    let waiter_terminal = terminal_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("terminal publication must wake the already-sleeping participant")
        .unwrap_err();
    worker.join().unwrap();
    assert_eq!(waiter_terminal, owner_terminal);
    assert_eq!(
        owner_terminal.kind(),
        PhysicalFrameLoadTerminalKind::SourceExecutionFailed
    );
    assert_eq!(pool.counters().frame_entries(), 1);
    assert_eq!(pool.counters().resident_bytes(), 32);
}

#[test]
fn bounded_alias_is_removed_with_eviction_and_refaults() {
    let identity = store(24);
    let pool = PhysicalResidencyPool::open(identity, limits(32, 1, 1, 64, 1)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let bounded = bounded_key(identity, 1, 32);
    let first = expect_bounded_fault(&pool, &allocation, bounded)
        .load(
            |_| Ok::<_, ()>(32),
            |target| {
                target.fill(4);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    drop(first);

    let incoming = PhysicalFrameKey::new(identity, coordinate(2, 32));
    drop(
        expect_fault(&pool, &allocation, incoming)
            .load(|target| fill(target, 8))
            .unwrap(),
    );
    assert_eq!(pool.counters().evictions(), 1);
    assert!(matches!(
        pool.access_bounded_frame(&allocation, bounded).unwrap(),
        PhysicalBoundedFrameAccess::Fault(_)
    ));
}

#[test]
fn one_artifact_reuses_its_resident_frame_across_compatible_request_limits() {
    let identity = store(25);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let wide = bounded_key(identity, 1, 64);
    drop(
        expect_bounded_fault(&pool, &allocation, wide)
            .load(
                |_| Ok::<_, ()>(32),
                |target| {
                    target.fill(5);
                    Ok::<_, ()>(())
                },
            )
            .unwrap(),
    );

    let exact_bound = bounded_key(identity, 1, 32);
    assert!(matches!(
        pool.access_bounded_frame(&allocation, exact_bound).unwrap(),
        PhysicalBoundedFrameAccess::Hit(_)
    ));
    assert_eq!(
        pool.access_bounded_frame(&allocation, bounded_key(identity, 1, 16))
            .unwrap_err(),
        PhysicalResidencyDenial::FrameLengthMismatch
    );
    assert_eq!(pool.counters().source_loads(), 1);
}

#[test]
fn narrower_request_conflicts_with_wider_loading_then_uses_resolved_length() {
    let identity = store(26);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let wide = bounded_key(identity, 1, 64);
    let narrow = bounded_key(identity, 1, 32);
    let owner = expect_bounded_fault(&pool, &allocation, wide);

    assert_eq!(
        pool.access_bounded_frame(&allocation, narrow).unwrap_err(),
        PhysicalResidencyDenial::BoundedLoadLimitConflict {
            active_limit: 64,
            requested_limit: 32,
        }
    );
    assert_eq!(pool.counters().source_loads(), 0);
    drop(
        owner
            .load(
                |_| Ok::<_, ()>(16),
                |target| {
                    target.fill(4);
                    Ok::<_, ()>(())
                },
            )
            .unwrap(),
    );
    assert!(matches!(
        pool.access_bounded_frame(&allocation, narrow).unwrap(),
        PhysicalBoundedFrameAccess::Hit(_)
    ));
    assert_eq!(pool.counters().source_loads(), 1);
}

#[test]
fn wider_request_cannot_inherit_a_narrow_owner_limit_and_retries_validly() {
    let identity = store(27);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let narrow = bounded_key(identity, 1, 16);
    let wide = bounded_key(identity, 1, 64);
    let owner = expect_bounded_fault(&pool, &allocation, narrow);

    assert_eq!(
        pool.access_bounded_frame(&allocation, wide).unwrap_err(),
        PhysicalResidencyDenial::BoundedLoadLimitConflict {
            active_limit: 16,
            requested_limit: 64,
        }
    );
    assert!(matches!(
        owner.load(|_| Ok::<_, ()>(32), |_| Ok::<_, ()>(())),
        Err(PhysicalFrameFaultError::Residency {
            denial: PhysicalResidencyDenial::FrameLengthMismatch,
            ..
        })
    ));
    let recovered = expect_bounded_fault(&pool, &allocation, wide)
        .load(
            |_| Ok::<_, ()>(32),
            |target| {
                target.fill(8);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    assert_eq!(&*recovered, &[8; 32]);
    assert_eq!(pool.counters().source_loads(), 2);
}

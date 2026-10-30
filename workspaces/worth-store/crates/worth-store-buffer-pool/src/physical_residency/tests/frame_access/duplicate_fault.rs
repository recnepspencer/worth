use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc, Barrier,
};
use std::thread::JoinHandle;

use super::*;

const DUPLICATE_SOURCE_LOAD_PREDICATE: &str = "duplicate-source-load";

#[derive(Clone)]
struct SourceExecutionCount {
    discoveries: Arc<AtomicU64>,
    fills: Arc<AtomicU64>,
}

impl SourceExecutionCount {
    fn new() -> Self {
        Self {
            discoveries: Arc::new(AtomicU64::new(0)),
            fills: Arc::new(AtomicU64::new(0)),
        }
    }

    fn discoveries(&self) -> u64 {
        self.discoveries.load(Ordering::Acquire)
    }

    fn fills(&self) -> u64 {
        self.fills.load(Ordering::Acquire)
    }
}

struct RunningFaultOwner {
    loading_identity: PhysicalFrameLoadingIdentity,
    release: Arc<Barrier>,
    worker: JoinHandle<PhysicalFrameLease>,
}

impl RunningFaultOwner {
    #[expect(
        clippy::disallowed_methods,
        reason = "category: coordinated storage-owner unit-test harness; purpose: RunningFaultOwner start; source: worth-store-buffer-pool::physical_residency::tests::frame_access::duplicate_fault::RunningFaultOwner::start; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
    )]
    fn start(
        pool: Arc<PhysicalResidencyPool>,
        key: PhysicalBoundedFrameKey,
        source: SourceExecutionCount,
    ) -> Self {
        let ready = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let (identity_tx, identity_rx) = mpsc::sync_channel(1);
        let worker_ready = Arc::clone(&ready);
        let worker_release = Arc::clone(&release);
        let worker = std::thread::spawn(move || {
            let allocation = allocation(&pool, READ_SCOPE);
            let owner = match pool.access_bounded_frame(&allocation, key).unwrap() {
                PhysicalBoundedFrameAccess::Fault(owner) => owner,
                _ => panic!("first bounded access must own the source fault"),
            };
            identity_tx.send(owner.loading_identity()).unwrap();
            worker_ready.wait();
            worker_release.wait();
            owner
                .load(
                    |_| {
                        source.discoveries.fetch_add(1, Ordering::AcqRel);
                        Ok::<_, ()>(32)
                    },
                    |target| {
                        source.fills.fetch_add(1, Ordering::AcqRel);
                        target.fill(7);
                        Ok::<_, ()>(())
                    },
                )
                .unwrap()
        });

        ready.wait();
        Self {
            loading_identity: identity_rx.recv().unwrap(),
            release,
            worker,
        }
    }

    fn complete(self) -> PhysicalFrameLease {
        self.release.wait();
        self.worker.join().unwrap()
    }
}

fn require_shared_loading_identity(
    access: &PhysicalBoundedFrameAccess,
    expected: PhysicalFrameLoadingIdentity,
) {
    match access {
        PhysicalBoundedFrameAccess::Fault(owner) => {
            assert_eq!(owner.loading_identity(), expected);
        }
        PhysicalBoundedFrameAccess::Coalesced(waiter) => {
            assert_eq!(waiter.loading_identity(), expected);
        }
        PhysicalBoundedFrameAccess::Hit(_) => {
            panic!("overlapping bounded access cannot observe a completed hit")
        }
    }
}

fn resolve_competing_access(
    access: PhysicalBoundedFrameAccess,
    source: &SourceExecutionCount,
) -> Option<PhysicalFrameLease> {
    match access {
        PhysicalBoundedFrameAccess::Coalesced(waiter) => Some(waiter.wait().unwrap()),
        PhysicalBoundedFrameAccess::Fault(owner) => {
            let _ = owner.load(
                |_| {
                    source.discoveries.fetch_add(1, Ordering::AcqRel);
                    Ok::<_, ()>(32)
                },
                |target| {
                    source.fills.fetch_add(1, Ordering::AcqRel);
                    target.fill(7);
                    Ok::<_, ()>(())
                },
            );
            None
        }
        PhysicalBoundedFrameAccess::Hit(_) => unreachable!("classified before owner release"),
    }
}

#[test]
fn duplicate_bounded_fault_cannot_mint_second_source_owner() {
    let identity = store(22);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(256, 2, 1, 64, 4)).unwrap());
    let key = PhysicalBoundedFrameKey::new(
        identity,
        RecordArtifactFile::RootRoutingBlock {
            generation: 1,
            block: 1,
        },
        NonZeroU32::new(64).unwrap(),
    );
    let source = SourceExecutionCount::new();
    let running_owner = RunningFaultOwner::start(Arc::clone(&pool), key, source.clone());
    let allocation = allocation(&pool, READ_SCOPE);
    let competing_access = pool.access_bounded_frame(&allocation, key).unwrap();
    require_shared_loading_identity(&competing_access, running_owner.loading_identity);
    let owner_lease = running_owner.complete();
    let competing_lease = resolve_competing_access(competing_access, &source);
    let counters = pool.counters();
    if source.discoveries() != 1 || source.fills() != 1 || counters.source_loads() != 1 {
        panic!("MUTANT_PREDICATE:{DUPLICATE_SOURCE_LOAD_PREDICATE}");
    }

    let competing_lease =
        competing_lease.expect("a correct duplicate fault returns wait-only authority");
    assert_eq!(&*owner_lease, &*competing_lease);
    assert_eq!(counters.faults(), 1);
    assert_eq!(counters.coalesced_waiters(), 1);
    assert_eq!(counters.pinned_frames(), 1);
    assert_eq!(counters.pin_leases(), 2);
    drop((owner_lease, competing_lease));
    let released = pool.counters();
    assert_eq!(released.pinned_frames(), 0);
    assert_eq!(released.pin_leases(), 0);
    assert_eq!(released.active_loading_frames(), 0);
}

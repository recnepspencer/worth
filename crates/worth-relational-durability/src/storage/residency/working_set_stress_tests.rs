//! S1/M1/M2: eight threads across two branches evict and rewarm concurrently.
use super::working_set_fixture::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use worth_relational_storage::internal_port::storage::residency::retention::RelationalRegionRetention;
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

const SLOTS: usize = 6;
const ROUNDS: usize = 48;

/// One branch head region and the basis its selected root names.
struct BranchSlot {
    region: RelationalRegionRetention,
    basis: usize,
}

struct Court {
    bases: [Basis; 2],
    accountant: Arc<WorkingSetAccountant>,
    branches: [Vec<RwLock<Arc<BranchSlot>>>; 2],
    entity: worth_relational_contracts::internal_port::identity::data::record_ids::EntityId,
    reads: AtomicUsize,
    retries: AtomicUsize,
}

impl Court {
    fn slot(&self, basis: usize) -> Arc<BranchSlot> {
        Arc::new(BranchSlot {
            region: RelationalRegionRetention::new(
                bound_cell(
                    &self.bases[basis],
                    &self.accountant,
                    ResidencyPoolKind::Head,
                ),
                None,
            ),
            basis,
        })
    }

    /// Read through a possibly superseded root until it serves: it must serve
    /// its own basis. Saturation while other threads hold borrows is retried.
    fn read(
        &self,
        slot: &BranchSlot,
        full: bool,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        for _ in 0..100_000 {
            let served = if full {
                slot.region.borrow_full(execution, LIMIT).map(|lease| {
                    assert_basis(&lease, &self.bases[slot.basis]);
                })
            } else {
                slot.region.borrow_head(execution, LIMIT).map(|lease| {
                    assert!(lease.head_view().entity(self.entity).is_some());
                })
            };
            match served {
                Ok(()) => {
                    self.reads.fetch_add(1, Ordering::AcqRel);
                    return;
                }
                Err(failure) if memory_exhausted(&failure) => {
                    self.retries.fetch_add(1, Ordering::AcqRel);
                    std::thread::yield_now();
                }
                Err(failure) => panic!("a bound read failed: {failure:?}"),
            }
        }
        panic!("a read never regained memory after borrows were released");
    }

    fn run(&self, thread: usize) {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let branch = &self.branches[thread % 2];
        let mut seed = 0x9e37_79b9_u64.wrapping_mul(thread as u64 + 1);
        for round in 0..ROUNDS {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let index = (seed >> 33) as usize % SLOTS;
            if thread < 2 && round % 4 == 0 {
                // Branch head update: the slot now names the other basis. The
                // superseded root may still be read by other threads.
                let previous = branch[index].read().unwrap().basis;
                let next = self.slot(1 - previous);
                *branch[index].write().unwrap() = next.clone();
                self.read(&next, true, execution);
                continue;
            }
            let slot = branch[index].read().unwrap().clone();
            self.read(&slot, round % 3 != 0, execution);
        }
    }
}

#[test]
fn eight_threads_across_two_branches_rewarm_only_their_exact_basis_without_deadlock() {
    let Bases {
        original,
        edited,
        entity,
    } = bases();
    let (head, full) = image_bytes(&original);
    // Three resident Full borrows fit; twelve branch regions never do.
    let ceiling = 3 * (head + full);
    let accountant = WorkingSetAccountant::new(budget(ceiling, ceiling));
    let mut court = Court {
        bases: [original, edited],
        accountant,
        branches: [Vec::new(), Vec::new()],
        entity,
        reads: AtomicUsize::new(0),
        retries: AtomicUsize::new(0),
    };
    for basis in 0..2 {
        court.branches[basis] = (0..SLOTS).map(|_| RwLock::new(court.slot(basis))).collect();
    }
    let court = Arc::new(court);
    let (done, finished) = std::sync::mpsc::channel();
    let workers = (0..8)
        .map(|thread| {
            let court = court.clone();
            let done = done.clone();
            std::thread::spawn(move || {
                court.run(thread);
                done.send(()).unwrap();
            })
        })
        .collect::<Vec<_>>();
    for _ in 0..workers.len() {
        finished
            .recv_timeout(Duration::from_secs(300))
            .expect("every worker finishes: no AB/BA deadlock across eviction");
    }
    for worker in workers {
        worker.join().expect("no worker observed a wrong basis");
    }
    let pressure = court.accountant.pressure();
    assert_eq!(court.reads.load(Ordering::Acquire), 8 * ROUNDS);
    assert!(
        pressure.evictions_completed > 0,
        "the working set was exceeded"
    );
    assert!(pressure.head_resident_bytes <= ceiling);
}

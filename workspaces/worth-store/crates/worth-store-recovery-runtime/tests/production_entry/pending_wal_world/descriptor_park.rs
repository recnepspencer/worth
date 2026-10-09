//! Parks a reclaim at the durable WAL of its descriptor, the third WAL
//! durability point of the batch, and only then writes the marker.

use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{production::PhysicalMutationCheckpoint, BlobReclaimRequest};

pub(super) fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
    marker_bytes: &[u8],
) {
    let first = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
    let cancelled = AtomicBool::new(false);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(180);
            while !first.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "manifest WAL durability");
            }
            let second =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            while !second.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "reservation WAL durability");
            }
            let third =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            while !third.await_arrival() {
                if cancelled.load(Ordering::SeqCst) {
                    return;
                }
                assert!(Instant::now() < deadline, "descriptor WAL durability");
            }
            let pending = marker.with_extension("pending");
            fs::write(&pending, marker_bytes).unwrap();
            fs::rename(pending, marker).unwrap();
        });
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, Ordering::SeqCst);
        panic!("released descriptor ended before durable WAL pause: {result:?}");
    });
}

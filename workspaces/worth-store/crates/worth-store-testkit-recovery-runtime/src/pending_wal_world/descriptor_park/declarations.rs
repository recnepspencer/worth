//! Parks a reclaim at the durable WAL of its descriptor, the third WAL
//! durability point of the batch, and only then writes the marker.

pub use {std::fs, std::path::Path, std::sync::atomic::AtomicBool, std::sync::atomic::Ordering, std::thread, std::time::Duration, std::time::Instant};

pub use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobReclaimRequest};

pub fn park_at_descriptor_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: BlobReclaimRequest,
    marker: &Path,
    marker_bytes: &[u8],
) {
    let first = serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            while !first.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "manifest WAL durability");
            }
            let second =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            first.release();
            while !second.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "reservation WAL durability");
            }
            let third =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            second.release();
            while !third.await_arrival() {
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                assert!(std::time::Instant::now() < deadline, "descriptor WAL durability");
            }
            let pending = marker.with_extension("pending");
            std::fs::write(&pending, marker_bytes).unwrap();
            std::fs::rename(pending, marker).unwrap();
        });
        let result = serving
            .blobs()
            .unwrap()
            .reclaim(request)
            .and_then(|handle| handle.wait());
        cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        panic!("released descriptor ended before durable WAL pause: {result:?}");
    });
}
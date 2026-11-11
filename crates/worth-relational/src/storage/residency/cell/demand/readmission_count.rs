//! A completed region load acquires one lease; any refusal acquires none.
use super::*;

pub(super) fn acquire(
    loaded: Result<Arc<ResidentRegion>, Failure>,
    leases: &AtomicUsize,
    partition_id: crate::identity::data::PartitionId,
) -> Result<Arc<ResidentRegion>, Failure> {
    let region = loaded?;
    let mut count = leases.load(Ordering::Acquire);
    loop {
        let Some(updated) = count.checked_add(1) else {
            return Err(Failure::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition_id),
                cause: Cause::MemoryExhausted,
            }));
        };
        match leases.compare_exchange_weak(count, updated, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => return Ok(region),
            Err(reloaded) => count = reloaded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_readmission_keeps_the_original_lease_count() {
        for original in [0, 3, usize::MAX] {
            let leases = AtomicUsize::new(original);
            let denied = acquire(
                Err(Failure::Request(
                    crate::execution::RelationalExecutionDenialCause::Cancelled,
                )),
                &leases,
                crate::identity::data::PartitionId(7),
            );
            assert!(matches!(
                denied,
                Err(Failure::Request(
                    crate::execution::RelationalExecutionDenialCause::Cancelled
                ))
            ));
            assert_eq!(leases.load(Ordering::SeqCst), original);
        }
    }
}

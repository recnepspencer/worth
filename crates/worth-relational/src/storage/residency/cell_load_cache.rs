//! One raw readmission serves both adjacency authority and serving admission.
//! Retryable request stops never become a stored negative result.
use super::{RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::storage::data::RelationalStorageUnavailable;
use std::sync::{Arc, Mutex};

pub(super) type RegionLoadCache =
    Mutex<Option<Result<Arc<ResidentRegion>, RelationalStorageUnavailable>>>;

pub(super) fn load(
    cache: &RegionLoadCache,
    partition_id: crate::identity::data::PartitionId,
    readmit: impl FnOnce() -> Result<Arc<ResidentRegion>, Failure>,
) -> Result<Arc<ResidentRegion>, Failure> {
    let mut cached = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(result) = &*cached {
        return result.clone().map_err(Into::into);
    }
    let result = readmit();
    match &result {
        Ok(region) => *cached = Some(Ok(region.clone())),
        Err(Failure::StorageUnavailable(failure))
            if failure.cause.is_proven_damage()
                && failure.subject
                    == crate::storage::data::RelationalStorageSubject::Region(partition_id) =>
        {
            *cached = Some(Err(*failure))
        }
        Err(Failure::StorageUnavailable(_)) => {}
        Err(Failure::Request(_)) => {}
    }
    result
}

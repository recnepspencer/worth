use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::{
    RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationDenial,
    RecoveryObserverObservationFailure,
};

pub(super) struct BoundedArtifactRead {
    pub(super) contents: Vec<u8>,
    pub(super) byte_length: u64,
    pub(super) digest: [u8; 32],
}

pub(super) fn read_bounded_artifact(
    path: &Path,
    limits: RecoveryObserverLimits,
    counters: &mut RecoveryObserverCounters,
) -> Result<BoundedArtifactRead, RecoveryObserverObservationFailure> {
    let declared = path
        .metadata()
        .map_err(|error| media_failure(error, *counters, path))?
        .len();
    let projected = counters
        .bytes_read()
        .checked_add(declared)
        .ok_or_else(|| byte_limit_failure(u64::MAX, limits, *counters, path))?;
    if projected > limits.maximum_bytes() {
        return Err(byte_limit_failure(projected, limits, *counters, path));
    }
    let declared_capacity =
        usize::try_from(declared).map_err(|_| allocation_failure(*counters, path))?;
    let mut contents = Vec::new();
    contents
        .try_reserve_exact(declared_capacity)
        .map_err(|_| allocation_failure(*counters, path))?;
    let mut file =
        std::fs::File::open(path).map_err(|error| media_failure(error, *counters, path))?;
    counters
        .record_file_opened()
        .ok_or_else(|| byte_limit_failure(u64::MAX, limits, *counters, path))?;
    let mut digest = Sha256::new();
    let mut observed = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| media_failure(error, *counters, path))?;
        if count == 0 {
            break;
        }
        let count = count as u64;
        let total = counters
            .bytes_read()
            .checked_add(count)
            .ok_or_else(|| byte_limit_failure(u64::MAX, limits, *counters, path))?;
        if total > limits.maximum_bytes() {
            return Err(byte_limit_failure(total, limits, *counters, path));
        }
        counters
            .record_bytes_read(count)
            .ok_or_else(|| byte_limit_failure(u64::MAX, limits, *counters, path))?;
        observed = observed
            .checked_add(count)
            .ok_or_else(|| byte_limit_failure(u64::MAX, limits, *counters, path))?;
        digest.update(&buffer[..count as usize]);
        contents.extend_from_slice(&buffer[..count as usize]);
    }
    if observed != declared {
        return Err(RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::ArtifactChanged,
            *counters,
            path,
        ));
    }
    Ok(BoundedArtifactRead {
        contents,
        byte_length: observed,
        digest: digest.finalize().into(),
    })
}

fn byte_limit_failure(
    observed: u64,
    limits: RecoveryObserverLimits,
    counters: RecoveryObserverCounters,
    path: &Path,
) -> RecoveryObserverObservationFailure {
    RecoveryObserverObservationFailure::at_path(
        RecoveryObserverObservationDenial::ByteLimit {
            observed,
            admitted: limits.maximum_bytes(),
        },
        counters,
        path,
    )
}

fn media_failure(
    error: std::io::Error,
    counters: RecoveryObserverCounters,
    path: &Path,
) -> RecoveryObserverObservationFailure {
    RecoveryObserverObservationFailure::at_path(
        RecoveryObserverObservationDenial::Media(error.kind()),
        counters,
        path,
    )
}

fn allocation_failure(
    counters: RecoveryObserverCounters,
    path: &Path,
) -> RecoveryObserverObservationFailure {
    RecoveryObserverObservationFailure::at_path(
        RecoveryObserverObservationDenial::EvidenceBufferAllocationFailed,
        counters,
        path,
    )
}

//! Recovered certificate digests charge the exact Format wire buffer while
//! selected roster and proof backing remain live in the Store rejoin ledger.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    ReleaseCheckpointAccumulatorV2, ReleaseCheckpointNoReleaseV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
};

use super::RecoveredReleaseLedgerDenial;
use crate::physical_runtime::recovery_residency::StoreRejoinResidentLedger;

pub(in super::super) fn accumulator_digest_with_resident(
    accumulator: ReleaseCheckpointAccumulatorV2,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<[u8; 32], RecoveredReleaseLedgerDenial> {
    digest_encoded(
        RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
        resident,
        |reserved| accumulator.encode_in_reserved(reserved),
    )
}

pub(in super::super) fn marker_digest_with_resident(
    marker: ReleaseCheckpointNoReleaseV1,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<[u8; 32], RecoveredReleaseLedgerDenial> {
    digest_encoded(
        RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
        resident,
        |reserved| marker.encode_in_reserved(reserved),
    )
}

fn digest_encoded(
    exact_wire_bytes: usize,
    resident: &mut StoreRejoinResidentLedger,
    encode: impl FnOnce(Vec<u8>) -> Option<Vec<u8>>,
) -> Result<[u8; 32], RecoveredReleaseLedgerDenial> {
    let reserved = resident.reserve_vec::<u8>(exact_wire_bytes)?;
    let reserved_capacity = reserved.capacity();
    let charged = resident.vector_bytes(&reserved)?;
    let Some(encoded) = encode(reserved) else {
        resident.release(charged);
        return Err(RecoveredReleaseLedgerDenial::SelectedFactMismatch);
    };
    if encoded.len() != exact_wire_bytes {
        drop(encoded);
        resident.release(charged);
        return Err(RecoveredReleaseLedgerDenial::SelectedFactMismatch);
    }
    debug_assert_eq!(encoded.capacity(), reserved_capacity);
    let digest = Sha256::digest(&encoded).into();
    drop(encoded);
    resident.release(charged);
    Ok(digest)
}

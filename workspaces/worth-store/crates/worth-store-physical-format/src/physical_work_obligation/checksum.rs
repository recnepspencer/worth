use sha2::{Digest, Sha256};

use super::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES;

const COVERED_END: usize = 128;

pub(super) fn calculate(record: &[u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES]) -> [u8; 32] {
    Sha256::digest(&record[..COVERED_END]).into()
}

/// Fingerprints exactly one complete immutable v6 byte incarnation.
pub fn physical_work_obligation_v6_incarnation_digest(
    record: &[u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES],
) -> [u8; 32] {
    Sha256::digest(record).into()
}

/// Digests the fixed-width, domain-separated physical-work scope preimage.
///
/// Runtime integrity owns construction of the exact validation scope. This
/// format mechanism only fixes the SHA-256 execution and complete preimage
/// width; callers cannot select a subrange.
pub fn physical_work_obligation_v6_scope_digest(exact_scope_preimage: &[u8; 65]) -> [u8; 32] {
    Sha256::digest(exact_scope_preimage).into()
}

use super::classify_physical_handoff::{
    classify_and_reject_physical_handoff, PhysicalCorruptionHandoffClassification,
};
use crate::{BlobCorruptionDenial, BlobDamageCase};

/// Production observation entry: physical pre-decode denial does not mint blob authority.
pub fn observe_physical_pre_decode_denial(
    observed_damage: BlobDamageCase,
) -> (
    PhysicalCorruptionHandoffClassification,
    BlobCorruptionDenial,
) {
    classify_and_reject_physical_handoff(observed_damage)
}

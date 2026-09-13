use crate::corruption::{
    observe_physical_pre_decode_denial, PhysicalCorruptionHandoffClassification,
};
use crate::{BlobCorruptionDenial, BlobDamageCase};

/// Cross-crate production handoff: physical-integrity pre-decode denial → blob corruption rejection.
pub fn reject_physical_handoff_from_pre_decode_denial(
    observed_damage: BlobDamageCase,
) -> (
    PhysicalCorruptionHandoffClassification,
    BlobCorruptionDenial,
) {
    observe_physical_pre_decode_denial(observed_damage)
}

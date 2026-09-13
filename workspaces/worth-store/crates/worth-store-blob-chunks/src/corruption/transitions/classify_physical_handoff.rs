use crate::{BlobCorruptionDenial, BlobDamageCase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCorruptionHandoffClassification {
    damage_case: BlobDamageCase,
}

impl PhysicalCorruptionHandoffClassification {
    pub const fn classify_from_damage_observation(
        damage_case: BlobDamageCase,
    ) -> PhysicalCorruptionHandoffClassification {
        PhysicalCorruptionHandoffClassification { damage_case }
    }

    pub const fn damage_case(self) -> BlobDamageCase {
        self.damage_case
    }
}

pub fn classify_and_reject_physical_handoff(
    observed_damage: BlobDamageCase,
) -> (
    PhysicalCorruptionHandoffClassification,
    BlobCorruptionDenial,
) {
    let classification =
        PhysicalCorruptionHandoffClassification::classify_from_damage_observation(observed_damage);
    let rejection = reject_physical_handoff_as_blob_authority(&classification);
    (classification, rejection)
}

pub const fn reject_physical_handoff_as_blob_authority(
    classification: &PhysicalCorruptionHandoffClassification,
) -> BlobCorruptionDenial {
    BlobCorruptionDenial::LowerPhysicalEvidenceRejected {
        damage_case: classification.damage_case(),
    }
}

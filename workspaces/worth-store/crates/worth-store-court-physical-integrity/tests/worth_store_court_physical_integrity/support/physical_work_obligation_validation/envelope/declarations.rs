pub(crate) use worth_store_physical_format::integrity_declarations::families::{
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};
pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::physical_work_obligation::{
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
};
pub(crate) use worth_store_physical_integrity::{
    validate_physical_work_obligation, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, PhysicalIntegrityVersionAxis,
    PhysicalWorkObligationIntegrityValidation, UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::physical_work_obligation_validation::{
    assert_damage, field_range, identity, independent_sha256, rejection, reseal, scope, store,
    OBLIGATION_OFFSET,
};
pub(crate) use crate::vectors::{
    operation_3, operation_4, OPERATION_3_SCOPE_SHA, OPERATION_3_SHA, OPERATION_4_SHA, STORE_BYTES,
};

pub(crate) fn validation_record(
    bytes: &[u8; 160],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
) -> PhysicalIntegrityValidationRecord {
    let (validation, _) = validate_physical_work_obligation(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let worth_store_physical_integrity::PhysicalWorkObligationIntegrityValidation::Intact(
        validated,
    ) = validation
    else {
        panic!("scope-digest fixture was rejected")
    };
    validated.into_validation_record()
}

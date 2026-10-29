pub(crate) use {worth_store_physical_format::integrity_declarations::families::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, worth_store_physical_format::integrity_declarations::families::PHYSICAL_WORK_OBLIGATION_V6_VERSION};
pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode};
pub(crate) use {worth_store_physical_integrity::validate_physical_work_obligation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityValidationRecord, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::PhysicalWorkObligationIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::physical_work_obligation_validation::assert_damage, crate::support::physical_work_obligation_validation::field_range, crate::support::physical_work_obligation_validation::identity, crate::support::physical_work_obligation_validation::independent_sha256, crate::support::physical_work_obligation_validation::rejection, crate::support::physical_work_obligation_validation::reseal, crate::support::physical_work_obligation_validation::scope, crate::support::physical_work_obligation_validation::store, crate::support::physical_work_obligation_validation::OBLIGATION_OFFSET};
pub(crate) use {crate::vectors::operation_3, crate::vectors::operation_4, crate::vectors::OPERATION_3_SCOPE_SHA, crate::vectors::OPERATION_3_SHA, crate::vectors::OPERATION_4_SHA, crate::vectors::STORE_BYTES};

pub(crate) fn validation_record(
    bytes: &[u8; 160],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
) -> PhysicalIntegrityValidationRecord {
    let (validation, _) = validate_physical_work_obligation(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let worth_store_physical_integrity::PhysicalWorkObligationIntegrityValidation::Intact(validated) = validation else {
        panic!("scope-digest fixture was rejected")
    };
    validated.into_validation_record()
}
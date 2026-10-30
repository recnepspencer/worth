pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_integrity::{
    validate_wal_frame, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    UntrustedPhysicalArtifact, WalFrameIntegrityValidation, WalPayloadProjectionDenial,
};

pub(crate) use crate::sha256::independent_sha256;
pub(crate) use crate::wal_frame_validation::support::{
    assert_damage, assert_independent_frame_checksums, assert_rejected_counters, literal,
    rejection, scope, store, CLEAN_HEX, FRAME_OFFSET,
};

pub(crate) fn intact_record(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> worth_store_physical_integrity::PhysicalIntegrityValidationRecord {
    let (validation, _) = validate_wal_frame(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) = validation
    else {
        panic!("frozen frame must validate under a source-bound descriptive scope");
    };
    validated.into_validation_record()
}

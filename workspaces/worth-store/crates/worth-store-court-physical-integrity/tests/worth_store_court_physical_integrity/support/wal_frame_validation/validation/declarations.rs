pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_integrity::validate_wal_frame, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::UntrustedPhysicalArtifact, worth_store_physical_integrity::WalFrameIntegrityValidation, worth_store_physical_integrity::WalPayloadProjectionDenial};

pub(crate) use {crate::sha256::independent_sha256};
pub(crate) use {crate::wal_frame_validation::support::assert_damage, crate::wal_frame_validation::support::assert_independent_frame_checksums, crate::wal_frame_validation::support::assert_rejected_counters, crate::wal_frame_validation::support::literal, crate::wal_frame_validation::support::rejection, crate::wal_frame_validation::support::scope, crate::wal_frame_validation::support::store, crate::wal_frame_validation::support::CLEAN_HEX, crate::wal_frame_validation::support::FRAME_OFFSET};

pub(crate) fn intact_record(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> worth_store_physical_integrity::PhysicalIntegrityValidationRecord {
    let (validation, _) =
        validate_wal_frame(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("frozen frame must validate under a source-bound descriptive scope");
    };
    validated.into_validation_record()
}
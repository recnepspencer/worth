pub(crate) use {worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::checksum_oracles::crc32c};
pub(crate) use {crate::literal_vectors::checkpoint_validation::HEADER, crate::literal_vectors::checkpoint_validation::SECURED_HEADER};
pub(crate) use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_header};

pub(crate) fn assert_header_damage(
    bytes: &[u8],
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: PhysicalFormatField,
) {
    let scope = header_scope_staged();
    let (validation, _) = validate_checkpoint_stream_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged security binding unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        cause,
        range,
        Some(field),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
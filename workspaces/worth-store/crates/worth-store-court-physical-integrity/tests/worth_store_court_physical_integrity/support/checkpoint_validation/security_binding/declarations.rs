pub(crate) use worth_store_physical_integrity::{
    validate_checkpoint_stream_header, CheckpointStreamHeaderIntegrityValidation,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::checkpoint_validation::support::{
    assert_damage, field_range, header_scope_staged, reseal_record, validate_header,
};
pub(crate) use crate::checksum_oracles::crc32c;
pub(crate) use crate::literal_vectors::checkpoint_validation::{HEADER, SECURED_HEADER};

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
    let worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(
        rejection,
    ) = validation
    else {
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

use worth_store_physical_integrity::{
    validate_checkpoint_stream_header, CheckpointStreamHeaderIntegrityValidation,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

use super::checksum_oracles::crc32c;
use super::literal_vectors::{HEADER, SECURED_HEADER};
use super::support::{
    assert_damage, field_range, header_scope_staged, reseal_record, validate_header,
};

#[test]
fn secured_header_literal_binds_policy_retention_and_digest() {
    assert_eq!(crc32c(&SECURED_HEADER[..160]), 0xa967_3d6b);
    let validated = validate_header(&SECURED_HEADER, header_scope_staged());
    let security = validated.source().security_binding().unwrap();
    assert_eq!(security.policy_identity(), [9; 32]);
    assert_eq!(security.idempotency_retention_generations(), 8);
    let literal_digest: [u8; 32] = SECURED_HEADER[128..160].try_into().unwrap();
    assert_eq!(security.digest(), literal_digest);
}

#[test]
fn absent_binding_residue_localizes_the_actual_nonzero_byte() {
    let mut residue = HEADER;
    residue[128] = 1;
    reseal_record(&mut residue);
    assert_header_damage(
        &residue,
        PhysicalDamageCause::MalformedStructure,
        field_range(header_scope_staged(), 128, 1),
        PhysicalFormatField::Reserved,
    );
}

#[test]
fn secured_binding_relation_localizes_its_complete_ambiguous_evidence() {
    for offset in [88, 128] {
        let mut substitution = SECURED_HEADER;
        substitution[offset] ^= 1;
        reseal_record(&mut substitution);
        assert_header_damage(
            &substitution,
            PhysicalDamageCause::ChecksumMismatch,
            field_range(header_scope_staged(), 16, 144),
            PhysicalFormatField::Checksum,
        );
    }
}

fn assert_header_damage(
    bytes: &[u8],
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: PhysicalFormatField,
) {
    let scope = header_scope_staged();
    let (validation, _) = validate_checkpoint_stream_header(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    );
    let CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) = validation else {
        panic!("damaged security binding unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        cause,
        range,
        Some(field),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

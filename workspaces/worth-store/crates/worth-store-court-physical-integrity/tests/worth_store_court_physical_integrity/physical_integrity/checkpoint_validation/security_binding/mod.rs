#[allow(unused_imports)] use crate::checkpoint_validation::security_binding::*;
use {worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::checksum_oracles::crc32c};
use {crate::literal_vectors::checkpoint_validation::HEADER, crate::literal_vectors::checkpoint_validation::SECURED_HEADER};
use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_header};

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
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(header_scope_staged(), 128, 1),
        worth_store_physical_integrity::PhysicalFormatField::Reserved,
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
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            field_range(header_scope_staged(), 16, 144),
            worth_store_physical_integrity::PhysicalFormatField::Checksum,
        );
    }
}

#[allow(unused_imports)]
use crate::checkpoint_validation::security_binding::*;

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

use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_staged, identity, reseal_record, validate_binding, validate_compaction,
    validate_dirty, validate_header,
};

#[test]
fn footer_rejects_checksum_valid_record_kind_substitution() {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let scope = footer_scope(identity);
    let mut substituted = FOOTER;
    substituted[9] = 4;
    reseal_record(&mut substituted);
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, counters) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&substituted),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("record-kind-substituted footer unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 9, 1),
        Some(PhysicalFormatField::CheckpointRecordKind),
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_eq!(
        counters.family(),
        PhysicalIntegrityArtifactFamily::CheckpointFooter
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), FOOTER.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(
            PhysicalDamageCause::RecordKindMismatch
        )),
        1
    );
}

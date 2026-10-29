use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_header};

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
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, counters) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&substituted),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("record-kind-substituted footer unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 9, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointRecordKind),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_eq!(
        counters.family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter
    );
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), FOOTER.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(
            worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch
        )),
        1
    );
}

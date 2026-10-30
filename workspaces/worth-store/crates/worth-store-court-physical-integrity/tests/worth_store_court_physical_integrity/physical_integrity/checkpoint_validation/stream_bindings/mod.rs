#[allow(unused_imports)]
use crate::checkpoint_validation::stream_bindings::*;

#[test]
fn clean_literal_stream_seals_all_five_views_and_validation_records() {
    let identity = identity();
    let header_scope = header_scope_staged();
    let dirty_scope = dirty_scope(identity);
    let compaction_scope = compaction_scope(identity);
    let binding_scope = binding_scope(identity, BINDING.len() as u64);
    let footer_scope = footer_scope(identity);
    let header_input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&HEADER);
    let dirty_input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&DIRTY_BASIS);
    let binding_input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&BINDING);

    let header = validate_header(&HEADER, header_scope);
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope);
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope);
    let binding = validate_binding(&BINDING, binding_scope);
    let footer = validate_footer(
        &FOOTER,
        footer_scope,
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &compaction,
            std::slice::from_ref(&binding),
        ),
    );

    assert_eq!(header.checkpoint_identity(), identity);
    assert_eq!(header.source().wal().admitted_begin_lsn(), 10);
    assert_eq!(header.source().wal().covered_end_lsn_exclusive(), 20);
    assert_eq!(header.source().root().generation(), 3);
    assert_eq!(header.source().root().tree_identity(), 4);
    assert_eq!(header.source().dirty_generation_frontier(), 5);
    assert!(header.source().security_binding().is_none());
    assert_eq!(
        dirty.basis().coordinate().artifact(),
        worth_store_physical_format::RecordArtifactFile::BootstrapCatalog
    );
    assert_eq!(dirty.basis().coordinate().offset(), 64);
    assert_eq!(dirty.basis().coordinate().length(), 16);
    assert_eq!(dirty.basis().dirty_generation(), 6);
    assert_eq!(compaction.generation(), 9);
    assert_eq!(compaction.wal_cutoff_lsn_exclusive(), 18);
    assert_eq!(binding.payload_bytes(), 3);
    assert_eq!(binding.encoded_bytes(), 23);
    assert_eq!(footer.footer().dirty_record_count(), 1);
    assert_eq!(footer.footer().binding_record_count(), 1);
    assert_eq!(footer.footer().binding_record_bytes(), 23);
    assert!(header.matches_input(header_input));
    assert!(dirty.matches_input(dirty_input));
    assert!(binding.matches_input(binding_input));
    assert!(header.into_validation_record().matches_scope(header_scope));
    assert!(dirty.into_validation_record().matches_scope(dirty_scope));
    assert!(compaction
        .into_validation_record()
        .matches_scope(compaction_scope));
    assert!(binding
        .into_validation_record()
        .matches_scope(binding_scope));
    assert!(footer.into_validation_record().matches_scope(footer_scope));
}

#[test]
fn footer_rejects_checksum_valid_selective_aggregate_substitution() {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let mut substituted_dirty = DIRTY_BASIS;
    substituted_dirty[56] ^= 1;
    reseal_record(&mut substituted_dirty);
    let dirty = validate_dirty(&substituted_dirty, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let scope = footer_scope(identity);
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("substituted dirty aggregate unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 48, 32),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn footer_rejects_binding_aggregate_cardinality_and_ordering_lies() {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let mut substituted_binding = BINDING;
    substituted_binding[16] ^= 1;
    reseal_record(&mut substituted_binding);
    let binding = validate_binding(
        &substituted_binding,
        binding_scope(identity, substituted_binding.len() as u64),
    );
    let scope = footer_scope(identity);
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("substituted binding aggregate unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 120, 32),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut count_lie = FOOTER;
    count_lie[40..48].copy_from_slice(&0_u64.to_le_bytes());
    reseal_record(&mut count_lie);
    let clean_binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&clean_binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&count_lie),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("footer cardinality lie unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 40, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let misplaced_scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
        identity,
        worth_store_physical_integrity::PhysicalByteRange::new(BINDING_OFFSET + 1, 23).unwrap(),
    );
    let misplaced = validate_binding(&BINDING, misplaced_scope);
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&misplaced),
    );
    let (validation, _) = validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) =
        validation
    else {
        panic!("misordered binding unexpectedly validated");
    };
    assert_damage(
        rejection,
        misplaced_scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        misplaced_scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_eq!(DIRTY_OFFSET, 164);
}

#[test]
fn footer_rejects_every_later_record_identity_substitution() {
    for target in [
        crate::checkpoint_validation::stream_bindings::LaterRecord::Dirty,
        crate::checkpoint_validation::stream_bindings::LaterRecord::Compaction,
        crate::checkpoint_validation::stream_bindings::LaterRecord::Binding,
    ] {
        assert_later_identity_substitution(target);
    }
}

#[test]
fn footer_localizes_each_compaction_and_binding_summary_field() {
    let cases = [
        (
            80,
            8,
            worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate,
        ),
        (
            88,
            8,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            96,
            8,
            worth_store_physical_integrity::PhysicalFormatField::WalLsnRange,
        ),
        (
            104,
            8,
            worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate,
        ),
        (
            112,
            8,
            worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate,
        ),
    ];
    for (offset, length, field) in cases {
        let identity = identity();
        let header = validate_header(&HEADER, header_scope_staged());
        let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
        let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
        let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
        let scope = footer_scope(identity);
        let mut footer = FOOTER;
        footer[offset] ^= 1;
        reseal_record(&mut footer);
        let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &compaction,
            std::slice::from_ref(&binding),
        );
        let (validation, _) = validate_checkpoint_footer(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&footer),
            scope,
            basis,
        );
        let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(
            rejection,
        ) = validation
        else {
            panic!("footer summary-field lie at {offset} unexpectedly validated");
        };
        assert_damage(
            rejection,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
            field_range(scope, offset as u64, length as u64),
            Some(field),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

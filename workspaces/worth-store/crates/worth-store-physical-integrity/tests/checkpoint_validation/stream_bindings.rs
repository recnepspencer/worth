use worth_store_physical_format::{
    CheckpointSelectiveRecordAggregate, PhysicalCheckpointIdentity, RecordArtifactFile,
};
use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

use super::checksum_oracles::{crc32c, sha256};
use super::literal_vectors::{
    BINDING, BINDING_AGGREGATE, BINDING_COMPACTION, DIRTY_AGGREGATE, DIRTY_BASIS, FOOTER, HEADER,
};
use super::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_staged, identity, reseal_record, validate_binding, validate_compaction,
    validate_dirty, validate_footer, validate_header, BINDING_OFFSET, DIRTY_OFFSET,
};
use std::num::NonZeroU64;

#[test]
fn literal_records_have_independent_crc_and_sha_vectors() {
    for record in [
        HEADER.as_slice(),
        DIRTY_BASIS.as_slice(),
        BINDING_COMPACTION.as_slice(),
        BINDING.as_slice(),
        FOOTER.as_slice(),
    ] {
        let checksum_offset = record.len() - 4;
        assert_eq!(
            u32::from_le_bytes(record[checksum_offset..].try_into().unwrap()),
            crc32c(&record[..checksum_offset])
        );
    }
    assert_eq!(sha256(&DIRTY_BASIS), DIRTY_AGGREGATE);
    assert_eq!(sha256(&BINDING), BINDING_AGGREGATE);
    assert_eq!(&FOOTER[48..80], DIRTY_AGGREGATE);
    assert_eq!(&FOOTER[120..152], BINDING_AGGREGATE);

    let mut aggregate = CheckpointSelectiveRecordAggregate::new();
    aggregate.include(&DIRTY_BASIS).unwrap();
    assert_eq!(aggregate.summary().digest(), DIRTY_AGGREGATE);
    assert_eq!(aggregate.summary().record_count(), 1);
    assert_eq!(aggregate.summary().encoded_bytes(), 68);
}

#[test]
fn clean_literal_stream_seals_all_five_views_and_validation_records() {
    let identity = identity();
    let header_scope = header_scope_staged();
    let dirty_scope = dirty_scope(identity);
    let compaction_scope = compaction_scope(identity);
    let binding_scope = binding_scope(identity, BINDING.len() as u64);
    let footer_scope = footer_scope(identity);
    let header_input = UntrustedPhysicalArtifact::from_bounded_bytes(&HEADER);
    let dirty_input = UntrustedPhysicalArtifact::from_bounded_bytes(&DIRTY_BASIS);
    let binding_input = UntrustedPhysicalArtifact::from_bounded_bytes(&BINDING);

    let header = validate_header(&HEADER, header_scope);
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope);
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope);
    let binding = validate_binding(&BINDING, binding_scope);
    let footer = validate_footer(
        &FOOTER,
        footer_scope,
        CheckpointFooterValidationBasis::new(
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
        RecordArtifactFile::BootstrapCatalog
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
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("substituted dirty aggregate unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 48, 32),
        Some(PhysicalFormatField::CheckpointAggregate),
        PhysicalBlastRadius::CompleteArtifact,
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
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("substituted binding aggregate unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 120, 32),
        Some(PhysicalFormatField::CheckpointAggregate),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut count_lie = FOOTER;
    count_lie[40..48].copy_from_slice(&0_u64.to_le_bytes());
    reseal_record(&mut count_lie);
    let clean_binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&clean_binding),
    );
    let (validation, _) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&count_lie),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("footer cardinality lie unexpectedly validated");
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 40, 8),
        Some(PhysicalFormatField::CheckpointAggregate),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let misplaced_scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
        identity,
        worth_store_physical_integrity::PhysicalByteRange::new(BINDING_OFFSET + 1, 23).unwrap(),
    );
    let misplaced = validate_binding(&BINDING, misplaced_scope);
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&misplaced),
    );
    let (validation, _) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    );
    let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
        panic!("misordered binding unexpectedly validated");
    };
    assert_damage(
        rejection,
        misplaced_scope,
        PhysicalDamageCause::SequenceMismatch,
        misplaced_scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_eq!(DIRTY_OFFSET, 164);
}

#[test]
fn footer_rejects_every_later_record_identity_substitution() {
    for target in [
        LaterRecord::Dirty,
        LaterRecord::Compaction,
        LaterRecord::Binding,
    ] {
        assert_later_identity_substitution(target);
    }
}

#[derive(Clone, Copy)]
enum LaterRecord {
    Dirty,
    Compaction,
    Binding,
}

fn assert_later_identity_substitution(target: LaterRecord) {
    let identity = identity();
    let substituted = PhysicalCheckpointIdentity::new(
        identity.store_identity(),
        NonZeroU64::new(identity.sequence().get() + 1).unwrap(),
    );
    let header = validate_header(&HEADER, header_scope_staged());
    let scope = footer_scope(identity);
    let (rejection, substituted_scope) = match target {
        LaterRecord::Dirty => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(substituted));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = dirty.scope();
            let basis = CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        LaterRecord::Compaction => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction =
                validate_compaction(&BINDING_COMPACTION, compaction_scope(substituted));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = compaction.scope();
            let basis = CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        LaterRecord::Binding => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding =
                validate_binding(&BINDING, binding_scope(substituted, BINDING.len() as u64));
            let substituted_scope = binding.scope();
            let basis = CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
    };
    assert_damage(
        rejection,
        substituted_scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        substituted_scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn footer_basis_rejection<'records, 'media>(
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'records, 'media>,
) -> worth_store_physical_integrity::PhysicalIntegrityRejection {
    match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    )
    .0
    {
        CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        CheckpointFooterIntegrityValidation::Intact(_) => {
            panic!("substituted later-record identity unexpectedly validated")
        }
    }
}

#[test]
fn footer_localizes_each_compaction_and_binding_summary_field() {
    let cases = [
        (80, 8, PhysicalFormatField::CheckpointAggregate),
        (88, 8, PhysicalFormatField::PhysicalGeneration),
        (96, 8, PhysicalFormatField::WalLsnRange),
        (104, 8, PhysicalFormatField::CheckpointAggregate),
        (112, 8, PhysicalFormatField::CheckpointAggregate),
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
        let basis = CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &compaction,
            std::slice::from_ref(&binding),
        );
        let (validation, _) = validate_checkpoint_footer(
            UntrustedPhysicalArtifact::from_bounded_bytes(&footer),
            scope,
            basis,
        );
        let CheckpointFooterIntegrityValidation::Rejected(rejection) = validation else {
            panic!("footer summary-field lie at {offset} unexpectedly validated");
        };
        assert_damage(
            rejection,
            scope,
            PhysicalDamageCause::AggregateMismatch,
            field_range(scope, offset as u64, length as u64),
            Some(field),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

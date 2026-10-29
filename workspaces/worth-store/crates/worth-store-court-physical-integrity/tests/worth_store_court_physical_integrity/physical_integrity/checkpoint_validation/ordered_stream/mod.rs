#[allow(unused_imports)] use crate::checkpoint_validation::ordered_stream::*;
use {worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::checksum_oracles::sha256};
use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header};

#[test]
fn two_record_aggregates_are_literal_order_sensitive_in_both_sections() {
    let mut dirty_b = DIRTY_BASIS;
    dirty_b[56] = 7;
    reseal_record(&mut dirty_b);
    let mut binding_b = BINDING;
    binding_b[16] = 0xdd;
    reseal_record(&mut binding_b);
    let dirty_ab = concatenate(&DIRTY_BASIS, &dirty_b);
    let dirty_ba = concatenate(&dirty_b, &DIRTY_BASIS);
    let binding_ab = concatenate(&BINDING, &binding_b);
    let binding_ba = concatenate(&binding_b, &BINDING);
    assert_ne!(sha256(&dirty_ab), sha256(&dirty_ba));
    assert_ne!(sha256(&binding_ab), sha256(&binding_ba));

    let footer_bytes = two_record_footer(sha256(&dirty_ab), sha256(&binding_ab));
    validate_correct_two_record_stream(&dirty_b, &binding_b, &footer_bytes);
    assert_swapped_dirty_rejected(&dirty_b, &binding_b, &footer_bytes);
    assert_swapped_binding_rejected(&dirty_b, &binding_b, &footer_bytes);
}

#[test]
fn every_section_boundary_requires_a_contiguous_physical_scope() {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty_a = validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_A_OFFSET));
    let misplaced_dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_B_OFFSET + 1));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(COMPACTION_OFFSET));
    let binding = validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET));
    let scope = footer_scope();
    let dirty_records = [dirty_a, misplaced_dirty];
    let rejection = reject_footer(
        &FOOTER,
        scope,
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            &dirty_records,
            &compaction,
            std::slice::from_ref(&binding),
        ),
    );
    let offending = dirty_records[1].scope();
    assert_sequence_damage(rejection, offending);

    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_A_OFFSET));
    let misplaced_compaction =
        validate_compaction(&BINDING_COMPACTION, compaction_scope(DIRTY_B_OFFSET + 1));
    let binding = validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET));
    let rejection = reject_footer(
        &FOOTER,
        scope,
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &misplaced_compaction,
            std::slice::from_ref(&binding),
        ),
    );
    assert_sequence_damage(rejection, misplaced_compaction.scope());
    assert_eq!(identity, header.checkpoint_identity());
}

#[test]
fn second_binding_and_final_footer_boundaries_are_independently_checked() {
    let mut dirty_b = DIRTY_BASIS;
    dirty_b[56] = 7;
    reseal_record(&mut dirty_b);
    let mut binding_b = BINDING;
    binding_b[16] = 0xdd;
    reseal_record(&mut binding_b);
    let footer_bytes = two_record_footer(
        sha256(&concatenate(&DIRTY_BASIS, &dirty_b)),
        sha256(&concatenate(&BINDING, &binding_b)),
    );
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = [
        validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_A_OFFSET)),
        validate_dirty(&dirty_b, dirty_scope(DIRTY_B_OFFSET)),
    ];
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(COMPACTION_OFFSET));
    let bindings = [
        validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET)),
        validate_binding(&binding_b, binding_scope(BINDING_B_OFFSET + 1)),
    ];
    let rejection = reject_footer(
        &footer_bytes,
        footer_scope(),
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_sequence_damage(rejection, bindings[1].scope());

    let bindings = [
        validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET)),
        validate_binding(&binding_b, binding_scope(BINDING_B_OFFSET)),
    ];
    let shifted_footer =
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(identity(), range(FOOTER_OFFSET + 1, 156));
    let rejection = reject_footer(
        &footer_bytes,
        shifted_footer,
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_sequence_damage(rejection, shifted_footer);
}

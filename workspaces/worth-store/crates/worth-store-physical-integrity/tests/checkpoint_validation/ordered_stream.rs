use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::checksum_oracles::sha256;
use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    assert_damage, field_range, header_scope_staged, identity, reseal_record, validate_binding,
    validate_compaction, validate_dirty, validate_footer, validate_header,
};

const DIRTY_A_OFFSET: u64 = 164;
const DIRTY_B_OFFSET: u64 = 232;
const COMPACTION_OFFSET: u64 = 300;
const BINDING_A_OFFSET: u64 = 336;
const BINDING_B_OFFSET: u64 = 359;
const FOOTER_OFFSET: u64 = 382;

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
        CheckpointFooterValidationBasis::new(
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
        CheckpointFooterValidationBasis::new(
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
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_sequence_damage(rejection, bindings[1].scope());

    let bindings = [
        validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET)),
        validate_binding(&binding_b, binding_scope(BINDING_B_OFFSET)),
    ];
    let shifted_footer =
        PhysicalArtifactScope::checkpoint_footer(identity(), range(FOOTER_OFFSET + 1, 156));
    let rejection = reject_footer(
        &footer_bytes,
        shifted_footer,
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_sequence_damage(rejection, shifted_footer);
}

fn validate_correct_two_record_stream(dirty_b: &[u8], binding_b: &[u8], footer_bytes: &[u8]) {
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = [
        validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_A_OFFSET)),
        validate_dirty(dirty_b, dirty_scope(DIRTY_B_OFFSET)),
    ];
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(COMPACTION_OFFSET));
    let bindings = [
        validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET)),
        validate_binding(binding_b, binding_scope(BINDING_B_OFFSET)),
    ];
    let validated = validate_footer(
        footer_bytes,
        footer_scope(),
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_eq!(validated.footer().dirty_record_count(), 2);
    assert_eq!(validated.footer().binding_record_count(), 2);
    assert_eq!(validated.footer().binding_record_bytes(), 46);
}

fn assert_swapped_dirty_rejected(dirty_b: &[u8], binding_b: &[u8], footer_bytes: &[u8]) {
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = [
        validate_dirty(dirty_b, dirty_scope(DIRTY_A_OFFSET)),
        validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_B_OFFSET)),
    ];
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(COMPACTION_OFFSET));
    let bindings = [
        validate_binding(&BINDING, binding_scope(BINDING_A_OFFSET)),
        validate_binding(binding_b, binding_scope(BINDING_B_OFFSET)),
    ];
    let scope = footer_scope();
    let rejection = reject_footer(
        footer_bytes,
        scope,
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 48, 32),
        Some(PhysicalFormatField::CheckpointAggregate),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn assert_swapped_binding_rejected(dirty_b: &[u8], binding_b: &[u8], footer_bytes: &[u8]) {
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = [
        validate_dirty(&DIRTY_BASIS, dirty_scope(DIRTY_A_OFFSET)),
        validate_dirty(dirty_b, dirty_scope(DIRTY_B_OFFSET)),
    ];
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(COMPACTION_OFFSET));
    let bindings = [
        validate_binding(binding_b, binding_scope(BINDING_A_OFFSET)),
        validate_binding(&BINDING, binding_scope(BINDING_B_OFFSET)),
    ];
    let scope = footer_scope();
    let rejection = reject_footer(
        footer_bytes,
        scope,
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    );
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 120, 32),
        Some(PhysicalFormatField::CheckpointAggregate),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn two_record_footer(dirty_digest: [u8; 32], binding_digest: [u8; 32]) -> [u8; 156] {
    let mut footer = FOOTER;
    footer[40..48].copy_from_slice(&2_u64.to_le_bytes());
    footer[48..80].copy_from_slice(&dirty_digest);
    footer[80..88].copy_from_slice(&COMPACTION_OFFSET.to_le_bytes());
    footer[104..112].copy_from_slice(&2_u64.to_le_bytes());
    footer[112..120].copy_from_slice(&46_u64.to_le_bytes());
    footer[120..152].copy_from_slice(&binding_digest);
    reseal_record(&mut footer);
    footer
}

fn concatenate(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(left.len() + right.len());
    bytes.extend_from_slice(left);
    bytes.extend_from_slice(right);
    bytes
}

fn dirty_scope(offset: u64) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_dirty_basis(identity(), range(offset, 68))
}

fn compaction_scope(offset: u64) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_binding_compaction(identity(), range(offset, 36))
}

fn binding_scope(offset: u64) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_binding(identity(), range(offset, 23))
}

fn footer_scope() -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_footer(identity(), range(FOOTER_OFFSET, 156))
}

fn range(offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(offset, length).unwrap()
}

fn reject_footer<'records, 'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'records, 'media>,
) -> PhysicalIntegrityRejection {
    match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        basis,
    )
    .0
    {
        CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        CheckpointFooterIntegrityValidation::Intact(_) => {
            panic!("invalid ordered checkpoint stream unexpectedly validated")
        }
    }
}

fn assert_sequence_damage(rejection: PhysicalIntegrityRejection, scope: PhysicalArtifactScope) {
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}

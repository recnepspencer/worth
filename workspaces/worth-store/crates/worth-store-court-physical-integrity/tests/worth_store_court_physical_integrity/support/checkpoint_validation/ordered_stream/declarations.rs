pub(crate) use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::checkpoint_validation::support::{
    assert_damage, field_range, header_scope_staged, identity, reseal_record, validate_binding,
    validate_compaction, validate_dirty, validate_footer, validate_header,
};
pub(crate) use crate::checksum_oracles::sha256;
pub(crate) use crate::literal_vectors::checkpoint_validation::{
    BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER,
};

pub(crate) const DIRTY_A_OFFSET: u64 = 164;
pub(crate) const DIRTY_B_OFFSET: u64 = 232;
pub(crate) const COMPACTION_OFFSET: u64 = 300;
pub(crate) const BINDING_A_OFFSET: u64 = 336;
pub(crate) const BINDING_B_OFFSET: u64 = 359;
pub(crate) const FOOTER_OFFSET: u64 = 382;

pub(crate) fn validate_correct_two_record_stream(
    dirty_b: &[u8],
    binding_b: &[u8],
    footer_bytes: &[u8],
) {
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
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            &dirty,
            &compaction,
            &bindings,
        ),
    );
    assert_eq!(validated.footer().dirty_record_count(), 2);
    assert_eq!(validated.footer().binding_record_count(), 2);
    assert_eq!(validated.footer().binding_record_bytes(), 46);
}

pub(crate) fn assert_swapped_dirty_rejected(dirty_b: &[u8], binding_b: &[u8], footer_bytes: &[u8]) {
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
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            &dirty,
            &compaction,
            &bindings,
        ),
    );
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 48, 32),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

pub(crate) fn assert_swapped_binding_rejected(
    dirty_b: &[u8],
    binding_b: &[u8],
    footer_bytes: &[u8],
) {
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
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            &dirty,
            &compaction,
            &bindings,
        ),
    );
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::AggregateMismatch,
        field_range(scope, 120, 32),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointAggregate),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

pub(crate) fn two_record_footer(dirty_digest: [u8; 32], binding_digest: [u8; 32]) -> [u8; 156] {
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

pub(crate) fn concatenate(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(left.len() + right.len());
    bytes.extend_from_slice(left);
    bytes.extend_from_slice(right);
    bytes
}

pub(crate) fn dirty_scope(offset: u64) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(
        identity(),
        range(offset, 68),
    )
}

pub(crate) fn compaction_scope(offset: u64) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(
        identity(),
        range(offset, 36),
    )
}

pub(crate) fn binding_scope(offset: u64) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
        identity(),
        range(offset, 23),
    )
}

pub(crate) fn footer_scope() -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        identity(),
        range(FOOTER_OFFSET, 156),
    )
}

pub(crate) fn range(offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap()
}

pub(crate) fn reject_footer<'records, 'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'records, 'media>,
) -> PhysicalIntegrityRejection {
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(_) => {
            panic!("invalid ordered checkpoint stream unexpectedly validated")
        }
    }
}

pub(crate) fn assert_sequence_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
) {
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

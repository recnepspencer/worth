use std::num::NonZeroU64;

use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalIntegrityValidationRecord,
};

use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    binding_scope, compaction_scope, dirty_scope, footer_scope, header_scope_known,
    header_scope_staged, identity, other_store, validate_binding, validate_compaction,
    validate_dirty, validate_footer, validate_header,
};

#[test]
fn validation_records_bind_every_kind_to_store_sequence_family_and_range() {
    let identity = identity();
    let foreign_sequence = PhysicalCheckpointIdentity::new(
        identity.store_identity(),
        NonZeroU64::new(identity.sequence().get() + 1).unwrap(),
    );
    let foreign_store = PhysicalCheckpointIdentity::new(other_store(), identity.sequence());
    let header_scope = header_scope_staged();
    let dirty_own = dirty_scope(identity);
    let compaction_own = compaction_scope(identity);
    let binding_own = binding_scope(identity, BINDING.len() as u64);
    let footer_own = footer_scope(identity);
    let header = validate_header(&HEADER, header_scope);
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_own);
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_own);
    let binding = validate_binding(&BINDING, binding_own);
    let footer = validate_footer(
        &FOOTER,
        footer_own,
        CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &compaction,
            std::slice::from_ref(&binding),
        ),
    );
    let records = [
        (header.into_validation_record(), header_scope),
        (dirty.into_validation_record(), dirty_own),
        (compaction.into_validation_record(), compaction_own),
        (binding.into_validation_record(), binding_own),
        (footer.into_validation_record(), footer_own),
    ];
    let foreign_identities = [
        header_scope_known(foreign_sequence),
        dirty_scope(foreign_sequence),
        compaction_scope(foreign_sequence),
        binding_scope(foreign_sequence, BINDING.len() as u64),
        footer_scope(foreign_sequence),
    ];
    let foreign_stores = [
        header_scope_known(foreign_store),
        dirty_scope(foreign_store),
        compaction_scope(foreign_store),
        binding_scope(foreign_store, BINDING.len() as u64),
        footer_scope(foreign_store),
    ];
    let foreign_families = [
        dirty_scope(identity),
        header_scope_known(identity),
        header_scope_known(identity),
        header_scope_known(identity),
        header_scope_known(identity),
    ];
    for (index, (record, own_scope)) in records.into_iter().enumerate() {
        assert!(record.matches_scope(own_scope));
        assert!(!record.matches_scope(foreign_identities[index]));
        assert!(!record.matches_scope(foreign_stores[index]));
        assert!(!record.matches_scope(shifted(own_scope)));
        assert!(!record.matches_scope(foreign_families[index]));
    }
}

#[test]
fn scope_digest_changes_while_same_inspected_bytes_digest_does_not() {
    let identity = identity();
    let staged = validate_header(&HEADER, header_scope_staged()).into_validation_record();
    let known = validate_header(&HEADER, header_scope_known(identity)).into_validation_record();
    assert_digest_partition(staged, known);

    let dirty = dirty_scope(identity);
    let shifted_dirty = shifted(dirty);
    let original = validate_dirty(&DIRTY_BASIS, dirty).into_validation_record();
    let shifted = validate_dirty(&DIRTY_BASIS, shifted_dirty).into_validation_record();
    assert_digest_partition(original, shifted);
}

fn assert_digest_partition(
    left: PhysicalIntegrityValidationRecord,
    right: PhysicalIntegrityValidationRecord,
) {
    assert_ne!(left.exact_scope_digest(), right.exact_scope_digest());
    assert_eq!(left.byte_range_digest(), right.byte_range_digest());
}

fn shifted(scope: PhysicalArtifactScope) -> PhysicalArtifactScope {
    let range =
        PhysicalByteRange::new(scope.byte_range().offset() + 1, scope.byte_range().length())
            .unwrap();
    let identity = scope.checkpoint_identity().unwrap_or_else(identity);
    match scope.artifact_family() {
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader => {
            PhysicalArtifactScope::checkpoint_stream_header(
                worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::known(identity),
                range,
            )
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis => {
            PhysicalArtifactScope::checkpoint_dirty_basis(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction => {
            PhysicalArtifactScope::checkpoint_binding_compaction(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding => {
            PhysicalArtifactScope::checkpoint_binding(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter => {
            PhysicalArtifactScope::checkpoint_footer(identity, range)
        }
        _ => unreachable!("checkpoint record has a checkpoint family"),
    }
}

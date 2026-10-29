pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_store_physical_format::PhysicalCheckpointIdentity};
pub(crate) use {worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalIntegrityValidationRecord};

pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_known, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header};

pub(crate) fn assert_digest_partition(
    left: PhysicalIntegrityValidationRecord,
    right: PhysicalIntegrityValidationRecord,
) {
    assert_ne!(left.exact_scope_digest(), right.exact_scope_digest());
    assert_eq!(left.byte_range_digest(), right.byte_range_digest());
}

pub(crate) fn shifted(scope: PhysicalArtifactScope) -> PhysicalArtifactScope {
    let range =
        worth_store_physical_integrity::PhysicalByteRange::new(scope.byte_range().offset() + 1, scope.byte_range().length())
            .unwrap();
    let identity = scope.checkpoint_identity().unwrap_or_else(identity);
    match scope.artifact_family() {
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader => {
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
                worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::known(identity),
                range,
            )
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis => {
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction => {
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding => {
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(identity, range)
        }
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter => {
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(identity, range)
        }
        _ => unreachable!("checkpoint record has a checkpoint family"),
    }
}
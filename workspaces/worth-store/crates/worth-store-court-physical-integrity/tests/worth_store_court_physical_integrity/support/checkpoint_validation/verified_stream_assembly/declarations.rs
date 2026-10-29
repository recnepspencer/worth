pub(crate) use {worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::UntrustedPhysicalArtifact, worth_store_physical_integrity::VerifiedCheckpointStream, worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial};

pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header, crate::checkpoint_validation::support::BINDING_OFFSET, crate::checkpoint_validation::support::COMPACTION_OFFSET, crate::checkpoint_validation::support::DIRTY_OFFSET, crate::checkpoint_validation::support::FOOTER_OFFSET};

pub(crate) fn complete_stream(binding: [u8; BINDING.len()]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&HEADER);
    bytes.extend_from_slice(&DIRTY_BASIS);
    bytes.extend_from_slice(&BINDING_COMPACTION);
    bytes.extend_from_slice(&binding);
    bytes.extend_from_slice(&FOOTER);
    bytes
}
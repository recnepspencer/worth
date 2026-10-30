pub(crate) use worth_store_physical_integrity::{
    CheckpointFooterValidationBasis, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial,
};

pub(crate) use crate::checkpoint_validation::support::{
    binding_scope, compaction_scope, dirty_scope, footer_scope, header_scope_staged, identity,
    reseal_record, validate_binding, validate_compaction, validate_dirty, validate_footer,
    validate_header, BINDING_OFFSET, COMPACTION_OFFSET, DIRTY_OFFSET, FOOTER_OFFSET,
};
pub(crate) use crate::literal_vectors::checkpoint_validation::{
    BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER,
};

pub(crate) fn complete_stream(binding: [u8; BINDING.len()]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&HEADER);
    bytes.extend_from_slice(&DIRTY_BASIS);
    bytes.extend_from_slice(&BINDING_COMPACTION);
    bytes.extend_from_slice(&binding);
    bytes.extend_from_slice(&FOOTER);
    bytes
}

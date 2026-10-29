#[allow(unused_imports)] use crate::checkpoint_validation::verified_stream_assembly::*;
use {worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::UntrustedPhysicalArtifact, worth_store_physical_integrity::VerifiedCheckpointStream, worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header, crate::checkpoint_validation::support::BINDING_OFFSET, crate::checkpoint_validation::support::COMPACTION_OFFSET, crate::checkpoint_validation::support::DIRTY_OFFSET, crate::checkpoint_validation::support::FOOTER_OFFSET};

#[test]
fn assembly_rejects_footer_basis_substituted_from_another_allocation() {
    let original = complete_stream(BINDING);
    let mut stale = original.clone();
    stale[BINDING_OFFSET as usize + 16] ^= 0x20;
    reseal_record(&mut stale[BINDING_OFFSET as usize..FOOTER_OFFSET as usize]);

    let checkpoint = identity();
    let original_header =
        validate_header(&original[..DIRTY_OFFSET as usize], header_scope_staged());
    let original_dirty = validate_dirty(
        &original[DIRTY_OFFSET as usize..COMPACTION_OFFSET as usize],
        dirty_scope(checkpoint),
    );
    let original_compaction = validate_compaction(
        &original[COMPACTION_OFFSET as usize..BINDING_OFFSET as usize],
        compaction_scope(checkpoint),
    );
    let original_binding = validate_binding(
        &original[BINDING_OFFSET as usize..FOOTER_OFFSET as usize],
        binding_scope(checkpoint, BINDING.len() as u64),
    );
    let substituted_basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &original_header,
        std::slice::from_ref(&original_dirty),
        &original_compaction,
        std::slice::from_ref(&original_binding),
    );

    let header = validate_header(&stale[..DIRTY_OFFSET as usize], header_scope_staged());
    let dirty = validate_dirty(
        &stale[DIRTY_OFFSET as usize..COMPACTION_OFFSET as usize],
        dirty_scope(checkpoint),
    );
    let compaction = validate_compaction(
        &stale[COMPACTION_OFFSET as usize..BINDING_OFFSET as usize],
        compaction_scope(checkpoint),
    );
    let binding = validate_binding(
        &stale[BINDING_OFFSET as usize..FOOTER_OFFSET as usize],
        binding_scope(checkpoint, BINDING.len() as u64),
    );
    let footer = validate_footer(
        &stale[FOOTER_OFFSET as usize..],
        footer_scope(checkpoint),
        substituted_basis,
    );

    let denial = worth_store_physical_integrity::VerifiedCheckpointStream::assemble_from_validated_records(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&stale),
        &header,
        &[&dirty],
        &compaction,
        &[&binding],
        &footer,
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial::FooterBasisMismatch(_)
    ));
    let denial = worth_store_physical_integrity::VerifiedCheckpointStream::validate_records_with_certificates(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&stale),
        &header,
        &[&dirty],
        &compaction,
        &[&binding],
        &[],
        &footer,
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial::FooterBasisMismatch(_)
    ));
}

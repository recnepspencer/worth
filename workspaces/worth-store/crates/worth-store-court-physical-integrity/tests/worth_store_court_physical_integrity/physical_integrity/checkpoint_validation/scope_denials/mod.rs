#[allow(unused_imports)] use crate::checkpoint_validation::scope_denials::*;
use {worth_store_physical_integrity::validate_checkpoint_binding, worth_store_physical_integrity::validate_checkpoint_binding_compaction, worth_store_physical_integrity::validate_checkpoint_dirty_basis, worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation, worth_store_physical_integrity::CheckpointBindingIntegrityValidation, worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_header};

#[test]
fn every_validator_rejects_a_different_checkpoint_family_scope() {
    let identity = identity();
    let scopes = [
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(identity, range(0, HEADER.len() as u64)),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(
            identity,
            range(164, DIRTY_BASIS.len() as u64),
        ),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
            identity,
            range(232, BINDING_COMPACTION.len() as u64),
        ),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(identity, range(268, BINDING.len() as u64)),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(identity, range(291, FOOTER.len() as u64)),
    ];
    let rejections = [
        header_rejection(&HEADER, scopes[0]),
        dirty_rejection(&DIRTY_BASIS, scopes[1]),
        compaction_rejection(&BINDING_COMPACTION, scopes[2]),
        binding_rejection(&BINDING, scopes[3]),
        footer_wrong_scope_rejection(scopes[4]),
    ];
    for (scope, rejection) in scopes.into_iter().zip(rejections) {
        assert_damage(
            rejection,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn footer_store_substitution_is_distinct_from_sequence_substitution() {
    let identity = identity();
    let mut substituted = FOOTER;
    substituted[16..32].copy_from_slice(&other_store().bytes());
    reseal_record(&mut substituted);
    let scope = footer_scope(identity);
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let rejection = match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&substituted),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("foreign-store footer unexpectedly validated"),
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::StoreIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

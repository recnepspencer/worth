pub(crate) use {worth_store_physical_format::CheckpointSelectiveRecordAggregate, worth_store_physical_format::PhysicalCheckpointIdentity, worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::checksum_oracles::crc32c, crate::checksum_oracles::sha256};
pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_AGGREGATE, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_AGGREGATE, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header, crate::checkpoint_validation::support::BINDING_OFFSET, crate::checkpoint_validation::support::DIRTY_OFFSET};
pub(crate) use {std::num::NonZeroU64};

#[derive(Clone, Copy)]
pub(crate) enum LaterRecord {
    Dirty,
    Compaction,
    Binding,
}

pub(crate) fn assert_later_identity_substitution(target: LaterRecord) {
    let identity = identity();
    let substituted = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        identity.store_identity(),
        std::num::NonZeroU64::new(identity.sequence().get() + 1).unwrap(),
    );
    let header = validate_header(&HEADER, header_scope_staged());
    let scope = footer_scope(identity);
    let (rejection, substituted_scope) = match target {
        crate::checkpoint_validation::stream_bindings::LaterRecord::Dirty => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(substituted));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = dirty.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        crate::checkpoint_validation::stream_bindings::LaterRecord::Compaction => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction =
                validate_compaction(&BINDING_COMPACTION, compaction_scope(substituted));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = compaction.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        crate::checkpoint_validation::stream_bindings::LaterRecord::Binding => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding =
                validate_binding(&BINDING, binding_scope(substituted, BINDING.len() as u64));
            let substituted_scope = binding.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
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
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        substituted_scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

pub(crate) fn footer_basis_rejection<'records, 'media>(
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'records, 'media>,
) -> worth_store_physical_integrity::PhysicalIntegrityRejection {
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(_) => {
            panic!("substituted later-record identity unexpectedly validated")
        }
    }
}
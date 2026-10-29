pub(crate) use {worth_store_physical_integrity::validate_checkpoint_binding, worth_store_physical_integrity::validate_checkpoint_binding_compaction, worth_store_physical_integrity::validate_checkpoint_dirty_basis, worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation, worth_store_physical_integrity::CheckpointBindingIntegrityValidation, worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_header};

pub(crate) fn header_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_stream_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family header scope validated"),
    }
}

pub(crate) fn dirty_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_dirty_basis(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family dirty scope validated"),
    }
}

pub(crate) fn compaction_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_binding_compaction(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family compaction scope validated"),
    }
}

pub(crate) fn binding_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_binding(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope).0
    {
        worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family binding scope validated"),
    }
}

pub(crate) fn footer_wrong_scope_rejection(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    let identity = identity();
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
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family footer scope validated"),
    }
}

pub(crate) fn range(offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap()
}
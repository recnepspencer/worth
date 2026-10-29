pub(crate) use worth_store_physical_integrity::{
    validate_checkpoint_binding, validate_checkpoint_binding_compaction,
    validate_checkpoint_dirty_basis, validate_checkpoint_footer, validate_checkpoint_stream_header,
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

pub(crate) use crate::checkpoint_validation::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_staged, identity, other_store, reseal_record, validate_binding,
    validate_compaction, validate_dirty, validate_header,
};
pub(crate) use crate::literal_vectors::checkpoint_validation::{
    BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER,
};

pub(crate) fn header_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
    match validate_checkpoint_stream_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        _ => panic!("wrong-family header scope validated"),
    }
}

pub(crate) fn dirty_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
    match validate_checkpoint_dirty_basis(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        _ => panic!("wrong-family dirty scope validated"),
    }
}

pub(crate) fn compaction_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
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

pub(crate) fn binding_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
    match validate_checkpoint_binding(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        _ => panic!("wrong-family binding scope validated"),
    }
}

pub(crate) fn footer_wrong_scope_rejection(
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
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
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        _ => panic!("wrong-family footer scope validated"),
    }
}

pub(crate) fn range(offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap()
}

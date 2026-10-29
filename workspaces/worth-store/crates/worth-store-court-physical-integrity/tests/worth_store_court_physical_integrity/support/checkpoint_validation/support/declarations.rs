pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::PhysicalCheckpointIdentity};
pub(crate) use {worth_store_physical_integrity::validate_checkpoint_binding, worth_store_physical_integrity::validate_checkpoint_binding_compaction, worth_store_physical_integrity::validate_checkpoint_dirty_basis, worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation, worth_store_physical_integrity::CheckpointBindingIntegrityValidation, worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity, worth_store_physical_integrity::IntegrityValidatedCheckpointBinding, worth_store_physical_integrity::IntegrityValidatedCheckpointBindingCompaction, worth_store_physical_integrity::IntegrityValidatedCheckpointDirtyBasis, worth_store_physical_integrity::IntegrityValidatedCheckpointFooter, worth_store_physical_integrity::IntegrityValidatedCheckpointStreamHeader, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::checksum_oracles::crc32c};

pub(crate) const HEADER_OFFSET: u64 = 0;
pub(crate) const DIRTY_OFFSET: u64 = 164;
pub(crate) const COMPACTION_OFFSET: u64 = 232;
pub(crate) const BINDING_OFFSET: u64 = 268;
pub(crate) const FOOTER_OFFSET: u64 = 291;

pub(crate) fn store() -> StableStoreIdentity {
    let bytes = core::array::from_fn(|index| (index + 1) as u8);
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(bytes).unwrap(),
    )
    .published_identity()
}

pub(crate) fn other_store() -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([0x44; 16]).unwrap(),
    )
    .published_identity()
}

pub(crate) fn identity() -> PhysicalCheckpointIdentity {
    worth_store_physical_format::PhysicalCheckpointIdentity::new(store(), std::num::NonZeroU64::new(7).unwrap())
}

pub(crate) fn header_scope_staged() -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
        worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::staged(store()),
        range(HEADER_OFFSET, 164),
    )
}

pub(crate) fn header_scope_known(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
        worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::known(identity),
        range(HEADER_OFFSET, 164),
    )
}

pub(crate) fn dirty_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(identity, range(DIRTY_OFFSET, 68))
}

pub(crate) fn compaction_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(identity, range(COMPACTION_OFFSET, 36))
}

pub(crate) fn binding_scope(
    identity: PhysicalCheckpointIdentity,
    encoded_bytes: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(identity, range(BINDING_OFFSET, encoded_bytes))
}

pub(crate) fn footer_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(identity, range(FOOTER_OFFSET, 156))
}

pub(crate) fn validate_header<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointStreamHeader<'media> {
    match validate_checkpoint_stream_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => {
            panic!("checkpoint header rejected: {rejection:?}")
        }
    }
}

pub(crate) fn validate_dirty<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointDirtyBasis<'media> {
    match validate_checkpoint_dirty_basis(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Rejected(rejection) => {
            panic!("dirty basis rejected: {rejection:?}")
        }
    }
}

pub(crate) fn validate_compaction<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointBindingCompaction<'media> {
    match validate_checkpoint_binding_compaction(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) => {
            panic!("binding compaction rejected: {rejection:?}")
        }
    }
}

pub(crate) fn validate_binding<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointBinding<'media> {
    match validate_checkpoint_binding(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope).0
    {
        worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Rejected(rejection) => {
            panic!("binding rejected: {rejection:?}")
        }
    }
}

pub(crate) fn validate_footer<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'_, 'media>,
) -> IntegrityValidatedCheckpointFooter<'media> {
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection) => {
            panic!("footer rejected: {rejection:?}")
        }
    }
}

pub(crate) fn reseal_record(bytes: &mut [u8]) {
    let checksum_offset = bytes.len() - 4;
    let checksum = crc32c(&bytes[..checksum_offset]);
    bytes[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    range(scope.byte_range().offset() + offset, length)
}

pub(crate) fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(worth_store_physical_integrity::PhysicalDamageLocalization::new(
            scope,
            cause,
            damaged_range,
            field,
            blast_radius,
        ))
    );
}

pub(crate) fn range(offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap()
}
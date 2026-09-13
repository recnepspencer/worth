use std::num::NonZeroU64;

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    validate_checkpoint_binding, validate_checkpoint_binding_compaction,
    validate_checkpoint_dirty_basis, validate_checkpoint_footer, validate_checkpoint_stream_header,
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    CheckpointStreamHeaderScopeIdentity, IntegrityValidatedCheckpointBinding,
    IntegrityValidatedCheckpointBindingCompaction, IntegrityValidatedCheckpointDirtyBasis,
    IntegrityValidatedCheckpointFooter, IntegrityValidatedCheckpointStreamHeader,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::checksum_oracles::crc32c;

pub const HEADER_OFFSET: u64 = 0;
pub const DIRTY_OFFSET: u64 = 164;
pub const COMPACTION_OFFSET: u64 = 232;
pub const BINDING_OFFSET: u64 = 268;
pub const FOOTER_OFFSET: u64 = 291;

pub fn store() -> StableStoreIdentity {
    let bytes = core::array::from_fn(|index| (index + 1) as u8);
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes(bytes).unwrap(),
    )
    .published_identity()
}

pub fn other_store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x44; 16]).unwrap(),
    )
    .published_identity()
}

pub fn identity() -> PhysicalCheckpointIdentity {
    PhysicalCheckpointIdentity::new(store(), NonZeroU64::new(7).unwrap())
}

pub fn header_scope_staged() -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(store()),
        range(HEADER_OFFSET, 164),
    )
}

pub fn header_scope_known(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::known(identity),
        range(HEADER_OFFSET, 164),
    )
}

pub fn dirty_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_dirty_basis(identity, range(DIRTY_OFFSET, 68))
}

pub fn compaction_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_binding_compaction(identity, range(COMPACTION_OFFSET, 36))
}

pub fn binding_scope(
    identity: PhysicalCheckpointIdentity,
    encoded_bytes: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_binding(identity, range(BINDING_OFFSET, encoded_bytes))
}

pub fn footer_scope(identity: PhysicalCheckpointIdentity) -> PhysicalArtifactScope {
    PhysicalArtifactScope::checkpoint_footer(identity, range(FOOTER_OFFSET, 156))
}

pub fn validate_header<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointStreamHeader<'media> {
    match validate_checkpoint_stream_header(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointStreamHeaderIntegrityValidation::Intact(validated) => validated,
        CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => {
            panic!("checkpoint header rejected: {rejection:?}")
        }
    }
}

pub fn validate_dirty<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointDirtyBasis<'media> {
    match validate_checkpoint_dirty_basis(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointDirtyBasisIntegrityValidation::Intact(validated) => validated,
        CheckpointDirtyBasisIntegrityValidation::Rejected(rejection) => {
            panic!("dirty basis rejected: {rejection:?}")
        }
    }
}

pub fn validate_compaction<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointBindingCompaction<'media> {
    match validate_checkpoint_binding_compaction(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointBindingCompactionIntegrityValidation::Intact(validated) => validated,
        CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) => {
            panic!("binding compaction rejected: {rejection:?}")
        }
    }
}

pub fn validate_binding<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedCheckpointBinding<'media> {
    match validate_checkpoint_binding(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope).0
    {
        CheckpointBindingIntegrityValidation::Intact(validated) => validated,
        CheckpointBindingIntegrityValidation::Rejected(rejection) => {
            panic!("binding rejected: {rejection:?}")
        }
    }
}

pub fn validate_footer<'media>(
    bytes: &'media [u8],
    scope: PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'_, 'media>,
) -> IntegrityValidatedCheckpointFooter<'media> {
    match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        basis,
    )
    .0
    {
        CheckpointFooterIntegrityValidation::Intact(validated) => validated,
        CheckpointFooterIntegrityValidation::Rejected(rejection) => {
            panic!("footer rejected: {rejection:?}")
        }
    }
}

pub fn reseal_record(bytes: &mut [u8]) {
    let checksum_offset = bytes.len() - 4;
    let checksum = crc32c(&bytes[..checksum_offset]);
    bytes[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

pub fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    range(scope.byte_range().offset() + offset, length)
}

pub fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            cause,
            damaged_range,
            field,
            blast_radius,
        ))
    );
}

fn range(offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(offset, length).unwrap()
}

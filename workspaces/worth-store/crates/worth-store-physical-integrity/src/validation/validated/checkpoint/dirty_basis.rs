use worth_store_physical_format::CheckpointDirtyFrameBasis;

use super::super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedCheckpointDirtyBasis<'media> {
    scope: PhysicalArtifactScope,
    basis: CheckpointDirtyFrameBasis,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedCheckpointDirtyBasis<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        basis: CheckpointDirtyFrameBasis,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_checkpoint_dirty_basis()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.checkpoint_exact_scope_digest()?),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            basis,
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn basis(&self) -> CheckpointDirtyFrameBasis {
        self.basis
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }

    pub(crate) const fn inspected_bytes(&self) -> &'media [u8] {
        self.inspected.bytes()
    }
}

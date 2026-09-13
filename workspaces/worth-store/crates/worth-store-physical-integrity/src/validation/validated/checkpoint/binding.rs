use std::ops::Range;

use worth_store_physical_format::{
    PhysicalCheckpointIdentity, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
};

use super::super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedCheckpointBinding<'media> {
    scope: PhysicalArtifactScope,
    payload_bytes: u32,
    payload_range: Range<usize>,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointBindingPayloadProjectionDenial {
    InputIncarnationMismatch,
    CheckpointIdentityMismatch,
}

#[derive(Debug)]
pub struct IntegrityValidatedCheckpointBindingPayloadProjection<'view, 'media> {
    validated: &'view IntegrityValidatedCheckpointBinding<'media>,
}

impl<'media> IntegrityValidatedCheckpointBinding<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        payload_bytes: u32,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_checkpoint_binding()
            || payload_bytes == 0
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
            payload_bytes,
            payload_range: CHECKPOINT_BINDING_RECORD_PREFIX_BYTES
                ..CHECKPOINT_BINDING_RECORD_PREFIX_BYTES + payload_bytes as usize,
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn payload_bytes(&self) -> u32 {
        self.payload_bytes
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.scope.byte_range().length()
    }

    pub fn project_payload<'view>(
        &'view self,
        input: UntrustedPhysicalArtifact<'media>,
        expected_checkpoint: PhysicalCheckpointIdentity,
    ) -> Result<
        IntegrityValidatedCheckpointBindingPayloadProjection<'view, 'media>,
        CheckpointBindingPayloadProjectionDenial,
    > {
        if !self.inspected.same_incarnation(input) {
            return Err(CheckpointBindingPayloadProjectionDenial::InputIncarnationMismatch);
        }
        if self.scope.checkpoint_identity() != Some(expected_checkpoint) {
            return Err(CheckpointBindingPayloadProjectionDenial::CheckpointIdentityMismatch);
        }
        Ok(IntegrityValidatedCheckpointBindingPayloadProjection { validated: self })
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

impl IntegrityValidatedCheckpointBindingPayloadProjection<'_, '_> {
    pub const fn checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        match self.validated.scope.checkpoint_identity() {
            Some(identity) => identity,
            None => unreachable!(),
        }
    }

    pub fn payload_range(&self) -> Range<usize> {
        self.validated.payload_range.clone()
    }
}

use worth_store_physical_format::{PhysicalCheckpointIdentity, PhysicalCheckpointSource};

use super::super::super::{
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityValidationRecord,
    UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedCheckpointStreamHeader<'media> {
    scope: PhysicalArtifactScope,
    source: PhysicalCheckpointSource,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedCheckpointStreamHeader<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        source: PhysicalCheckpointSource,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let expected = scope.checkpoint_stream_header_identity()?;
        let identity_matches = match expected {
            CheckpointStreamHeaderScopeIdentity::StagedFromChecksummedStream(store) => {
                source.identity().store_identity() == store
            }
            CheckpointStreamHeaderScopeIdentity::Known(identity) => source.identity() == identity,
        };
        if !identity_matches || inspected.byte_count() != scope.byte_range().length() {
            return None;
        }
        Some(Self {
            scope,
            source,
            validation_record: validation_record(scope, validated_range_checksum)?,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn checkpoint_identity(&self) -> PhysicalCheckpointIdentity {
        self.source.identity()
    }

    pub const fn source(&self) -> PhysicalCheckpointSource {
        self.source
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

fn validation_record(
    scope: PhysicalArtifactScope,
    byte_checksum: u32,
) -> Option<PhysicalIntegrityValidationRecord> {
    PhysicalIntegrityValidationRecord::from_validated_scope(
        scope,
        PhysicalIntegrityValidationDigest::crc32c(scope.checkpoint_exact_scope_digest()?),
        PhysicalIntegrityValidationDigest::crc32c(byte_checksum),
        PhysicalIntegrityValidationMechanism::Crc32cV1,
    )
}

use worth_store_physical_format::{blob_record_v1_validation_digest, BlobRecordV1};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedBlobRecord<'media> {
    record: BlobRecordV1<'media>,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedBlobRecord<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        record: BlobRecordV1<'media>,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if scope.blob_record_identity().map(|(_, kind)| kind) != Some(record.kind())
            || scope.byte_range().length() != inspected.byte_count()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::sha256(scope.exact_blob_record_scope_digest()),
            PhysicalIntegrityValidationDigest::sha256(blob_record_v1_validation_digest(
                inspected.bytes(),
            )),
            PhysicalIntegrityValidationMechanism::Sha256V1,
        )?;
        Some(Self {
            record,
            validation_record,
            inspected,
        })
    }

    pub fn record(&self) -> &BlobRecordV1<'media> {
        &self.record
    }

    pub fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}

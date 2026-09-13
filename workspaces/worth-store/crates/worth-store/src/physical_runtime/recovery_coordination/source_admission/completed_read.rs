use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityValidationRecord};

use crate::physical_runtime::RootProtocolAdmissionDenial;

pub(super) struct BoundScheduledRootProtocolSource<'source> {
    read: &'source CompletedScheduledRecoveryReopenRead,
    operation: worth_store_physical_backend::MediaOperationIdentity,
    scope: PhysicalArtifactScope,
}

pub(super) struct ScheduledRootProtocolSource<'source> {
    _read: &'source CompletedScheduledRecoveryReopenRead,
    operation: worth_store_physical_backend::MediaOperationIdentity,
    validation: PhysicalIntegrityValidationRecord,
}

impl<'source> BoundScheduledRootProtocolSource<'source> {
    pub(super) fn bind(
        read: &'source CompletedScheduledRecoveryReopenRead,
        expected_artifact: RecordArtifactFile,
        scope: PhysicalArtifactScope,
    ) -> Result<Self, RootProtocolAdmissionDenial> {
        let physical = read.physical();
        let coordinate = physical.coordinate();
        if read.artifact() != expected_artifact || coordinate.artifact() != expected_artifact {
            return Err(RootProtocolAdmissionDenial::SourceArtifactMismatch);
        }
        if coordinate.offset() != scope.byte_range().offset()
            || u64::from(coordinate.length()) != scope.byte_range().length()
            || physical.completed_bytes() != scope.byte_range().length()
            || read.bytes().len() as u64 != scope.byte_range().length()
        {
            return Err(RootProtocolAdmissionDenial::SourceRangeMismatch);
        }
        Ok(Self {
            read,
            operation: physical.operation(),
            scope,
        })
    }

    pub(super) fn admit(
        self,
        validation: PhysicalIntegrityValidationRecord,
    ) -> Result<ScheduledRootProtocolSource<'source>, RootProtocolAdmissionDenial> {
        if !validation.matches_scope(self.scope) {
            return Err(RootProtocolAdmissionDenial::SourceIncarnationMismatch);
        }
        Ok(ScheduledRootProtocolSource {
            _read: self.read,
            operation: self.operation,
            validation,
        })
    }
}

impl ScheduledRootProtocolSource<'_> {
    pub(super) const fn operation(&self) -> worth_store_physical_backend::MediaOperationIdentity {
        self.operation
    }

    pub(super) const fn validation(&self) -> PhysicalIntegrityValidationRecord {
        self.validation
    }
}

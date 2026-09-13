use worth_store_buffer_pool::{PhysicalFrameLease, PhysicalResidentFrameGeneration};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityValidationRecord};

use super::denial::ResidentIntegrityAdmissionDenial;
use super::source_scope::require_exact_resident_source;
use crate::physical_runtime::lifecycle::{LifecycleState, LifecycleStateSnapshot};

/// Private binding of one descriptive C6 record to its live Store incarnation.
pub(super) struct ResidentIntegrityRecordBinding<'lease> {
    lease: &'lease PhysicalFrameLease,
    lifecycle: std::sync::Arc<LifecycleState>,
    lifecycle_snapshot: LifecycleStateSnapshot,
    frame_generation: PhysicalResidentFrameGeneration,
    scope: PhysicalArtifactScope,
    record: PhysicalIntegrityValidationRecord,
}

impl<'lease> ResidentIntegrityRecordBinding<'lease> {
    pub(super) fn bind_fresh(
        lease: &'lease PhysicalFrameLease,
        lifecycle: std::sync::Arc<LifecycleState>,
        lifecycle_snapshot: LifecycleStateSnapshot,
        scope: PhysicalArtifactScope,
        record: PhysicalIntegrityValidationRecord,
    ) -> Result<Self, ResidentIntegrityAdmissionDenial> {
        require_exact_resident_source(lease, scope)?;
        if !record.matches_scope(scope) {
            return Err(ResidentIntegrityAdmissionDenial::SourceScopeMismatch);
        }
        lease
            .commit_integrity_validation(record)
            .map_err(ResidentIntegrityAdmissionDenial::Frame)?;
        if lifecycle.snapshot() != lifecycle_snapshot {
            lease.invalidate_integrity_validation_if(record);
            return Err(ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged);
        }
        Ok(Self::new(
            lease,
            lifecycle,
            lifecycle_snapshot,
            scope,
            record,
        ))
    }

    pub(super) fn reuse_exact(
        lease: &'lease PhysicalFrameLease,
        lifecycle: std::sync::Arc<LifecycleState>,
        lifecycle_snapshot: LifecycleStateSnapshot,
        scope: PhysicalArtifactScope,
    ) -> Result<Option<Self>, ResidentIntegrityAdmissionDenial> {
        require_exact_resident_source(lease, scope)?;
        let Some(record) = lease.integrity_validation() else {
            return Ok(None);
        };
        if !record.matches_scope(scope) {
            return Err(ResidentIntegrityAdmissionDenial::RetainedRecordChanged);
        }
        Ok(Some(Self::new(
            lease,
            lifecycle,
            lifecycle_snapshot,
            scope,
            record,
        )))
    }

    pub(super) fn enter_owner_decoder(
        &self,
    ) -> Result<&'lease PhysicalFrameLease, ResidentIntegrityAdmissionDenial> {
        self.require_current_binding()?;
        Ok(self.lease)
    }

    pub(super) fn require_current_binding(&self) -> Result<(), ResidentIntegrityAdmissionDenial> {
        if self.lifecycle.snapshot() != self.lifecycle_snapshot {
            return Err(ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged);
        }
        if self.lease.resident_generation() != self.frame_generation {
            return Err(ResidentIntegrityAdmissionDenial::FrameGenerationChanged);
        }
        require_exact_resident_source(self.lease, self.scope)?;
        let record = self
            .lease
            .integrity_validation()
            .ok_or(ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated)?;
        if record != self.record {
            return Err(ResidentIntegrityAdmissionDenial::RetainedRecordChanged);
        }
        Ok(())
    }

    pub(super) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(super) const fn validation_record(&self) -> PhysicalIntegrityValidationRecord {
        self.record
    }

    fn new(
        lease: &'lease PhysicalFrameLease,
        lifecycle: std::sync::Arc<LifecycleState>,
        lifecycle_snapshot: LifecycleStateSnapshot,
        scope: PhysicalArtifactScope,
        record: PhysicalIntegrityValidationRecord,
    ) -> Self {
        Self {
            lease,
            lifecycle,
            lifecycle_snapshot,
            frame_generation: lease.resident_generation(),
            scope,
            record,
        }
    }
}

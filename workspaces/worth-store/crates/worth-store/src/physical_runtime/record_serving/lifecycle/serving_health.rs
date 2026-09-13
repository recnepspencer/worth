use std::sync::atomic::{AtomicU64, Ordering};

use super::super::{
    RecordAppendDenial, RecordReadDenial, RecordScanDenial, RecordStreamFailureKind,
};

#[derive(Debug)]
pub(in crate::physical_runtime) struct ServingHealth {
    generation: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) struct ServingHealthPermit {
    generation: u64,
}

impl ServingHealth {
    pub(in crate::physical_runtime) const fn new(inspection_required: bool) -> Self {
        Self {
            generation: AtomicU64::new(if inspection_required { 1 } else { 0 }),
        }
    }

    pub(in crate::physical_runtime) fn requires_inspection(&self) -> bool {
        self.generation.load(Ordering::Acquire) != 0
    }

    pub(in crate::physical_runtime) fn revoke(&self) {
        let _ = self
            .generation
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire);
    }

    pub(in crate::physical_runtime::record_serving) fn permit(
        &self,
    ) -> Result<ServingHealthPermit, ()> {
        let generation = self.generation.load(Ordering::Acquire);
        if generation != 0 {
            return Err(());
        }
        Ok(ServingHealthPermit { generation })
    }

    pub(in crate::physical_runtime::record_serving) fn require(
        &self,
        permit: ServingHealthPermit,
    ) -> Result<(), ()> {
        (permit.generation == 0 && self.generation.load(Ordering::Acquire) == permit.generation)
            .then_some(())
            .ok_or(())
    }

    pub(in crate::physical_runtime) fn physical_dispatch_guard(
        &self,
    ) -> PhysicalDispatchUnwindGuard<'_> {
        PhysicalDispatchUnwindGuard {
            health: self,
            armed: true,
        }
    }

    pub(in crate::physical_runtime) fn consume_physical_revocation(
        &self,
        _revocation: crate::physical_runtime::PhysicalWorkHealthRevocation,
    ) {
        self.revoke();
    }

    pub(in crate::physical_runtime::record_serving) fn observe_append_denial(
        &self,
        denial: &RecordAppendDenial,
    ) {
        if *denial == RecordAppendDenial::PublishedLayoutDamaged {
            self.revoke();
        }
    }

    pub(in crate::physical_runtime::record_serving) fn observe_read_denial(
        &self,
        denial: RecordReadDenial,
    ) {
        if matches!(
            denial,
            RecordReadDenial::ArtifactUnavailable
                | RecordReadDenial::ArtifactDamaged
                | RecordReadDenial::FormatMismatch
                | RecordReadDenial::StalePlacement(_)
        ) {
            self.revoke();
        }
    }

    pub(in crate::physical_runtime::record_serving) fn observe_stream_failure(
        &self,
        kind: RecordStreamFailureKind,
    ) {
        if matches!(
            kind,
            RecordStreamFailureKind::ArtifactUnavailable
                | RecordStreamFailureKind::ArtifactDamaged
                | RecordStreamFailureKind::FormatMismatch
                | RecordStreamFailureKind::StalePlacement
        ) {
            self.revoke();
        }
    }

    pub(in crate::physical_runtime::record_serving) fn observe_scan_denial(
        &self,
        denial: RecordScanDenial,
    ) {
        match denial {
            RecordScanDenial::ManifestUnavailable => self.revoke(),
            RecordScanDenial::RecordRead(denial) => self.observe_read_denial(denial),
            RecordScanDenial::RecordStream(kind) => self.observe_stream_failure(kind),
            _ => {}
        }
    }
}

pub(in crate::physical_runtime) struct PhysicalDispatchUnwindGuard<'health> {
    health: &'health ServingHealth,
    armed: bool,
}

impl PhysicalDispatchUnwindGuard<'_> {
    pub(in crate::physical_runtime) fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for PhysicalDispatchUnwindGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.health.revoke();
        }
    }
}

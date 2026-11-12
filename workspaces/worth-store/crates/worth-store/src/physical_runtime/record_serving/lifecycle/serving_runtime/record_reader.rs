use worth_store_physical_format::DurablePhysicalRootManifest;

use super::ServingPhysicalRuntime;
use crate::physical_runtime::{
    stability::PhysicalRootReadLease, PhysicalReadProtectionDenial, PhysicalRecordReader,
};

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn owns_reader(&self, reader: &PhysicalRecordReader) -> bool {
        reader.store == self.store_identity()
            && reader.generation == self.parts.core.lifecycle_generation()
            && std::sync::Weak::ptr_eq(
                &reader.runtime,
                &std::sync::Arc::downgrade(&self.parts.work_runtime),
            )
    }

    pub(in crate::physical_runtime) fn recovery_retained_record_reader(
        &self,
    ) -> Result<Option<PhysicalRecordReader>, PhysicalReadProtectionDenial> {
        Ok(self
            .parts
            .publication
            .capture_recovery_retained_root()?
            .map(|(root, lease)| self.reader_from_captured_root(root, lease)))
    }

    /// Captures every C.10 retained root with a shared live registration.
    /// The registry extends each lease atomically with its manifest snapshot.
    pub(in crate::physical_runtime) fn retained_record_readers(
        &self,
        maximum_roots: usize,
    ) -> Result<Vec<PhysicalRecordReader>, PhysicalReadProtectionDenial> {
        let retained = self
            .parts
            .read_protection
            .registry()
            .snapshot_retained_roots(maximum_roots)?
            .into_iter()
            .map(|(root, lease)| self.reader_from_captured_root(root, lease))
            .collect();
        Ok(retained)
    }

    /// Atomically captures and protects the current root for this acquisition.
    pub fn records(&self) -> Result<PhysicalRecordReader, PhysicalReadProtectionDenial> {
        let (current_root, protection) = self.parts.publication.capture_read_root()?;
        Ok(self.reader_from_captured_root(current_root, protection))
    }

    /// Admit one actual protected reader without waiting for any capture lock.
    /// A refusal installs nothing. Liveness is checked before installation;
    /// accepted custody is returned even if the request stops immediately after.
    /// The caller must retain that reader before its next fallible consultation.
    pub fn try_records_for_checkpoint(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PhysicalRecordReader, crate::physical_runtime::PhysicalCheckpointReadProtectionDenial>
    {
        let (root, protection) = self.parts.publication.try_capture_read_root(request)?;
        Ok(self.reader_from_captured_root(root, protection))
    }

    /// Consumes an already captured root without selecting a newer root.
    pub(super) fn reader_from_captured_root(
        &self,
        current_root: DurablePhysicalRootManifest,
        protection: PhysicalRootReadLease,
    ) -> PhysicalRecordReader {
        let read = super::super::super::CanonicalRecordReadPort::new(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
            self.parts.work_admission,
            self.parts.scheduler_admission.clone(),
            self.parts.record_work.clone(),
        );
        let mutation = super::super::super::CanonicalRecordMutationPort::new(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
            self.parts.work_admission,
            self.parts.scheduler_admission.clone(),
            self.parts.record_work.clone(),
        );
        let frame_ports = self.parts.residency.ports().clone();
        let writeback = mutation.frame_writeback_port(frame_ports.clone());
        PhysicalRecordReader {
            execution: crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
                &self.parts.work_runtime,
                self.parts.core.lifecycle_generation(),
            ),
            store: self.store_identity(),
            format: self.parts.format,
            access: self.parts.access,
            current_root,
            protection,
            generation: self.parts.core.lifecycle_generation(),
            runtime: std::sync::Arc::downgrade(&self.parts.work_runtime),
            lifecycle: self.parts.record_owner.reader(),
            residency: super::super::super::residency::PhysicalResidencyWorkPort::new(
                frame_ports,
                super::super::super::residency::frame_loading::CanonicalFrameReadSource::new(read),
                writeback,
                self.parts.core.lifecycle_state(),
            ),
        }
    }
}

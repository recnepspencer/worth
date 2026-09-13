use worth_store_physical_format::DurableExtentRecordPlacement;

use super::{PhysicalRecordReader, ReadPlacement, RecordReadIdentity, RecordReadSession};
use crate::physical_runtime::record_serving::{
    access::extent_read_session::ExtentReadState,
    residency::record_frame_reader::RecordFrameReader, PhysicalRecordId, RecordReadDenial,
    RecordReadObservation,
};

mod manifest_admission;

impl PhysicalRecordReader {
    pub(super) fn open_extent(
        &self,
        record: PhysicalRecordId,
        placement: DurableExtentRecordPlacement,
        observation: &mut RecordReadObservation,
        allocation: worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<RecordReadSession, RecordReadDenial> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(RecordReadDenial::ServingRequiresInspection)?;
        let health_permit = runtime
            .health
            .permit()
            .map_err(|_| RecordReadDenial::ServingRequiresInspection)?;
        let artifacts = RecordFrameReader::serving(self.residency.clone());
        let admitted = manifest_admission::admit_extent_manifest(
            manifest_admission::ExtentManifestAdmission {
                reader: self,
                record,
                placement,
                observation,
                allocation: &allocation,
                artifacts: &artifacts,
            },
        )?;
        observation.touched_extents = 1;
        Ok(RecordReadSession {
            placement: ReadPlacement::Extent(Box::new(ExtentReadState::new(
                artifacts,
                admitted.artifact,
                admitted.manifest,
                admitted.artifact_bytes,
                admitted.integrity_membership,
                self.store,
                self.format.declaration(),
            ))),
            identity: RecordReadIdentity::for_extent(
                self.store,
                self.generation,
                record,
                placement.extent_cell(),
            ),
            observation: *observation,
            runtime: self.runtime.clone(),
            health_permit,
            _lifecycle: self.lifecycle.read_session(),
            _allocation: allocation,
        })
    }
}

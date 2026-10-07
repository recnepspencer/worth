use worth_store_physical_format::SelectedRecordContentClass;

use super::{manifest_failure, PhysicalRecordReader};
use crate::physical_runtime::record_serving::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestReader,
};
use crate::physical_runtime::{
    PhysicalRecordId, RecordReadDenial, RecordReadError, RecordReadObservation,
};

impl PhysicalRecordReader {
    /// Reads only the route class selected by this protected root. The frame
    /// still has to be opened and authenticated separately by the caller.
    pub(in crate::physical_runtime) fn selected_content_class(
        &self,
        record: PhysicalRecordId,
    ) -> Result<SelectedRecordContentClass, RecordReadError> {
        let _call = self.admit_read_call()?;
        let mut observation = RecordReadObservation::default();
        self.protection.require_live().map_err(|_| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        runtime.health.permit().map_err(|_| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        let allocation = self.begin_record_read_allocation(record, observation)?;
        let mut discovery = ManifestDiscoveryCounterSnapshot::default();
        let placement = ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            self.current_root.clone(),
        )
        .locate(&allocation, record.persisted(), &mut discovery);
        observation.observe_manifest(discovery);
        placement
            .map_err(|failure| {
                self.read_error_for_record(record, manifest_failure(failure), observation)
            })?
            .map(|selected| selected.content_class())
            .ok_or_else(|| RecordReadError::new(RecordReadDenial::RecordNotFound, observation))
    }
}

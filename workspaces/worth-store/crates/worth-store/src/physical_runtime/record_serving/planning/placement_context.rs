use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use super::super::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, RecordAllocationFrontier,
};

pub(in crate::physical_runtime::record_serving) struct PlacementPlanningContext<'plan> {
    pub(in crate::physical_runtime::record_serving) arena_owner:
        super::super::arena::SharedArenaAllocationOwner,
    pub(in crate::physical_runtime::record_serving) allocation:
        &'plan worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) media: &'plan QualifiedFilesystemMedia,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access:
        super::super::AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root:
        &'plan DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) current_free_space:
        &'plan DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) frontier: &'plan mut RecordAllocationFrontier,
    pub(in crate::physical_runtime::record_serving) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) residency:
        super::super::residency::PhysicalResidencyWorkPort,
}

use std::sync::{Arc, OnceLock};

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;

use super::{index_capacity, retention, InvalidationEditAdmission, SourceInvalidationOwner};

impl SourceInvalidationOwner {
    /// One prepared output witness owns its retained backing until the final
    /// lineage reference drops. The request charge precedes every allocation.
    pub(in crate::domain_computation::primary_graph) fn retain_native_output_witness<T>(
        &self,
        backing_bytes: u64,
        copy_work: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<RetainedInvalidationCapacity>, CompanionPreflightStop> {
        admission.charge_external_work(copy_work)?;
        let cell = index_capacity::arc_bytes::<OnceLock<T>>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bytes = backing_bytes
            .checked_add(cell)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.admit_read_scratch(bytes)?;
        retention::reserve(&self.resources, bytes, admission)
    }
}

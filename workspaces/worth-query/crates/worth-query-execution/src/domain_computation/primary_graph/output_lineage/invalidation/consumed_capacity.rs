use std::sync::Arc;

use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

use super::admission::IndexAdmission;
use super::{fact_retention, retention, InvalidationEditAdmission, SourceInvalidationOwner};

/// The consumed edge keeps its own custody, including when its originating
/// lineage/mark root retires. Both tickets share the installed host ledger.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RetainedConsumedOutputCapacity {
    _facts: Arc<RetainedInvalidationCapacity>,
    _metadata: Arc<RetainedInvalidationCapacity>,
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn retain_consumed_output(
        &self,
        facts: &[WorthQueryApplicationObservedFact],
        basis: &PositionedRelationalSnapshot,
        metadata_bytes: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetainedConsumedOutputCapacity, CompanionPreflightStop> {
        admission.work(basis.branch_id().0.len() as u64)?;
        let facts = fact_retention::reserve(facts, basis, &self.resources, admission)?;
        admission.bytes(metadata_bytes)?;
        let metadata = retention::reserve(&self.resources, metadata_bytes, admission)?;
        #[cfg(feature = "test-query-execution-observer")]
        {
            super::consumed_capacity_observation::observe(self.runtime_instance_id, &facts);
            super::consumed_capacity_observation::observe(self.runtime_instance_id, &metadata);
        }
        Ok(RetainedConsumedOutputCapacity {
            _facts: facts,
            _metadata: metadata,
        })
    }

    /// Normalize an admitted attempt's edge vector before candidate effects.
    /// The caller retains the returned custody with the Arc backing allocation.
    pub(in crate::domain_computation::primary_graph) fn retain_consumed_output_backing(
        &self,
        bytes: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<RetainedInvalidationCapacity>, CompanionPreflightStop> {
        admission.bytes(bytes)?;
        admission.work(bytes)?;
        let ticket = retention::reserve(&self.resources, bytes, admission)?;
        #[cfg(feature = "test-query-execution-observer")]
        super::consumed_capacity_observation::observe(self.runtime_instance_id, &ticket);
        Ok(ticket)
    }
}

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::application_checkpoint::facts::{
    FactPayload, FactRetentionAllocation,
};

use super::admission::IndexAdmission;
use super::{fact_retention, retention, InvalidationEditAdmission, SourceInvalidationOwner};

/// Edges keep actual original facts alive after lineage/mark retirement. Facts
/// share one allocation ticket in this ledger; each edge funds its own basis
/// and metadata. A requested read has no facts and claims no currentness.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RetainedConsumedOutputCapacity {
    _facts: Option<Arc<FactRetentionAllocation>>,
    _basis: Arc<RetainedInvalidationCapacity>,
    _metadata: Arc<RetainedInvalidationCapacity>,
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn retain_consumed_output(
        &self,
        facts: &FactPayload,
        basis: &PositionedRelationalSnapshot,
        metadata_bytes: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetainedConsumedOutputCapacity, CompanionPreflightStop> {
        self.retain_output_capacity(Some(facts), basis, metadata_bytes, admission)
    }

    /// A pending native read keeps scheduling context, not source facts or a
    /// currentness proof. It has no fictitious empty fact allocation to fund.
    pub(in crate::domain_computation::primary_graph) fn retain_requested_output(
        &self,
        basis: &PositionedRelationalSnapshot,
        metadata_bytes: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetainedConsumedOutputCapacity, CompanionPreflightStop> {
        self.retain_output_capacity(None, basis, metadata_bytes, admission)
    }

    fn retain_output_capacity(
        &self,
        facts: Option<&FactPayload>,
        basis: &PositionedRelationalSnapshot,
        metadata_bytes: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RetainedConsumedOutputCapacity, CompanionPreflightStop> {
        admission.work(basis.branch_id().0.len() as u64)?;
        let facts = facts
            .map(|facts| fact_retention::reserve_payload(facts, &self.resources, admission))
            .transpose()?;
        let basis = fact_retention::reserve_basis(basis, &self.resources, admission)?;
        admission.bytes(metadata_bytes)?;
        let metadata = retention::reserve(&self.resources, metadata_bytes, admission)?;
        if let Some(ticket) = &facts {
            super::consumed_capacity_observation::observe(
                self.runtime_instance_id,
                &ticket.capacity,
            );
        }
        super::consumed_capacity_observation::observe(self.runtime_instance_id, &basis);
        super::consumed_capacity_observation::observe(self.runtime_instance_id, &metadata);
        Ok(RetainedConsumedOutputCapacity {
            _facts: facts,
            _basis: basis,
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
        super::consumed_capacity_observation::observe(self.runtime_instance_id, &ticket);
        Ok(ticket)
    }
}

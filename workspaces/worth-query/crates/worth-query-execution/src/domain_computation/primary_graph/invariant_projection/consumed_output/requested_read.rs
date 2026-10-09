//! Scheduling custody from an output the producer actually consumed.
use super::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::{
    invariant_projection::RequestedOutputRead,
    output_lineage::invalidation::InvalidationEditAdmission, SourceInvalidationOwner,
};
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl ConsumedOutputEvidence {
    /// A native read may be Current before its managed Ready delivery completes.
    /// This separately funded packet lets the existing driver finish that delivery;
    /// it supplies scheduling evidence, never publication or currentness authority.
    pub(in crate::domain_computation::primary_graph) fn requested_read(
        &self,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequestedOutputRead, CompanionPreflightStop> {
        let selected = self.selected_native_root.as_ref();
        let bytes = std::mem::size_of::<RequestedOutputRead>() as u64;
        admission.charge_external_work(bytes + selected.branch_id().0.len() as u64)?;
        let capacity = owner.retain_consumed_output(&[], selected, bytes, admission)?;
        Ok(RequestedOutputRead::new(
            Arc::clone(&self.identity),
            selected.clone(),
            capacity,
        ))
    }
}

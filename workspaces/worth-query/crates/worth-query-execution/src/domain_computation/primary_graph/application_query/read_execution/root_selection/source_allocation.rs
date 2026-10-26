//! Pre-growth source/index admission, activated only by complete-walk reads.
use super::*;
use crate::domain_computation::primary_graph::application_query::observed_source::RootPathAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl RootSelectionWork<'_> {
    pub(super) fn activate_source_allocation(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<RootPathAllocation>, WorthQueryApplicationReadExecutionDenial> {
        if self.source_allocation.is_none() {
            self.source_allocation = Some(RootPathAllocation::open(execution)?);
        }
        Ok(Arc::clone(
            self.source_allocation
                .as_ref()
                .expect("source backing was opened"),
        ))
    }

    pub(super) fn admit_root_entry<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if let Some(allocation) = &self.source_allocation {
            allocation.claim_tree::<(EntityId, T)>(execution, len)?;
        }
        Ok(())
    }

    pub(super) fn admit_candidates(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if let Some(allocation) = &self.source_allocation {
            let bytes = len
                .checked_mul(size_of::<EntityId>())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| {
                    super::super::denial::relational_read_denial(
                        Cause::ChargedBytesOverflow,
                        "closure candidates",
                    )
                })?;
            allocation.claim(execution, bytes)?;
        }
        Ok(())
    }
}

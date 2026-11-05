//! Checked private partition issuance for application mutation attempts.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

/// One live issuer, initialized only from its actual Native allocation owner.
pub(in crate::domain_computation::primary_graph) struct ApplicationMutationPartitions {
    next: AtomicU32,
}
impl ApplicationMutationPartitions {
    pub(in crate::domain_computation::primary_graph) fn readmit(
        native: &worth_relational::facade::runtime::RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        let highest = native.observe_record_partition_high_water(request)?;
        // MAX is the exhausted sentinel, retaining the prior checked issuance
        // contract even when no successor of the recovered namespace exists.
        let next = highest.map_or(1, |partition| {
            partition.as_u32().checked_add(1).unwrap_or(u32::MAX)
        });
        Ok(Self {
            next: AtomicU32::new(next),
        })
    }

    fn issue(&self) -> Option<worth_relational::facade::identity::PartitionId> {
        self.next
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()
            .map(worth_relational::facade::identity::PartitionId)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn issue_application_mutation_partition(
        &self,
    ) -> Option<worth_relational::facade::identity::PartitionId> {
        self.runtime.mutation_partitions.issue()
    }
}

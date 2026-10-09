use std::sync::Arc;

use worth_execution::{ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder};
use worth_relational::facade::{
    mvcc::{CompanionPreflightStop, PreparedRelationalCommitCandidate},
    transactions::RecordRef,
};

use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    provider::mutation_work::WorthQueryTouchedRecordIdentity,
};
use crate::domain_computation::WorthQueryProviderSessionCommitStop;

/// Exact prepared native identities, physically admitted and sealed before any
/// publication. The same immutable backing follows every receipt and recovery
/// holder; Arc headers remain outside the array's payload charge.
pub(in crate::domain_computation::primary_graph::provider) struct PreparedTouchedRecords {
    records: Arc<ExecutionArray<WorthQueryTouchedRecordIdentity>>,
}

impl PreparedTouchedRecords {
    pub(super) fn prepare(
        candidate: &PreparedRelationalCommitCandidate,
        admission: &mut InvalidationEditAdmission,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryProviderSessionCommitStop> {
        admission
            .charge_external_work(1)
            .map_err(super::stops::touched_record_preparation_stop)?;
        candidate
            .with_prepared_changed_records(|exact| {
                admission
                    .charge_external_work(
                        u64::try_from(exact.len())
                            .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)
                            .map_err(super::stops::touched_record_preparation_stop)?,
                    )
                    .map_err(super::stops::touched_record_preparation_stop)?;
                let mut records = ExecutionArrayBuilder::allocate(exact.len(), allocation_policy)
                    .map_err(allocation_denied)?;
                for record in exact {
                    records
                        .push(WorthQueryTouchedRecordIdentity::from_commit_record(
                            record.clone(),
                        ))
                        .map_err(allocation_denied)?;
                }
                Ok(Self {
                    records: Arc::new(records.seal().map_err(allocation_denied)?),
                })
            })
            .unwrap_or_else(|| {
                Err(WorthQueryProviderSessionCommitStop::PreEffectDenied(
                    super::failure(
                        "native candidate no longer retains its prepared changed records",
                    ),
                ))
            })
    }

    /// Publication already occurred. This only checks the native performed
    /// identities and moves sealed custody; it cannot allocate or poll a stop.
    pub(in crate::domain_computation::primary_graph::provider) fn verify_performed(
        self,
        exact: &[RecordRef],
    ) -> Arc<ExecutionArray<WorthQueryTouchedRecordIdentity>> {
        assert_eq!(
            exact.len(),
            self.records.len(),
            "prepared native record count must match its performed result"
        );
        for (prepared, performed) in self.records.iter().zip(exact) {
            assert_eq!(
                prepared.record(),
                performed,
                "prepared native identities and order must match publication"
            );
        }
        self.records
    }
}

fn allocation_denied(
    denial: worth_execution::ExecutionAllocationDenial,
) -> WorthQueryProviderSessionCommitStop {
    WorthQueryProviderSessionCommitStop::PreEffectDenied(
        crate::domain_computation::WorthQueryProviderSessionFailure::allocation_denied(denial),
    )
}

#[cfg(test)]
mod tests;

use std::sync::Arc;

use worth_execution::ExecutionAllocationPolicy;
use worth_execution::ExecutionArray;
use worth_execution::ExecutionArrayBuilder;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::primary_graph::provider::mutation_work::WorthQueryTouchedRecordIdentity;
use crate::domain_computation::WorthQueryProviderSessionCommitStop;

/// Exact prepared native identities, physically admitted and sealed before any
/// publication. The same immutable backing follows every receipt and recovery
/// holder; Arc headers remain outside the array's payload charge.
pub struct PreparedTouchedRecords {
    records: Arc<ExecutionArray<WorthQueryTouchedRecordIdentity>>,
}

impl PreparedTouchedRecords {
    pub(super) fn prepare(
        candidate: &PreparedRelationalCommitCandidate,

        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryProviderSessionCommitStop> {
        Self::prepare_retained(candidate, allocation_policy)
            .map_err(TouchedRecordsPreparationDenial::into_memory_stop)
    }

    pub(super) fn prepare_retained(
        candidate: &PreparedRelationalCommitCandidate,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, TouchedRecordsPreparationDenial> {
        candidate
            .with_prepared_changed_records(|exact| {
                Self::from_records_retained(exact, allocation_policy)
            })
            .unwrap_or_else(|| Err(TouchedRecordsPreparationDenial::ChangedRecordsUnavailable))
    }

    pub(in crate::domain_computation::primary_graph::provider) fn from_records(
        exact: &[RecordRef],
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryProviderSessionCommitStop> {
        Self::from_records_retained(exact, allocation_policy)
            .map_err(TouchedRecordsPreparationDenial::into_memory_stop)
    }
    fn from_records_retained(
        exact: &[RecordRef],
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, TouchedRecordsPreparationDenial> {
        let mut records = ExecutionArrayBuilder::allocate(exact.len(), allocation_policy)
            .map_err(TouchedRecordsPreparationDenial::Allocation)?;
        for record in exact {
            records
                .push(WorthQueryTouchedRecordIdentity::from_commit_record(
                    record.clone(),
                ))
                .map_err(TouchedRecordsPreparationDenial::Allocation)?;
        }
        Ok(Self {
            records: Arc::new(
                records
                    .seal()
                    .map_err(TouchedRecordsPreparationDenial::Allocation)?,
            ),
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

#[derive(Debug)]
pub(super) enum TouchedRecordsPreparationDenial {
    Allocation(worth_execution::ExecutionAllocationDenial),
    ChangedRecordsUnavailable,
}
impl TouchedRecordsPreparationDenial {
    pub(super) fn into_memory_stop(self) -> WorthQueryProviderSessionCommitStop {
        WorthQueryProviderSessionCommitStop::PreEffectDenied(match self {
            Self::Allocation(cause) => {
                crate::domain_computation::WorthQueryProviderSessionFailure::allocation_denied(
                    cause,
                    "application touched records",
                )
            }
            Self::ChangedRecordsUnavailable => {
                super::failure("native candidate no longer retains its prepared changed records")
            }
        })
    }
}

mod court_doors_for_test;

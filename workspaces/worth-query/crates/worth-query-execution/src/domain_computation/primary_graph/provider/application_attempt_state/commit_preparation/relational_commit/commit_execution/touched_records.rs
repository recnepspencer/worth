use std::sync::{Arc, OnceLock};

use worth_relational::facade::{
    mvcc::{CompanionPreflightStop, PreparedRelationalCommitCandidate},
    transactions::RecordRef,
};

use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    provider::mutation_work::WorthQueryTouchedRecordIdentity,
};
use crate::domain_computation::WorthQueryProviderSessionCommitStop;

/// One exact prepared native record count funds the Query receipt's public
/// touched-record slice before World can perform its owner effect.
pub(in crate::domain_computation::primary_graph::provider) struct PreparedTouchedRecords {
    expected: usize,
    records: Vec<WorthQueryTouchedRecordIdentity>,
    retained: Arc<RetainedTouchedRecords>,
}

pub(in crate::domain_computation::primary_graph) struct RetainedTouchedRecords {
    records: OnceLock<Vec<WorthQueryTouchedRecordIdentity>>,
    _ticket: crate::domain_computation::primary_graph::provider::completed_evidence_capacity::CompletedEvidenceTicket,
}

impl RetainedTouchedRecords {
    pub(in crate::domain_computation::primary_graph::provider) fn as_slice(
        &self,
    ) -> &[WorthQueryTouchedRecordIdentity] {
        self.records
            .get()
            .expect("performed evidence fills its prepared cell")
    }
}

impl std::fmt::Debug for RetainedTouchedRecords {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("RetainedTouchedRecords")
            .field(&self.records.get())
            .finish()
    }
}

impl PartialEq for RetainedTouchedRecords {
    fn eq(&self, other: &Self) -> bool {
        self.records.get() == other.records.get()
    }
}

impl Eq for RetainedTouchedRecords {}

impl PreparedTouchedRecords {
    pub(super) fn prepare(
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        candidate: &PreparedRelationalCommitCandidate,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryProviderSessionCommitStop> {
        admission
            .charge_external_work(1)
            .map_err(super::stops::touched_record_preparation_stop)?;
        let expected = candidate
            .prepared_changed_record_count()
            .expect("the owner still retains its prepared native publication");
        let slots = expected
            .checked_mul(std::mem::size_of::<WorthQueryTouchedRecordIdentity>())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
            .map_err(super::stops::touched_record_preparation_stop)?;
        let cell = std::mem::size_of::<RetainedTouchedRecords>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(std::mem::align_of::<RetainedTouchedRecords>()))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
            .map_err(super::stops::touched_record_preparation_stop)?;
        let retained_bytes = slots
            .checked_add(cell)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
            .map_err(super::stops::touched_record_preparation_stop)?;
        let bytes = u64::try_from(retained_bytes)
            .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)
            .map_err(super::stops::touched_record_preparation_stop)?;
        admission
            .admit_read_scratch(bytes)
            .map_err(super::stops::touched_record_preparation_stop)?;
        admission
            .charge_external_work(
                u64::try_from(expected)
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)
                    .map_err(super::stops::touched_record_preparation_stop)?,
            )
            .map_err(super::stops::touched_record_preparation_stop)?;
        let ticket = provider
            .completed_evidence_capacity
            .reserve(retained_bytes)
            .ok_or_else(|| {
                WorthQueryProviderSessionCommitStop::Deferred(
                    crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
                        crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RetentionCapacityExhausted,
                        "completed evidence retention capacity is exhausted",
                    ),
                )
            })?;
        let mut records = Vec::new();
        records.try_reserve_exact(expected).map_err(|_| {
            WorthQueryProviderSessionCommitStop::PreEffectDenied(super::failure(
                "prepared touched-record receipt storage could not be allocated",
            ))
        })?;
        let retained = Arc::new(RetainedTouchedRecords {
            records: OnceLock::new(),
            _ticket: ticket,
        });
        Ok(Self {
            expected,
            records,
            retained,
        })
    }

    pub(in crate::domain_computation::primary_graph::provider) fn fill(
        mut self,
        exact: &[RecordRef],
    ) -> Arc<RetainedTouchedRecords> {
        assert_eq!(
            exact.len(),
            self.expected,
            "prepared native record count must match its performed result"
        );
        for record in exact {
            self.records
                .push(WorthQueryTouchedRecordIdentity::from_commit_record(
                    record.clone(),
                ));
        }
        self.retained
            .records
            .set(self.records)
            .expect("a prepared touched-record cell fills once");
        self.retained
    }
}

#[cfg(test)]
mod tests {
    use super::{RetainedTouchedRecords, WorthQueryTouchedRecordIdentity};
    use crate::domain_computation::primary_graph::provider::completed_evidence_capacity::CompletedEvidenceCapacity;
    use std::sync::{Arc, OnceLock};

    #[test]
    fn completed_evidence_clones_share_one_refundable_custody() {
        let capacity = CompletedEvidenceCapacity::new(256);
        let held = Arc::new(RetainedTouchedRecords {
            records: OnceLock::<Vec<WorthQueryTouchedRecordIdentity>>::new(),
            _ticket: capacity.reserve(256).expect("one completed backing fits"),
        });
        assert!(capacity.reserve(1).is_none());
        let receipt_observation = Arc::clone(&held);
        drop(held);
        assert_eq!(capacity.retained_bytes(), 256);
        drop(receipt_observation);
        assert_eq!(capacity.retained_bytes(), 0);
        assert!(capacity.reserve(256).is_some());
    }
}

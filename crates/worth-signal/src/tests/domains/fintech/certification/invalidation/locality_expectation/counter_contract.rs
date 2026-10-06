use std::collections::BTreeMap;

use super::candidates::ExpectedCandidateManifest;
use super::trace::ExpectedTrace;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) enum ExpectedLocalityCounterRow {
    SourceOutputDeltasConsumed,
    DirectSubscriberEdgesExamined,
    ReverseIndexBucketProbes,
    ReverseIndexCandidatesReturned,
    CandidatesRejectedByAspectContract,
    CandidatesRejectedByScope,
    CandidatesRejectedByComparator,
    DirectSettlementsProduced,
    WorkItemsAdmitted,
    WorkItemsMerged,
    ReadyItemsEnqueued,
    ReadyItemsPopped,
    StaleWorkRejected,
    NodesEvaluated,
    ProducedDeltasEmitted,
    PropagationStops,
    NonSemanticNodeVisits,
    MaximumReadyFrontierWidth,
    RetainedReadyFrontierWidth,
    TopologyRevisionRevalidations,
    RejectedTopologyMutations,
    BatchLocalAllocations,
    PeakBatchMemoryItems,
    RecoveryReconstructionWork,
}

impl ExpectedLocalityCounterRow {
    pub(in crate::tests::domains::fintech) const fn is_physical_batch_shape(self) -> bool {
        matches!(
            self,
            Self::MaximumReadyFrontierWidth
                | Self::BatchLocalAllocations
                | Self::PeakBatchMemoryItems
        )
    }

    pub(in crate::tests::domains::fintech) const ALL: [Self; 24] = [
        Self::SourceOutputDeltasConsumed,
        Self::DirectSubscriberEdgesExamined,
        Self::ReverseIndexBucketProbes,
        Self::ReverseIndexCandidatesReturned,
        Self::CandidatesRejectedByAspectContract,
        Self::CandidatesRejectedByScope,
        Self::CandidatesRejectedByComparator,
        Self::DirectSettlementsProduced,
        Self::WorkItemsAdmitted,
        Self::WorkItemsMerged,
        Self::ReadyItemsEnqueued,
        Self::ReadyItemsPopped,
        Self::StaleWorkRejected,
        Self::NodesEvaluated,
        Self::ProducedDeltasEmitted,
        Self::PropagationStops,
        Self::NonSemanticNodeVisits,
        Self::MaximumReadyFrontierWidth,
        Self::RetainedReadyFrontierWidth,
        Self::TopologyRevisionRevalidations,
        Self::RejectedTopologyMutations,
        Self::BatchLocalAllocations,
        Self::PeakBatchMemoryItems,
        Self::RecoveryReconstructionWork,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct ExpectedLocalityCounterManifest {
    rows: BTreeMap<ExpectedLocalityCounterRow, u64>,
}

impl ExpectedLocalityCounterManifest {
    pub(super) fn derive(
        trace: &ExpectedTrace,
        candidates: &ExpectedCandidateManifest,
        peak_ready_width: u64,
    ) -> Self {
        let producer_deltas_consumed = trace.deltas.len() as u64;
        let enqueued = trace.evaluation_occurrences;
        let work_admitted = enqueued + trace.retries;
        let work_merged = trace.retries;
        let popped = trace.evaluation_occurrences;
        let retained = enqueued
            .checked_sub(popped)
            .expect("expected ready pops cannot exceed enqueued work");
        assert_eq!(
            retained, 0,
            "the complete named trace settles every ready item: admitted={work_admitted} merges={work_merged} evaluations={}",
            trace.evaluation_occurrences,
        );
        let rows = [
            (
                ExpectedLocalityCounterRow::SourceOutputDeltasConsumed,
                producer_deltas_consumed,
            ),
            (
                ExpectedLocalityCounterRow::DirectSubscriberEdgesExamined,
                candidates.candidate_dependencies.len() as u64,
            ),
            (
                ExpectedLocalityCounterRow::ReverseIndexBucketProbes,
                candidates.queried_bucket_occurrences,
            ),
            (
                ExpectedLocalityCounterRow::ReverseIndexCandidatesReturned,
                candidates.candidate_dependencies.len() as u64,
            ),
            (
                ExpectedLocalityCounterRow::CandidatesRejectedByAspectContract,
                0,
            ),
            (
                ExpectedLocalityCounterRow::CandidatesRejectedByScope,
                candidates.scope_rejections,
            ),
            (
                ExpectedLocalityCounterRow::CandidatesRejectedByComparator,
                0,
            ),
            (
                ExpectedLocalityCounterRow::DirectSettlementsProduced,
                candidates.admitted_candidate_occurrences,
            ),
            (ExpectedLocalityCounterRow::WorkItemsAdmitted, work_admitted),
            (ExpectedLocalityCounterRow::WorkItemsMerged, work_merged),
            (ExpectedLocalityCounterRow::ReadyItemsEnqueued, enqueued),
            (ExpectedLocalityCounterRow::ReadyItemsPopped, popped),
            (
                ExpectedLocalityCounterRow::StaleWorkRejected,
                trace.stale_denials,
            ),
            (
                ExpectedLocalityCounterRow::NodesEvaluated,
                trace.evaluation_occurrences,
            ),
            (
                ExpectedLocalityCounterRow::ProducedDeltasEmitted,
                trace.deltas.len() as u64,
            ),
            (
                ExpectedLocalityCounterRow::PropagationStops,
                trace.stop_occurrences,
            ),
            (ExpectedLocalityCounterRow::NonSemanticNodeVisits, 0),
            (
                ExpectedLocalityCounterRow::MaximumReadyFrontierWidth,
                peak_ready_width,
            ),
            (
                ExpectedLocalityCounterRow::RetainedReadyFrontierWidth,
                retained,
            ),
            (
                ExpectedLocalityCounterRow::TopologyRevisionRevalidations,
                trace.topology_revalidations,
            ),
            (
                ExpectedLocalityCounterRow::RejectedTopologyMutations,
                trace.rejected_topology_mutations,
            ),
            (
                ExpectedLocalityCounterRow::BatchLocalAllocations,
                trace.ready_batch_allocation_count(),
            ),
            (
                ExpectedLocalityCounterRow::PeakBatchMemoryItems,
                peak_ready_width,
            ),
            (ExpectedLocalityCounterRow::RecoveryReconstructionWork, 0),
        ]
        .into_iter()
        .collect();
        Self { rows }
    }

    pub(in crate::tests::domains::fintech) fn rows(
        &self,
    ) -> &BTreeMap<ExpectedLocalityCounterRow, u64> {
        &self.rows
    }

    pub(in crate::tests::domains::fintech) fn value(&self, row: ExpectedLocalityCounterRow) -> u64 {
        self.rows[&row]
    }
}

use super::super::PreparedDirectCounterDeltas;
use crate::data::graph::SignalGraph;
use crate::data::telemetry::InvalidationPerformedCounter;

impl PreparedDirectCounterDeltas {
    pub(super) fn publish(self, graph: &SignalGraph) {
        let performed = graph.invalidation_performed_counter_state();
        performed.add(
            InvalidationPerformedCounter::SourceOutputDeltasConsumed,
            self.source_deltas,
        );
        performed.add(
            InvalidationPerformedCounter::DirectSubscriberEdgesExamined,
            self.edges_examined,
        );
        performed.add(
            InvalidationPerformedCounter::ReverseIndexBucketProbes,
            self.bucket_probes,
        );
        performed.add(
            InvalidationPerformedCounter::ReverseIndexCandidatesReturned,
            self.candidates_returned,
        );
        performed.add(
            InvalidationPerformedCounter::CandidatesRejectedByAspectContract,
            self.aspect_contract_rejections,
        );
        performed.add(
            InvalidationPerformedCounter::CandidatesRejectedByScope,
            self.scope_rejections,
        );
        performed.add(
            InvalidationPerformedCounter::CandidatesRejectedByComparator,
            self.comparator_rejections,
        );
        performed.add(
            InvalidationPerformedCounter::DirectSettlementsProduced,
            self.settlements,
        );
    }
}

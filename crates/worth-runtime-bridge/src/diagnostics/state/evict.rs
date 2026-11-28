use std::borrow::Borrow;

use super::*;

impl BridgeDiagnosticsState {
    pub(crate) fn evict_route_indexes(&mut self, evicted: &Arc<BridgeRouteRecord>) {
        evict_index_entry(
            &mut self.latest_route_by_route_identity,
            evicted.route_identity(),
            evicted,
        );
        evict_index_entry(
            &mut self.latest_route_by_invalidation_identity,
            evicted.invalidation_identity(),
            evicted,
        );
        evict_index_entry(
            &mut self.latest_route_by_source_commit,
            evicted.source_commit(),
            evicted,
        );
    }

    pub(crate) fn evict_bulk_indexes(&mut self, evicted: &Arc<BridgeCanonicalBulkPlanRecord>) {
        evict_index_entry(
            &mut self.latest_bulk_by_workload_identity,
            evicted.workload_identity().as_str(),
            evicted,
        );
    }

    pub(crate) fn evict_continuity_indexes(
        &mut self,
        evicted: &Arc<BridgeCanonicalContinuityRecord>,
    ) {
        evict_index_entry(
            &mut self.latest_continuity_by_route_identity,
            evicted.route_identity(),
            evicted,
        );
    }

    pub(crate) fn evict_historical_indexes(
        &mut self,
        evicted: &Arc<BridgeCanonicalHistoricalEvaluationRecord>,
    ) {
        evict_index_entry(
            &mut self.latest_historical_by_record_identity,
            evicted.record_identity().as_str(),
            evicted,
        );
        evict_index_entry(
            &mut self.latest_historical_by_decision_log_identity,
            evicted.decision_log().decision_log_identity().as_str(),
            evicted,
        );
    }

    pub(crate) fn evict_historical_failure_indexes(
        &mut self,
        evicted: &Arc<BridgeHistoricalEvaluationFailureRecord>,
    ) {
        evict_index_entry(
            &mut self.latest_historical_failure_by_declaration_identity,
            evicted.declaration_identity().as_str(),
            evicted,
        );
        evict_index_entry(
            &mut self.latest_historical_failure_by_failure_identity,
            evicted.failure_identity().as_str(),
            evicted,
        );
    }

    pub(crate) fn evict_source_materialization_indexes(
        &mut self,
        evicted: &Arc<SourceMaterializationRecord>,
    ) {
        evict_index_entry(
            &mut self.latest_source_materialization_by_record_identity,
            evicted.record_identity().as_str(),
            evicted,
        );
    }

    pub(crate) fn evict_source_failure_indexes(&mut self, evicted: &Arc<SourceFailureRecord>) {
        evict_index_entry(
            &mut self.latest_source_failure_by_declaration_identity,
            evicted.declaration_identity().as_str(),
            evicted,
        );
        evict_index_entry(
            &mut self.latest_source_failure_by_failure_identity,
            evicted.failure_identity().as_str(),
            evicted,
        );
    }
}

fn evict_index_entry<K, Q, T>(index: &mut BTreeMap<K, Arc<T>>, key: &Q, evicted: &Arc<T>)
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    let Some(latest) = index.get(key) else {
        return;
    };

    if Arc::ptr_eq(latest, evicted) {
        index.remove(key);
    }
}

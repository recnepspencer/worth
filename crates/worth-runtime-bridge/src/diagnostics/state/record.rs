use super::*;

impl BridgeDiagnosticsState {
    pub(crate) fn record_route(&mut self, record: BridgeRouteRecord, limit: usize) {
        let record = Arc::new(record);
        self.latest_route_by_route_identity
            .insert(record.route_identity().clone(), Arc::clone(&record));
        self.latest_route_by_invalidation_identity
            .insert(record.invalidation_identity().clone(), Arc::clone(&record));
        self.latest_route_by_source_commit
            .insert(record.source_commit().clone(), Arc::clone(&record));
        self.route_records.push_back(record);
        while self.route_records.len() > limit.max(1) {
            if let Some(evicted) = self.route_records.pop_front() {
                self.evict_route_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_bulk(&mut self, record: BridgeCanonicalBulkPlanRecord, limit: usize) {
        let record = Arc::new(record);
        self.latest_bulk_by_workload_identity.insert(
            record.workload_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.bulk_records.push_back(record);
        while self.bulk_records.len() > limit.max(1) {
            if let Some(evicted) = self.bulk_records.pop_front() {
                self.evict_bulk_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_failure(&mut self, record: BridgeFailureRecord, limit: usize) {
        self.failure_records.push_back(Arc::new(record));
        trim_retained_records(&mut self.failure_records, limit, |_| {});
    }

    pub(crate) fn record_continuity(
        &mut self,
        record: BridgeCanonicalContinuityRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_continuity_by_route_identity
            .insert(record.route_identity().clone(), Arc::clone(&record));
        self.continuity_records.push_back(record);
        while self.continuity_records.len() > limit.max(1) {
            if let Some(evicted) = self.continuity_records.pop_front() {
                self.evict_continuity_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_historical(
        &mut self,
        record: BridgeCanonicalHistoricalEvaluationRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_historical_by_record_identity.insert(
            record.record_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.latest_historical_by_decision_log_identity.insert(
            record
                .decision_log()
                .decision_log_identity()
                .as_str()
                .to_string(),
            Arc::clone(&record),
        );
        self.historical_records.push_back(record);
        while self.historical_records.len() > limit.max(1) {
            if let Some(evicted) = self.historical_records.pop_front() {
                self.evict_historical_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_historical_failure(
        &mut self,
        record: BridgeHistoricalEvaluationFailureRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_historical_failure_by_declaration_identity
            .insert(
                record.declaration_identity().as_str().to_string(),
                Arc::clone(&record),
            );
        self.latest_historical_failure_by_failure_identity.insert(
            record.failure_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.historical_failures.push_back(record);
        while self.historical_failures.len() > limit.max(1) {
            if let Some(evicted) = self.historical_failures.pop_front() {
                self.evict_historical_failure_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_source_materialization(
        &mut self,
        record: SourceMaterializationRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_source_materialization_by_record_identity
            .insert(
                record.record_identity().as_str().to_string(),
                Arc::clone(&record),
            );
        self.source_materialization_records.push_back(record);
        while self.source_materialization_records.len() > limit.max(1) {
            if let Some(evicted) = self.source_materialization_records.pop_front() {
                self.evict_source_materialization_indexes(&evicted);
            }
        }
    }

    pub(crate) fn record_source_failure(&mut self, record: SourceFailureRecord, limit: usize) {
        let record = Arc::new(record);
        self.latest_source_failure_by_declaration_identity.insert(
            record.declaration_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.latest_source_failure_by_failure_identity.insert(
            record.failure_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.source_failure_records.push_back(record);
        while self.source_failure_records.len() > limit.max(1) {
            if let Some(evicted) = self.source_failure_records.pop_front() {
                self.evict_source_failure_indexes(&evicted);
            }
        }
    }
}

fn trim_retained_records<T, F>(records: &mut VecDeque<T>, limit: usize, mut on_evict: F)
where
    F: FnMut(&T),
{
    while records.len() > limit.max(1) {
        if let Some(evicted) = records.pop_front() {
            on_evict(&evicted);
        }
    }
}

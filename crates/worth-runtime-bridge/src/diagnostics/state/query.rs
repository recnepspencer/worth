use super::*;

impl BridgeDiagnosticsState {
    pub(crate) fn route_records(&self) -> Vec<BridgeRouteRecord> {
        self.route_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn bulk_records(&self) -> Vec<BridgeCanonicalBulkPlanRecord> {
        self.bulk_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn failure_records(&self) -> Vec<BridgeFailureRecord> {
        self.failure_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn continuity_records(&self) -> Vec<BridgeCanonicalContinuityRecord> {
        self.continuity_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn historical_records(&self) -> Vec<BridgeCanonicalHistoricalEvaluationRecord> {
        self.historical_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn historical_failures(&self) -> Vec<BridgeHistoricalEvaluationFailureRecord> {
        self.historical_failures
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn source_materialization_records(&self) -> Vec<SourceMaterializationRecord> {
        self.source_materialization_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn source_failure_records(&self) -> Vec<SourceFailureRecord> {
        self.source_failure_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn last_route_record(&self) -> Option<BridgeRouteRecord> {
        self.route_records.back().map(|record| (**record).clone())
    }

    pub(crate) fn last_failure_record(&self) -> Option<BridgeFailureRecord> {
        self.failure_records.back().map(|record| (**record).clone())
    }

    pub(crate) fn last_bulk_record(&self) -> Option<BridgeCanonicalBulkPlanRecord> {
        self.bulk_records.back().map(|record| (**record).clone())
    }

    pub(crate) fn last_continuity_record(&self) -> Option<BridgeCanonicalContinuityRecord> {
        self.continuity_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_historical_record(
        &self,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.historical_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_historical_failure(
        &self,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.historical_failures
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_source_materialization_record(&self) -> Option<SourceMaterializationRecord> {
        self.source_materialization_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_source_failure_record(&self) -> Option<SourceFailureRecord> {
        self.source_failure_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn route_record_for_route_identity(
        &self,
        route_identity: &BridgeRouteIdentity,
    ) -> Option<BridgeRouteRecord> {
        self.latest_route_by_route_identity
            .get(route_identity)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn route_record_for_invalidation_identity(
        &self,
        invalidation_identity: &BridgeInvalidationIdentity,
    ) -> Option<BridgeRouteRecord> {
        self.latest_route_by_invalidation_identity
            .get(invalidation_identity)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn route_record_for_source_commit(
        &self,
        source_commit: &crate::input::envelope::TruthCommitIdentity,
    ) -> Option<BridgeRouteRecord> {
        self.latest_route_by_source_commit
            .get(source_commit)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn continuity_record_for_route_identity(
        &self,
        route_identity: &BridgeRouteIdentity,
    ) -> Option<BridgeCanonicalContinuityRecord> {
        self.latest_continuity_by_route_identity
            .get(route_identity)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn bulk_record_for_workload_identity(
        &self,
        workload_identity: &BridgeWorkloadIdentity,
    ) -> Option<BridgeCanonicalBulkPlanRecord> {
        self.latest_bulk_by_workload_identity
            .get(workload_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn historical_record_for_record_identity(
        &self,
        record_identity: &BridgeHistoricalEvaluationRecordIdentity,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.latest_historical_by_record_identity
            .get(record_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn historical_record_for_decision_log_identity(
        &self,
        decision_log_identity: &BridgeHistoricalEvaluationDecisionLogIdentity,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.latest_historical_by_decision_log_identity
            .get(decision_log_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn historical_failure_for_declaration_identity(
        &self,
        declaration_identity: &crate::policy::BridgePolicyDeclarationIdentity,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.latest_historical_failure_by_declaration_identity
            .get(declaration_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn source_materialization_record_for_identity(
        &self,
        record_identity: &crate::source::SourceMaterializationRecordIdentity,
    ) -> Option<SourceMaterializationRecord> {
        self.latest_source_materialization_by_record_identity
            .get(record_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn source_failure_for_declaration_identity(
        &self,
        declaration_identity: &str,
    ) -> Option<SourceFailureRecord> {
        self.latest_source_failure_by_declaration_identity
            .get(declaration_identity)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn historical_failure_for_identity(
        &self,
        failure_identity: &BridgeHistoricalEvaluationFailureIdentity,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.latest_historical_failure_by_failure_identity
            .get(failure_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn source_failure_record_for_identity(
        &self,
        failure_identity: &crate::source::SourceFailureRecordIdentity,
    ) -> Option<SourceFailureRecord> {
        self.latest_source_failure_by_failure_identity
            .get(failure_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }
}

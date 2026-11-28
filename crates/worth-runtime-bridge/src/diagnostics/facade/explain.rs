use super::*;

impl BridgeDiagnosticsFacade {
    pub fn explain_route_record(&self, record: &BridgeRouteRecord) -> BridgeRouteExplanation {
        BridgeRouteExplanation::from_route_record(record)
    }

    pub fn explain_last_route_record(&self) -> Option<BridgeRouteExplanation> {
        self.last_route_record()
            .map(|record| BridgeRouteExplanation::from_route_record(&record))
    }

    pub fn explain_continuity_record(
        &self,
        record: &BridgeCanonicalContinuityRecord,
    ) -> BridgeContinuityExplanation {
        BridgeContinuityExplanation::from_canonical_record(record)
    }

    pub fn explain_last_continuity_record(&self) -> Option<BridgeContinuityExplanation> {
        self.last_canonical_continuity_record()
            .map(|record| BridgeContinuityExplanation::from_canonical_record(&record))
    }

    pub fn explain_bulk_record(
        &self,
        record: &BridgeCanonicalBulkPlanRecord,
    ) -> BridgeBulkPlanExplanation {
        BridgeBulkPlanExplanation::from_canonical_record(record)
    }

    pub fn explain_last_bulk_record(&self) -> Option<BridgeBulkPlanExplanation> {
        self.last_bulk_record()
            .map(|record| BridgeBulkPlanExplanation::from_canonical_record(&record))
    }

    pub fn explain_historical_evaluation_record(
        &self,
        record: &BridgeCanonicalHistoricalEvaluationRecord,
    ) -> BridgeHistoricalEvaluationExplanation {
        BridgeHistoricalEvaluationExplanation::from_canonical_record(record)
    }

    pub fn explain_last_historical_evaluation_record(
        &self,
    ) -> Option<BridgeHistoricalEvaluationExplanation> {
        self.last_historical_evaluation_record()
            .map(|record| BridgeHistoricalEvaluationExplanation::from_canonical_record(&record))
    }

    pub fn explain_source_materialization_record(
        &self,
        record: &crate::source::SourceMaterializationRecord,
    ) -> crate::diagnostics::BridgeSourceMaterializationExplanation {
        crate::diagnostics::BridgeSourceMaterializationExplanation::from_record(record)
    }

    pub fn explain_last_source_materialization_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeSourceMaterializationExplanation> {
        self.last_source_materialization_record().map(|record| {
            crate::diagnostics::BridgeSourceMaterializationExplanation::from_record(&record)
        })
    }

    pub fn explain_source_failure_record(
        &self,
        record: &crate::source::SourceFailureRecord,
    ) -> crate::diagnostics::BridgeSourceFailureExplanation {
        crate::diagnostics::BridgeSourceFailureExplanation::from_record(record)
    }

    pub fn explain_last_source_failure_record(
        &self,
    ) -> Option<crate::diagnostics::BridgeSourceFailureExplanation> {
        self.last_source_failure_record()
            .map(|record| crate::diagnostics::BridgeSourceFailureExplanation::from_record(&record))
    }

    pub fn last_canonical_route_record(&self) -> Option<BridgeCanonicalRouteRecord> {
        self.last_route_record()
            .map(BridgeCanonicalRouteRecord::from_route_record)
    }

    pub fn handle(&self) -> BridgeDiagnosticsHandle {
        BridgeDiagnosticsHandle {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
        }
    }
}

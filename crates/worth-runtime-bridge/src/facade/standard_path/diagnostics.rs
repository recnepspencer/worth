use super::*;
use crate::diagnostics::BridgeHistoricalEvaluationRecordIdentity;
use crate::routing::BridgeRouteIdentity;

#[derive(Debug, Clone)]
pub struct BridgeDiagnostics<'a> {
    inner: &'a BridgeDiagnosticsFacade,
}

impl<'a> BridgeDiagnostics<'a> {
    pub(crate) fn new(inner: &'a BridgeDiagnosticsFacade) -> Self {
        Self { inner }
    }

    pub fn retained_artifacts(&self) -> &'a BridgeDiagnosticsFacade {
        self.inner
    }

    /// Returns the richest available explanation for the most recent
    /// standard-path bridge action.
    pub fn explain_last(&self) -> Option<BridgeStandardDiagnosticsExplanation> {
        self.explain_last_evaluation()
            .map(BridgeStandardDiagnosticsExplanation::Evaluation)
            .or_else(|| {
                self.explain_last_route()
                    .map(BridgeStandardDiagnosticsExplanation::Route)
            })
    }

    /// Explains the most recent routed truth change.
    pub fn explain_last_route(&self) -> Option<BridgeRouteExplanation> {
        self.inner.explain_last_route_record()
    }

    /// Explains one routed truth change by route identity.
    pub fn explain_route(
        &self,
        route_identity: &BridgeRouteIdentity,
    ) -> Option<BridgeRouteExplanation> {
        self.inner
            .route_record_for_route_identity(route_identity)
            .map(|record| self.inner.explain_route_record(&record))
    }

    /// Explains the most recent explicit or implicit truth-view evaluation.
    pub fn explain_last_evaluation(&self) -> Option<BridgeHistoricalEvaluationExplanation> {
        self.inner.explain_last_historical_evaluation_record()
    }

    /// Explains one truth-view evaluation by retained record identity.
    pub fn explain_evaluation(
        &self,
        historical_record_identity: &BridgeHistoricalEvaluationRecordIdentity,
    ) -> Option<BridgeHistoricalEvaluationExplanation> {
        self.inner
            .historical_record_for_record_identity(historical_record_identity)
            .map(|record| self.inner.explain_historical_evaluation_record(&record))
    }
}

impl<'a> std::ops::Deref for BridgeDiagnostics<'a> {
    type Target = BridgeDiagnosticsFacade;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub enum BridgeStandardDiagnosticsExplanation {
    Route(BridgeRouteExplanation),
    Evaluation(BridgeHistoricalEvaluationExplanation),
}

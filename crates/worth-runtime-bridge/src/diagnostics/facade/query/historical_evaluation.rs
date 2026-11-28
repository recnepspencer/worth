use super::*;
use crate::diagnostics::history::{
    BridgeHistoricalEvaluationDecisionLogIdentity, BridgeHistoricalEvaluationFailureIdentity,
    BridgeHistoricalEvaluationRecordIdentity,
};
use crate::policy::BridgePolicyDeclarationIdentity;

impl BridgeDiagnosticsFacade {
    pub fn historical_evaluation_records(&self) -> Vec<BridgeCanonicalHistoricalEvaluationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_records()
    }

    pub fn historical_evaluation_failures(&self) -> Vec<BridgeHistoricalEvaluationFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_failures()
    }

    pub fn last_historical_evaluation_record(
        &self,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_historical_record()
    }

    pub fn last_historical_evaluation_failure(
        &self,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_historical_failure()
    }

    pub fn historical_record_for_record_identity(
        &self,
        record_identity: &BridgeHistoricalEvaluationRecordIdentity,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_record_for_record_identity(record_identity)
    }

    pub fn historical_record_for_decision_log_identity(
        &self,
        decision_log_identity: &BridgeHistoricalEvaluationDecisionLogIdentity,
    ) -> Option<BridgeCanonicalHistoricalEvaluationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_record_for_decision_log_identity(decision_log_identity)
    }

    pub fn historical_failure_for_declaration_identity(
        &self,
        declaration_identity: &BridgePolicyDeclarationIdentity,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_failure_for_declaration_identity(declaration_identity)
    }

    pub fn historical_failure_for_identity(
        &self,
        failure_identity: &BridgeHistoricalEvaluationFailureIdentity,
    ) -> Option<BridgeHistoricalEvaluationFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .historical_failure_for_identity(failure_identity)
    }
}

use std::sync::Arc;
use std::sync::Mutex;

use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceAssessment;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureObservedIncumbent {
    pub occurrence_identity: String,
    pub state_identity: String,
    pub report_evidence_identity: String,
}

impl FixtureObservedIncumbent {
    pub fn occurrence_identity(&self) -> &str {
        &self.occurrence_identity
    }

    pub fn state_identity(&self) -> &str {
        &self.state_identity
    }

    pub fn report_evidence_identity(&self) -> &str {
        &self.report_evidence_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureReportHistoryObservation {
    pub iteration_ordinal: usize,
    pub provider_receipt_identity: String,
    pub graph_evidence_identity: String,
    pub incumbents: Vec<FixtureObservedIncumbent>,
}

impl FixtureReportHistoryObservation {
    pub const fn iteration_ordinal(&self) -> usize {
        self.iteration_ordinal
    }

    pub fn provider_receipt_identity(&self) -> &str {
        &self.provider_receipt_identity
    }

    pub fn graph_evidence_identity(&self) -> &str {
        &self.graph_evidence_identity
    }

    pub fn incumbents(&self) -> &[FixtureObservedIncumbent] {
        &self.incumbents
    }
}

#[derive(Clone, Default)]
pub struct FixtureReportHistoryProbe {
    pub observations: Arc<Mutex<Vec<FixtureReportHistoryObservation>>>,
}

impl FixtureReportHistoryProbe {
    pub fn observe(&self, assessment: &WorthQueryConvergenceAssessment<'_>) {
        let receipt = assessment.receipt();
        let incumbents = assessment
            .incumbents()
            .iter()
            .map(|incumbent| FixtureObservedIncumbent {
                occurrence_identity: incumbent.occurrence_identity().to_owned(),
                state_identity: incumbent.state_identity().to_owned(),
                report_evidence_identity: incumbent.report_evidence_identity().to_owned(),
            })
            .collect();
        self.observations
            .lock()
            .expect("report history probe lock must remain available")
            .push(FixtureReportHistoryObservation {
                iteration_ordinal: assessment.iteration_ordinal(),
                provider_receipt_identity: receipt.provider_receipt().to_owned(),
                graph_evidence_identity: receipt.evidence_identity().to_owned(),
                incumbents,
            });
    }

    pub fn observations(&self) -> Vec<FixtureReportHistoryObservation> {
        self.observations
            .lock()
            .expect("report history probe lock must remain available")
            .clone()
    }
}

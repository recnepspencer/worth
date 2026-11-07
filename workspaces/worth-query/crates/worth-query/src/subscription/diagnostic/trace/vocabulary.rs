use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::evidence_identities::diagnostic_stage_trace_identity;
use super::super::bundle::QuerySubscriptionDiagnosticCounters;
use super::super::stage::{
    QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticOutcome,
    QuerySubscriptionDiagnosticStage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticStageTrace {
    stage: QuerySubscriptionDiagnosticStage,
    outcome: QuerySubscriptionDiagnosticOutcome,
    source_identity: WorthQueryEvidenceIdentity,
    evidence_identity: WorthQueryEvidenceIdentity,
    stage_trace_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionDiagnosticStageTrace {
    pub(super) fn from_evidence(
        evidence: &QuerySubscriptionDiagnosticEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let stage_trace_identity = diagnostic_stage_trace_identity(
            evidence.stage().as_str(),
            evidence.outcome().as_str(),
            evidence.source_identity(),
            evidence.evidence_identity(),
        )?;
        Ok(Self {
            stage: *evidence.stage(),
            outcome: *evidence.outcome(),
            source_identity: evidence.source_identity().clone(),
            evidence_identity: evidence.evidence_identity().clone(),
            stage_trace_identity,
        })
    }

    pub fn stage(&self) -> &QuerySubscriptionDiagnosticStage {
        &self.stage
    }

    pub fn outcome(&self) -> &QuerySubscriptionDiagnosticOutcome {
        &self.outcome
    }

    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }

    pub fn stage_trace_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.stage_trace_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticTrace {
    terminal_stage: QuerySubscriptionDiagnosticStage,
    stage_traces: Vec<QuerySubscriptionDiagnosticStageTrace>,
    counter_snapshot: String,
    trace_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionDiagnosticCounters,
}

impl QuerySubscriptionDiagnosticTrace {
    pub fn terminal_stage(&self) -> &QuerySubscriptionDiagnosticStage {
        &self.terminal_stage
    }

    pub fn stage_traces(&self) -> &[QuerySubscriptionDiagnosticStageTrace] {
        &self.stage_traces
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }

    pub fn trace_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trace_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionDiagnosticCounters {
        &self.counters
    }
}

impl QuerySubscriptionDiagnosticTrace {
    pub(super) fn from_parts(
        terminal_stage: QuerySubscriptionDiagnosticStage,
        stage_traces: Vec<QuerySubscriptionDiagnosticStageTrace>,
        counter_snapshot: String,
        trace_identity: WorthQueryEvidenceIdentity,
        counters: super::super::bundle::QuerySubscriptionDiagnosticCounters,
    ) -> Self {
        Self {
            terminal_stage,
            stage_traces,
            counter_snapshot,
            trace_identity,
            counters,
        }
    }
}

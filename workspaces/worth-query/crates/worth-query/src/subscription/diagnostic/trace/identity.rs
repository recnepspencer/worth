use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::evidence_identities::diagnostic_trace_identity;
use super::super::bundle::QuerySubscriptionDiagnosticCounters;
use super::super::stage::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};
use super::vocabulary::{QuerySubscriptionDiagnosticStageTrace, QuerySubscriptionDiagnosticTrace};

pub(super) fn stage_evidence_from_state(
    stage: QuerySubscriptionDiagnosticStage,
    admitted: bool,
    reason: String,
    source_identity: &WorthQueryEvidenceIdentity,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> Result<QuerySubscriptionDiagnosticEvidence, IdentityDerivationDenial> {
    Ok(if admitted {
        QuerySubscriptionDiagnosticEvidence::admitted(
            stage,
            reason,
            source_identity,
            counter_identity,
        )?
    } else {
        QuerySubscriptionDiagnosticEvidence::denied(
            stage,
            reason,
            source_identity,
            counter_identity,
        )?
    })
}

pub(super) fn trace_from_stage_evidence(
    stage_evidence: Vec<QuerySubscriptionDiagnosticEvidence>,
) -> Result<QuerySubscriptionDiagnosticTrace, IdentityDerivationDenial> {
    let terminal_stage = *stage_evidence
        .last()
        .map(|evidence| evidence.stage())
        .expect("diagnostic trace requires at least one stage");
    let stage_traces = stage_evidence
        .iter()
        .map(QuerySubscriptionDiagnosticStageTrace::from_evidence)
        .collect::<Vec<_>>();
    let counters = QuerySubscriptionDiagnosticCounters::trace_emitted(stage_traces.len() as u64);
    let counter_snapshot = counters.counter_projection().label().to_string();
    let stage_trace_refs: Vec<&WorthQueryEvidenceIdentity> = stage_traces
        .iter()
        .map(|trace| trace.stage_trace_identity())
        .collect();
    let trace_identity = diagnostic_trace_identity(
        terminal_stage.as_str(),
        &counters.evidence_identity(),
        stage_trace_refs,
    )?;
    Ok(QuerySubscriptionDiagnosticTrace::from_parts(
        terminal_stage,
        stage_traces,
        counter_snapshot,
        trace_identity,
        counters,
    ))
}

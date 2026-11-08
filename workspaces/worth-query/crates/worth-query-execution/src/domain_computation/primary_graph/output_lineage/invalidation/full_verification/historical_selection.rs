//! Historical selection changes the read posture, never the original evidence graph.
use super::*;
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial as Denial,
    VisibilityProjectionView,
};
impl FullVerificationImage<'_> {
    pub(in crate::domain_computation::primary_graph) fn verify_historical_upstream(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        upstream: &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
        evidence: &RelationalHistoricalSelectionRead,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        reserve_pending(&mut pending, upstream.len())?;
        pending.extend(upstream.iter().map(EvidenceView::from));
        self.verify_native_sources(execution, pending, Some(evidence))
    }
    pub(super) fn historical_source_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EvidenceView<'_>,
        evidence: &RelationalHistoricalSelectionRead,
        current: &VisibilityProjectionView,
    ) -> Result<bool, FullVerificationStop> {
        if !evidence
            .includes_source(execution, self.runtime, source.selected_native_root())
            .map_err(evidence_stop)?
        {
            return Ok(false);
        }
        for fact in source.source_facts().iter() {
            if fact_is_current(execution, fact, self.runtime, self.snapshot)? {
                continue;
            }
            let answer = fact.historical_revision_matches(execution, current, evidence);
            if !answer.map_err(evidence_stop)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
pub(super) fn evidence_stop(denial: Denial) -> FullVerificationStop {
    match denial {
        Denial::Execution(cause) => CompanionPreflightStop::Execution(cause).into(),
        Denial::Request(cause) => CompanionPreflightStop::Execution(cause.into()).into(),
        Denial::ForeignOwner => FullVerificationStop::Foreign,
        Denial::NotGraphSelection { .. } | Denial::SelectedBasisMismatch { .. } => {
            FullVerificationStop::OutputEvidenceUnavailable
        }
    }
}

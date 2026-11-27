//! Historical selection changes the read posture, never the original evidence graph.
use super::*;
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial as Denial,
    VisibilityProjectionView,
};
impl FullVerificationImage<'_> {
    pub fn verify_historical_upstream(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        upstream: &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
        evidence: &RelationalHistoricalSelectionRead,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        reserve_pending(&mut pending, upstream.len())?;
        for consumed in upstream {
            let view = EvidenceView::from(consumed);
            // A marked upstream verifies through its marks, like any marked
            // read: an equal successor the marks link carries the output this
            // row consumed (public #358). Only an unmarked one is compared
            // through this row's selection evidence.
            let marked = !self.native_only
                && (self.state.settlements.contains_key(view.identity())
                    || self.state.equal_links.contains_key(view.identity()));
            if !marked {
                pending.push(view);
            } else if self.verify_source_view(execution, view)? == FullVerificationDecision::Changed
            {
                return Ok(FullVerificationDecision::Changed);
            }
        }
        self.verify_native_sources(execution, pending, Some(evidence))
    }
    pub fn historical_source_is_current(
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
        let Some(facts) = source.source_facts() else {
            return Ok(false);
        };
        for fact in facts.iter() {
            if fact_is_current(execution, fact, self.runtime, self.basis)? {
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
pub fn evidence_stop(denial: Denial) -> FullVerificationStop {
    match denial {
        Denial::Execution(cause) => CompanionPreflightStop::Execution(cause).into(),
        Denial::Request(cause) => CompanionPreflightStop::Execution(cause.into()).into(),
        Denial::ForeignOwner => FullVerificationStop::Foreign,
        Denial::NotGraphSelection { .. } | Denial::SelectedBasisMismatch { .. } => {
            FullVerificationStop::OutputEvidenceUnavailable
        }
    }
}

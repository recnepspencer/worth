//! A transported row's recorded facts, compared through the selection's read evidence.
use super::selection::{read_denial, retained_fact_denial};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial as EvidenceDenial,
    VisibilityProjectionView,
};

/// Every fact holds by the ordinary comparison, or it is a field or aspect
/// revision of a member the committed selection restored whose original
/// revision the evidence proves at the selected step, unchanged since the
/// restoring commit. Any other moved fact ends reuse. O(facts).
pub(in crate::domain_computation::primary_graph) fn restored_facts_hold<
    F: FactSequence + ?Sized,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    current: &VisibilityProjectionView,
    evidence: &RelationalHistoricalSelectionRead,
    facts: Option<&F>,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    let Some(facts) = facts.filter(|facts| !facts.is_empty()) else {
        return Ok(false);
    };
    let mut extent = super::RetainedFactVerification::new(execution, runtime, snapshot, None);
    for fact in facts.iter() {
        let moved = super::first_moved_fact_in(&mut extent, std::iter::once(fact))
            .map_err(|stop| retained_fact_denial(stop, "restored comparison unavailable"))?;
        if moved.is_some()
            && !evidence_answer(fact.historical_revision_matches(execution, current, evidence))?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A stop stays a stop; a foreign or mismatched answer only declines reuse.
pub(in crate::domain_computation::primary_graph) fn evidence_answer(
    answer: Result<bool, EvidenceDenial>,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    match answer {
        Ok(answer) => Ok(answer),
        Err(EvidenceDenial::Execution(cause)) => Err(read_denial(cause)),
        Err(EvidenceDenial::Request(stop)) => Err(read_denial(stop.into())),
        Err(
            EvidenceDenial::ForeignOwner
            | EvidenceDenial::NotGraphSelection { .. }
            | EvidenceDenial::SelectedBasisMismatch { .. },
        ) => Ok(false),
    }
}

pub(in crate::domain_computation::primary_graph) fn evidence_unavailable(
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "restored output read evidence could not project the selected basis",
    )
}

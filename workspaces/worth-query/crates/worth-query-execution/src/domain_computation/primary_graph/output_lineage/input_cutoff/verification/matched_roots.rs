//! Consumed roots that this wave's certified successors already replaced.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::super::super::RecordedSettlementIdentity;
use super::InputCutoffVerificationStop;
use crate::domain_computation::primary_graph::application_contribution::MatchedRequiredPredecessors;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;

/// Whether every matched old identity is one of this record's own consumed
/// roots, joined on this invocation's certified wave.
pub(super) fn consumed_roots(
    roots: &[ConsumedOutputEvidence],
    matched: &MatchedRequiredPredecessors<'_>,
    selected: &PositionedRelationalSnapshot,
) -> Result<bool, InputCutoffVerificationStop> {
    if !matched.matches_cutoff_root(selected)? {
        return Ok(false);
    }

    Ok(matched
        .old_identities()
        .all(|old| roots.iter().any(|root| root.identity() == old)))
}

/// Whether every pending edge names a matched old identity.
pub(super) fn names_every_edge(
    edges: &OrdSet<Arc<RecordedSettlementIdentity>>,
    matched: &MatchedRequiredPredecessors<'_>,
) -> Result<bool, InputCutoffVerificationStop> {
    Ok(!edges.is_empty()
        && edges
            .iter()
            .all(|edge| matched.old_identities().any(|old| old == edge)))
}

/// Replacements carry current evidence issued at this exact wave. Unmatched
/// roots keep their complete evidence; every root is verified by the caller.
pub(super) fn rebound(
    roots: &[ConsumedOutputEvidence],
    matched: &MatchedRequiredPredecessors<'_>,
    selected: &PositionedRelationalSnapshot,
) -> Result<Option<Arc<[ConsumedOutputEvidence]>>, InputCutoffVerificationStop> {
    if !consumed_roots(roots, matched, selected)? {
        return Ok(None);
    }
    for old in matched.old_identities() {
        if matched.replacement(old).is_none() {
            return Ok(None);
        }
    }
    let mut rebound = Vec::new();
    rebound
        .try_reserve_exact(roots.len())
        .map_err(|_| InputCutoffVerificationStop::CapacityExhausted)?;
    for root in roots {
        let replacement = matched.replacement(root.identity());
        rebound.push(replacement.unwrap_or(root).clone());
    }
    Ok(Some(Arc::from(rebound.into_boxed_slice())))
}

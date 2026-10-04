//! Consumed roots that this wave's certified successors already replaced.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use super::super::super::{
    invalidation::{ConsumedOutputCurrentness, InvalidationEditAdmission},
    RecordedSettlementIdentity,
};
use super::InputCutoffVerificationStop;
use crate::domain_computation::primary_graph::{
    application_contribution::MatchedRequiredPredecessors,
    invariant_projection::{ConsumedOutputEvidence, ConsumedOutputVerificationStop},
    SourceInvalidationOwner,
};

/// Whether every matched old identity is one of this record's own consumed
/// roots, joined on this invocation's certified wave.
pub(super) fn consumed_roots(
    roots: &[ConsumedOutputEvidence],
    matched: &MatchedRequiredPredecessors<'_>,
    selected: &PositionedRelationalSnapshot,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    admission.charge_external_work(3)?;
    if !matched.matches_cutoff_root(selected, admission)? {
        return Ok(false);
    }
    let old = matched.old_identities();
    admission.charge_external_work(identity_work(roots.len().saturating_mul(old.len()))?)?;
    Ok(old
        .iter()
        .all(|old| roots.iter().any(|root| root.identity() == old)))
}

/// Whether every pending edge names a matched old identity.
pub(super) fn names_every_edge(
    edges: &OrdSet<Arc<RecordedSettlementIdentity>>,
    matched: &MatchedRequiredPredecessors<'_>,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    let old = matched.old_identities();
    admission.charge_external_work(identity_work(edges.len().saturating_mul(old.len()))?)?;
    Ok(!edges.is_empty() && edges.iter().all(|edge| old.contains(edge)))
}

/// Only the matched roots' own pending equality may be replaced by their
/// already certified current successors. Every other root, and any deeper
/// pending dependency, is verified again and stays deferred while pending.
#[allow(clippy::too_many_arguments)]
pub(super) fn pending_equalities_replaced(
    roots: &[ConsumedOutputEvidence],
    matched: &MatchedRequiredPredecessors<'_>,
    owner: &SourceInvalidationOwner,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    selected: &PositionedRelationalSnapshot,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    let mut replaced = Vec::new();
    replaced
        .try_reserve_exact(matched.old_identities().len())
        .map_err(|_| InputCutoffVerificationStop::WorkExhausted)?;
    for old in matched.old_identities() {
        if matches!(
            owner.consumed_output_currentness(selected, old, admission)?,
            ConsumedOutputCurrentness::PendingEqualSuccessor
        ) {
            replaced.push(Arc::clone(old));
        }
    }
    if replaced.is_empty() {
        return Ok(false);
    }
    match ConsumedOutputEvidence::verify_many_except(
        roots, &replaced, owner, runtime, snapshot, selected, admission,
    ) {
        Ok(_) | Err(ConsumedOutputVerificationStop::Unavailable) => Ok(true),
        Err(ConsumedOutputVerificationStop::PendingUpstream) => Ok(false),
        Err(ConsumedOutputVerificationStop::RetryCurrentness(_)) => {
            Err(InputCutoffVerificationStop::CurrentnessRaced)
        }
        Err(ConsumedOutputVerificationStop::WorkExhausted) => {
            Err(InputCutoffVerificationStop::WorkExhausted)
        }
    }
}

fn identity_work(comparisons: usize) -> Result<u64, CompanionPreflightStop> {
    std::mem::size_of::<RecordedSettlementIdentity>()
        .checked_mul(2)
        .and_then(|work| work.checked_add(2))
        .and_then(|work| work.checked_mul(comparisons.max(1)))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)
}

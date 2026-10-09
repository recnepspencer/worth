//! Consumed roots that this wave's certified successors already replaced.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::PositionedRelationalSnapshot,
};

use super::super::super::{invalidation::InvalidationEditAdmission, RecordedSettlementIdentity};
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
    let old_count = matched.root_count();
    admission.charge_external_work(identity_work(roots.len().saturating_mul(old_count))?)?;
    Ok(matched
        .old_identities()
        .all(|old| roots.iter().any(|root| root.identity() == old)))
}

/// Whether every pending edge names a matched old identity.
pub(super) fn names_every_edge(
    edges: &OrdSet<Arc<RecordedSettlementIdentity>>,
    matched: &MatchedRequiredPredecessors<'_>,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    let old_count = matched.root_count();
    admission.charge_external_work(identity_work(edges.len().saturating_mul(old_count))?)?;
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
    owner: &SourceInvalidationOwner,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<Arc<[ConsumedOutputEvidence]>>, InputCutoffVerificationStop> {
    if !consumed_roots(roots, matched, selected, admission)? {
        return Ok(None);
    }
    for old in matched.old_identities() {
        admission.charge_external_work(identity_work(matched.root_count())?)?;
        if matched.replacement(old).is_none() {
            return Ok(None);
        }
    }
    let initialized = roots
        .len()
        .checked_mul(std::mem::size_of::<ConsumedOutputEvidence>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    let bytes =
        super::super::super::invalidation::arc_slice_bytes::<ConsumedOutputEvidence>(roots.len())
            .and_then(|arc| arc.checked_add(initialized))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.admit_read_scratch(bytes)?;
    admission.charge_external_work(identity_work(
        roots.len().saturating_mul(matched.root_count()),
    )?)?;
    admission.charge_external_work(bytes)?;
    let mut rebound = Vec::new();
    rebound
        .try_reserve_exact(roots.len())
        .map_err(|_| InputCutoffVerificationStop::CapacityExhausted)?;
    for root in roots {
        let replacement = matched.replacement(root.identity());
        rebound.push(replacement.unwrap_or(root).clone());
    }
    match ConsumedOutputEvidence::admit_backing(&mut rebound, owner, admission) {
        Ok(()) => {}
        // Unavailable evidence cannot mint an equality witness. It is Fresh,
        // not an allocation refusal.
        Err(ConsumedOutputVerificationStop::Unavailable) => return Ok(None),
        Err(ConsumedOutputVerificationStop::CapacityExhausted) => {
            return Err(InputCutoffVerificationStop::CapacityExhausted)
        }
        Err(ConsumedOutputVerificationStop::Interrupted(event)) => {
            return Err(InputCutoffVerificationStop::Admission(
                CompanionPreflightStop::Interrupted(event),
            ))
        }
        Err(ConsumedOutputVerificationStop::WorkExhausted) => {
            return Err(InputCutoffVerificationStop::WorkExhausted)
        }
        Err(ConsumedOutputVerificationStop::PendingUpstream) => {
            return Err(InputCutoffVerificationStop::PendingUpstream)
        }
        Err(ConsumedOutputVerificationStop::RetryCurrentness(_)) => {
            return Err(InputCutoffVerificationStop::CurrentnessRaced)
        }
    }
    Ok(Some(Arc::from(rebound.into_boxed_slice())))
}

fn identity_work(comparisons: usize) -> Result<u64, CompanionPreflightStop> {
    std::mem::size_of::<RecordedSettlementIdentity>()
        .checked_mul(2)
        .and_then(|work| work.checked_add(2))
        .and_then(|work| work.checked_mul(comparisons.max(1)))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)
}

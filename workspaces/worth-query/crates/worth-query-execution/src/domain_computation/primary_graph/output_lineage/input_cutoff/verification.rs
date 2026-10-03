//! Exact prior-output checks before a producer may omit its handler.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::{CompanionCellEditStop, CompanionPreflightStop},
    runtime::{PositionedRelationalSnapshot, RelationalRuntime, SnapshotPositionDenial},
    snapshots::SnapshotHandle,
};

use super::super::{
    invalidation::{
        FullVerificationReason, InvalidationEditAdmission, SourceSettlementCurrentness,
    },
    PreparedInputReuseKey,
};
use super::{PreparedInputCutoffBasis, RetainedInputCutoffCandidate};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        PreparedDecisionReuseContext, WorthQueryApplicationObservedFact,
        WorthQuerySourceCurrentnessFailure,
    },
    application_contribution::MatchedRequiredPredecessor,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    SourceInvalidationOwner,
};

/// A prior performed record and the exact native read basis checked for reuse.
/// Only the output-lineage owner can consume this at publication.
pub(in crate::domain_computation::primary_graph) struct VerifiedInputCutoff<'selected> {
    pub(super) candidate: RetainedInputCutoffCandidate,
    pub(super) selected: &'selected PositionedRelationalSnapshot,
    pub(super) fresh_key: Option<PreparedInputReuseKey>,
    pub(super) fresh_context: PreparedDecisionReuseContext,
    /// The same alias-local facts whose completed prefix passed verification.
    pub(super) verified_facts: Arc<[WorthQueryApplicationObservedFact]>,
}

pub(in crate::domain_computation::primary_graph) enum InputCutoffDecision<'selected> {
    Fresh {
        key: PreparedInputReuseKey,
        context: PreparedDecisionReuseContext,
    },
    Reuse(VerifiedInputCutoff<'selected>),
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum InputCutoffVerificationStop {
    Admission(CompanionPreflightStop),
    WorkExhausted,
    PendingUpstream,
    CurrentnessRaced(CompanionCellEditStop),
    SelectedSourceUnavailable(SnapshotPositionDenial),
    SelectedSourceMismatch,
}

impl From<CompanionPreflightStop> for InputCutoffVerificationStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}

impl RetainedInputCutoffCandidate {
    /// A fresh selected input replaces the obsolete source-query suffix. The
    /// performed handler prefix and actually consumed upstream outputs still
    /// require proof at this exact native root. The candidate's own dirty marks remain intact.
    pub(in crate::domain_computation::primary_graph) fn verify_for_reuse<'selected>(
        self,
        fresh_key: PreparedInputReuseKey,
        fresh_context: PreparedDecisionReuseContext,
        matched_predecessor: Option<MatchedRequiredPredecessor<'_>>,
        runtime: &RelationalRuntime,
        basis: &'selected PreparedInputCutoffBasis<'_>,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
        currentness: &mut InvalidationEditAdmission,
    ) -> Result<InputCutoffDecision<'selected>, InputCutoffVerificationStop> {
        let (snapshot, selected) = basis.in_runtime(runtime, admission)?;
        let verified_facts = self.eligible_for_reuse(
            &fresh_key,
            &fresh_context,
            matched_predecessor,
            runtime,
            snapshot,
            selected,
            owner,
            admission,
            currentness,
        )?;
        Ok(if let Some(verified_facts) = verified_facts {
            InputCutoffDecision::Reuse(VerifiedInputCutoff {
                candidate: self,
                selected,
                fresh_key: Some(fresh_key),
                fresh_context,
                verified_facts,
            })
        } else {
            InputCutoffDecision::Fresh {
                key: fresh_key,
                context: fresh_context,
            }
        })
    }

    fn eligible_for_reuse(
        &self,
        fresh_key: &PreparedInputReuseKey,
        fresh_context: &PreparedDecisionReuseContext,
        matched_predecessor: Option<MatchedRequiredPredecessor<'_>>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
        currentness: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<[WorthQueryApplicationObservedFact]>>, InputCutoffVerificationStop> {
        admission.charge_external_work(1)?;
        let Some(prior_key) = self.prepared_input_key() else {
            return Ok(None);
        };
        let Some(prior_context) = self.completed_decision_reuse() else {
            return Ok(None);
        };
        let Some(count) = self.completed_handler_fact_count() else {
            return Ok(None);
        };
        let Some(witness) = self.native_output_witness() else {
            return Ok(None);
        };
        if !prior_key.same_prepared_input_as(fresh_key, |visits| {
            let visits =
                u64::try_from(visits).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
            admission.charge_external_work(visits)
        })? || !fresh_context.matches_completed(prior_context, |visits| {
            admission.charge_external_work(visits)
        })? {
            return Ok(None);
        }
        let requirement = self.verification_requirement(admission)?;
        if requirement == Some(FullVerificationReason::CheckpointRestore) {
            return Ok(None);
        }
        let Some(facts) = self.observed_source_facts(admission)? else {
            return Ok(None);
        };
        if count > facts.len() {
            return Ok(None);
        }
        let settlement = owner.currentness(selected, self.settlement_identity(), admission)?;
        let mut verify_full_prefix = requirement.is_some();
        let mut dirty_prefix = None;
        match settlement {
            SourceSettlementCurrentness::Clean => {}
            SourceSettlementCurrentness::Dirty(ordinals) => dirty_prefix = Some(ordinals),
            SourceSettlementCurrentness::PendingUpstream(edges) => {
                if let Some(matched) = matched_predecessor.as_ref() {
                    if matched_consumed_root(self.consumed_outputs(), matched, selected, admission)?
                    {
                        admission.charge_external_work(3)?;
                        if edges.len() == 1 {
                            let identity_work = std::mem::size_of::<
                                crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
                            >()
                            .checked_mul(2)
                            .and_then(|work| work.checked_add(2))
                            .and_then(|work| u64::try_from(work).ok())
                            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
                            admission.charge_external_work(identity_work)?;
                            if edges
                                .iter()
                                .next()
                                .is_some_and(|edge| edge.as_ref() == matched.old_identity())
                            {
                                // The certified successor replaced this one old consumed
                                // output. Its pending mark cannot authorize reuse, but the
                                // retained key and completed context may enter Fresh.
                                return Ok(None);
                            }
                        }
                    }
                }
                return Err(InputCutoffVerificationStop::PendingUpstream);
            }
            SourceSettlementCurrentness::FullVerificationRequired(reason) => {
                if matches!(
                    reason,
                    FullVerificationReason::CheckpointRestore
                        | FullVerificationReason::MissingSettlement
                        | FullVerificationReason::ForeignSource
                        | FullVerificationReason::DifferentBranch
                        | FullVerificationReason::BeforeReadBasis
                ) {
                    return Ok(None);
                }
                verify_full_prefix = true;
            }
        }
        let current = if verify_full_prefix {
            marked_facts_permit_reuse(facts.iter().take(count), runtime, snapshot, currentness)?
        } else if let Some(ordinals) = dirty_prefix {
            marked_facts_permit_reuse(
                ordinals
                    .iter()
                    .take_while(|ordinal| **ordinal < count)
                    .map(|ordinal| &facts[*ordinal]),
                runtime,
                snapshot,
                currentness,
            )?
        } else {
            true
        };
        if !current {
            return Ok(None);
        }
        match ConsumedOutputEvidence::verify_many_with_admission(
            self.consumed_outputs(),
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        ) {
            Ok(ConsumedOutputVerification::Current) => {}
            Ok(
                ConsumedOutputVerification::ChangedDirectFact(_)
                | ConsumedOutputVerification::ChangedUpstream,
            )
            | Err(ConsumedOutputVerificationStop::Unavailable) => {
                return Ok(None);
            }
            Err(ConsumedOutputVerificationStop::PendingUpstream) => {
                if let Some(matched) = matched_predecessor.as_ref() {
                    if matched_consumed_root(self.consumed_outputs(), matched, selected, admission)?
                    {
                        use super::super::invalidation::ConsumedOutputCurrentness;
                        if matches!(
                            owner.consumed_output_currentness(
                                selected,
                                matched.old_identity(),
                                admission,
                            )?,
                            ConsumedOutputCurrentness::PendingEqualSuccessor
                        ) {
                            // Only the exact root's pending equality may be replaced by
                            // its already certified current successor. A deeper pending
                            // dependency must remain deferred.
                            return Ok(None);
                        }
                    }
                }
                return Err(InputCutoffVerificationStop::PendingUpstream);
            }
            Err(ConsumedOutputVerificationStop::RetryCurrentness(stop)) => {
                return Err(InputCutoffVerificationStop::CurrentnessRaced(stop));
            }
            Err(ConsumedOutputVerificationStop::WorkExhausted) => {
                return Err(InputCutoffVerificationStop::WorkExhausted);
            }
        }
        if !witness.unchanged_in(runtime, snapshot, admission)? {
            return Ok(None);
        }
        Ok(Some(facts))
    }
}

fn matched_consumed_root(
    roots: &[ConsumedOutputEvidence],
    matched: &MatchedRequiredPredecessor<'_>,
    selected: &PositionedRelationalSnapshot,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    admission.charge_external_work(3)?;
    if roots.len() != 1 || !matched.matches_cutoff_root(selected, admission)? {
        return Ok(false);
    }
    let identity_work = std::mem::size_of::<
        crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    >()
    .checked_mul(2)
    .and_then(|work| work.checked_add(2))
    .and_then(|work| u64::try_from(work).ok())
    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    admission.charge_external_work(identity_work)?;
    Ok(roots[0].identity().as_ref() == matched.old_identity())
}

/// Whether the marked facts, or the full-verification prefix, still permit
/// reuse. An exhausted currentness allowance proves nothing about the prior
/// output, so it answers `false` and the demand executes fresh instead of
/// stopping; every other stop stays a stop.
fn marked_facts_permit_reuse<'fact>(
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    currentness: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    match facts_are_current(facts, runtime, snapshot, currentness) {
        Ok(current) => Ok(current),
        Err(
            InputCutoffVerificationStop::WorkExhausted
            | InputCutoffVerificationStop::Admission(CompanionPreflightStop::WorkExhausted {
                ..
            }),
        ) => Ok(false),
        Err(stop) => Err(stop),
    }
}

/// Re-verifies marked facts, or the full-verification prefix, on the
/// host-bounded source-currentness allowance.
fn facts_are_current<'fact>(
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    currentness: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    for fact in facts {
        currentness.charge_external_work(1)?;
        if !fact_is_current(fact, runtime, snapshot, currentness)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn fact_is_current(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, InputCutoffVerificationStop> {
    let remaining = admission.remaining_work();
    if remaining == 0 {
        return Err(InputCutoffVerificationStop::WorkExhausted);
    }
    let prepaid = fact
        .exact_probe_work()
        .map_err(|_| InputCutoffVerificationStop::WorkExhausted)?
        .unwrap_or(0);
    admission.charge_external_work(prepaid as u64)?;
    let (current, work) = match fact.source_currentness_in(runtime, snapshot, remaining) {
        Ok(value) => value,
        Err(WorthQuerySourceCurrentnessFailure::Unavailable) => return Ok(false),
        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
            return Err(InputCutoffVerificationStop::WorkExhausted);
        }
    };
    admission.charge_external_work(work.saturating_sub(prepaid) as u64)?;
    Ok(current)
}

#[cfg(test)]
mod tests;

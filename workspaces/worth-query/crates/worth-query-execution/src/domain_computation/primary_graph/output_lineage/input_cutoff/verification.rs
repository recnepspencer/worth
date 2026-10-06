//! Exact prior-output checks before a producer may omit its handler.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
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
        Movement, PreparedDecisionReuseContext, WorthQueryApplicationObservedFact,
        WorthQuerySourceCurrentnessFailure,
    },
    application_contribution::MatchedRequiredPredecessors,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    SourceInvalidationOwner,
};

/// What the cutoff does not verify: a row still in its checkpoint posture,
/// and a settlement no mark row answers for under the selected source.
/// Selection asks this before it calls a live output exact, so the two
/// cannot disagree about one row.
pub(in crate::domain_computation::primary_graph) fn cutoff_declines(
    requirement: Option<FullVerificationReason>,
    settlement: Option<&SourceSettlementCurrentness>,
) -> bool {
    requirement == Some(FullVerificationReason::CheckpointRestore)
        || settlement.is_some_and(SourceSettlementCurrentness::no_row_answers)
}

/// A prior performed record and the exact native read basis checked for reuse.
/// Only the output-lineage owner can consume this at publication.
pub(in crate::domain_computation::primary_graph) struct VerifiedInputCutoff<'selected> {
    pub(super) candidate: RetainedInputCutoffCandidate,
    pub(super) selected: &'selected PositionedRelationalSnapshot,
    pub(super) fresh_key: Option<PreparedInputReuseKey>,
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
    CurrentnessRaced,
    SelectedSourceUnavailable,
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
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
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
            matched_predecessors,
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
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
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
        if cutoff_declines(requirement, None) {
            return Ok(None);
        }
        let Some(facts) = self.observed_source_facts(admission)? else {
            return Ok(None);
        };
        if count > facts.len() {
            return Ok(None);
        }
        let settlement = owner.currentness(selected, self.settlement_identity(), admission)?;
        if cutoff_declines(requirement, Some(&settlement)) {
            return Ok(None);
        }
        let mut verify_full_prefix = requirement.is_some();
        let mut dirty_prefix = None;
        match settlement {
            SourceSettlementCurrentness::Clean => {}
            SourceSettlementCurrentness::Dirty(ordinals) => dirty_prefix = Some(ordinals),
            SourceSettlementCurrentness::PendingUpstream(edges) => {
                if let Some(matched) = &matched_predecessors {
                    if matched_roots::consumed_roots(
                        self.consumed_outputs(),
                        matched,
                        selected,
                        admission,
                    )? && matched_roots::names_every_edge(&edges, matched, admission)?
                    {
                        // Certified successors replaced exactly these old consumed
                        // outputs. Their pending marks cannot authorize reuse, but
                        // the retained key and completed context may enter Fresh.
                        return Ok(None);
                    }
                }
                return Err(InputCutoffVerificationStop::PendingUpstream);
            }
            SourceSettlementCurrentness::FullVerificationRequired(_) => verify_full_prefix = true,
            // `cutoff_declines` declined it above.
            SourceSettlementCurrentness::Foreign => return Ok(None),
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
                if let Some(matched) = &matched_predecessors {
                    if matched_roots::consumed_roots(
                        self.consumed_outputs(),
                        matched,
                        selected,
                        admission,
                    )? && matched_roots::pending_equalities_replaced(
                        self.consumed_outputs(),
                        matched,
                        owner,
                        runtime,
                        snapshot,
                        selected,
                        admission,
                    )? {
                        return Ok(None);
                    }
                }
                return Err(InputCutoffVerificationStop::PendingUpstream);
            }
            Err(ConsumedOutputVerificationStop::RetryCurrentness(_)) => {
                return Err(InputCutoffVerificationStop::CurrentnessRaced);
            }
            Err(ConsumedOutputVerificationStop::WorkExhausted) => {
                return Err(InputCutoffVerificationStop::WorkExhausted);
            }
            Err(ConsumedOutputVerificationStop::Interrupted(event)) => {
                return Err(InputCutoffVerificationStop::Admission(
                    CompanionPreflightStop::Interrupted(event),
                ));
            }
        }
        if !witness.unchanged_in(runtime, snapshot, admission)? {
            return Ok(None);
        }
        Ok(Some(facts))
    }
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
    match fact.source_currentness_in(runtime, snapshot, admission)? {
        Ok(movement) => Ok(movement.movement() == Movement::Unmoved),
        Err(WorthQuerySourceCurrentnessFailure::Unavailable) => Ok(false),
        Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
            Err(InputCutoffVerificationStop::WorkExhausted)
        }
    }
}

mod matched_roots;
#[cfg(test)]
mod tests;

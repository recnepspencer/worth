//! What a family selection selects from its candidates' verdicts (public #429).
//!
//! The `current_output` read and the re-resolution of its `FamilySelection`
//! fact both classify each candidate with [`candidate_verdict`] and select
//! with [`select_family`] and [`select_current`], so the two cannot disagree
//! on what the family selects at one basis.

use std::sync::{Arc, OnceLock};

use super::super::invalidation::FullVerificationReason;
use super::super::{ComparableSourceFacts, SealedNativeOutputWitness, WorthQueryCurrentOutputCandidate};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop,
};
use worth_relational::facade::identity::EntityId;

/// Why a family selects nothing a reader may use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionRefusal {
    FamilyUnavailable,
    OutputUnavailable,
    StaleSource,
}

/// One candidate's verdict at the selecting basis.
pub enum CandidateVerdict {
    /// Its sealed witness compared current.
    Current { facts: ComparableSourceFacts, witness: Arc<OnceLock<SealedNativeOutputWitness>> },
    /// An unwitnessed restore root compared current; the witness that
    /// comparison built is the read's to retain.
    RestoredCurrent { facts: ComparableSourceFacts, witness: SealedNativeOutputWitness },
    /// It has no proof a selection can compare.
    Unproven,
    /// It changed.
    Stale,
    /// Its producer's own source entity changed.
    Obsolete,
}

impl CandidateVerdict {
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current { .. } | Self::RestoredCurrent { .. })
    }
}

/// Verify one candidate. `sealed` compares a witnessed candidate and
/// `restored` an unwitnessed restore root; each caller passes its own
/// comparison, and nothing else differs between them.
pub fn candidate_verdict(
    candidate: &WorthQueryCurrentOutputCandidate,
    producer: EntityId,
    sealed: impl FnOnce(
        &ComparableSourceFacts,
        &Arc<OnceLock<SealedNativeOutputWitness>>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
    restored: impl FnOnce(
        &ComparableSourceFacts,
    ) -> Result<Option<SealedNativeOutputWitness>, ConsumedOutputVerificationStop>,
) -> Result<CandidateVerdict, ConsumedOutputVerificationStop> {
    let Some(facts) = candidate.observed_source_facts.for_comparison() else {
        return Ok(CandidateVerdict::Stale);
    };
    let witness = candidate
        .native_output_witness
        .as_ref()
        .filter(|witness| witness.get().is_some());
    let restored_root = candidate.consumed_outputs.is_empty()
        && matches!(
            candidate.verification_requirement,
            Some(FullVerificationReason::CheckpointRestore)
        );
    match witness {
        Some(witness) => Ok(match sealed(&facts, witness)? {
            ConsumedOutputVerification::Current => {
                CandidateVerdict::Current { facts, witness: Arc::clone(witness) }
            }
            ConsumedOutputVerification::ChangedDirectFact(ordinal)
                if matches!(
                    facts.get(ordinal),
                    Some(WorthQueryApplicationObservedFact::SourceEntity { entity_id })
                        if *entity_id == producer
                ) =>
            {
                CandidateVerdict::Obsolete
            }
            _ => CandidateVerdict::Stale,
        }),
        None if restored_root => Ok(match restored(&facts)? {
            Some(witness) => CandidateVerdict::RestoredCurrent { facts, witness },
            None => CandidateVerdict::Unproven,
        }),
        None => Ok(CandidateVerdict::Unproven),
    }
}

/// The refusal a family gives before any candidate is verified.
pub fn select_family(installed: bool, ambiguous_publication: bool) -> Result<(), SelectionRefusal> {
    if !installed {
        return Err(SelectionRefusal::FamilyUnavailable);
    }
    if ambiguous_publication {
        return Err(SelectionRefusal::OutputUnavailable);
    }
    Ok(())
}

/// The current candidates, or the refusal their verdicts give. A current
/// member the applied overlay supplied makes stale committed heads tolerable.
pub fn select_current<T>(
    verdicts: impl IntoIterator<Item = (T, bool, bool)>,
    applied: bool,
) -> Result<Vec<T>, SelectionRefusal> {
    let mut current = Vec::new();
    let (mut stale, mut obsolete) = (false, false);
    for (candidate, is_current, is_obsolete) in verdicts {
        if is_current {
            current.push(candidate);
        } else if is_obsolete {
            obsolete = true;
        } else {
            stale = true;
        }
    }
    if current.is_empty() && obsolete {
        return Err(SelectionRefusal::OutputUnavailable);
    }
    if current.is_empty() && stale && !applied {
        return Err(SelectionRefusal::StaleSource);
    }
    Ok(current)
}

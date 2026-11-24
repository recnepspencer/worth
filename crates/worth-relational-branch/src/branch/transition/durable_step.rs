use std::sync::Arc;
use worth_proof::DurablyCommitted;

use super::PositionedBranchTransition;
use super::RelationalDurableTransitionPublication;
use crate::durability::medium::publication_refusal::RelationalDurablePublicationRefusal;

pub mod sealed {
    pub trait Step {}
}

/// A transition's subject stays bound to its distinct publication authority
/// until the stream applies it. Replay is not a new medium publication.
pub struct RelationalTransitionPublication {
    admission: TransitionAdmission,
}

enum TransitionAdmission {
    MemoryOnly(PositionedBranchTransition),
    Replayed(PositionedBranchTransition),
    Durable(DurablyCommitted<RelationalDurableTransitionPublication>),
}

impl RelationalTransitionPublication {
    pub fn memory(positioned: PositionedBranchTransition) -> Self {
        Self {
            admission: TransitionAdmission::MemoryOnly(positioned),
        }
    }

    pub fn durable(committed: DurablyCommitted<RelationalDurableTransitionPublication>) -> Self {
        Self {
            admission: TransitionAdmission::Durable(committed),
        }
    }

    pub fn cutover<Output>(
        self,
        retain: impl FnOnce(PositionedBranchTransition) -> Arc<PositionedBranchTransition>,
        apply: impl FnOnce(
            Arc<PositionedBranchTransition>,
            Option<DurablyCommitted<super::RelationalCommittedTransitionEvidence>>,
        ) -> Output,
    ) -> Output {
        match self.admission {
            TransitionAdmission::MemoryOnly(positioned)
            | TransitionAdmission::Replayed(positioned) => apply(retain(positioned), None),
            TransitionAdmission::Durable(committed) => committed.consume(|publication| {
                let evidence = super::RelationalCommittedTransitionEvidence::from_publication(
                    publication,
                    retain,
                );
                let positioned = evidence.retained_positioned();
                apply(positioned, Some(DurablyCommitted::from_owner(evidence)))
            }),
        }
    }
}

/// What makes a positioned branch transition durable before its cutover.
///
/// The fork, archive and delete operations run once with either step: a live
/// transition publishes, a replayed one proves it is the transition durable
/// history recorded. Everything else, from preparation to cutover, is the
/// same function in both.
pub trait RelationalBranchTransitionDurableStep: sealed::Step {
    /// Why the step did not admit the cutover.
    type Stop;

    fn caller_association(&self) -> Option<&crate::history::data::RelationalCallerAssociation>;

    /// Make `positioned` durable, or say why its cutover must not run.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop>;
}

/// The step of a transition recovery replays. It is already durable; the
/// runtime must perform exactly the transition durable history recorded, so
/// a replay that would diverge installs nothing.
pub struct ReplayedBranchTransitionStep<'recorded> {
    recorded: &'recorded PositionedBranchTransition,
    fidelity: ReplayFidelity,
}

/// How much of the recorded transition the replay must reproduce.
#[derive(Clone, Copy)]
enum ReplayFidelity {
    /// The transition, its sequence and its followed position: a replay of
    /// the contiguous durable tail.
    Positioned,
    /// The transition alone: a replay of a selection from a longer history,
    /// whose sequence and position the partial history assigns.
    TransitionOnly,
}

impl<'recorded> ReplayedBranchTransitionStep<'recorded> {
    /// Reproduce `recorded` exactly, sequence and followed position included.
    pub fn positioned(recorded: &'recorded PositionedBranchTransition) -> Self {
        Self {
            recorded,
            fidelity: ReplayFidelity::Positioned,
        }
    }

    /// Reproduce `recorded`'s transition, wherever the partial history
    /// places it.
    pub fn transition_only(recorded: &'recorded PositionedBranchTransition) -> Self {
        Self {
            recorded,
            fidelity: ReplayFidelity::TransitionOnly,
        }
    }
}

impl sealed::Step for ReplayedBranchTransitionStep<'_> {}

/// The transition a replay would have performed instead of the recorded one.
#[derive(Debug, Clone)]
pub struct ReplayedBranchTransitionDivergence {
    pub performed: Box<PositionedBranchTransition>,
}

impl RelationalBranchTransitionDurableStep for ReplayedBranchTransitionStep<'_> {
    type Stop = ReplayedBranchTransitionDivergence;

    fn caller_association(&self) -> Option<&crate::history::data::RelationalCallerAssociation> {
        self.recorded.caller_association()
    }

    /// Cost: one structural comparison of two small records.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop> {
        let reproduced = match self.fidelity {
            ReplayFidelity::Positioned => self.recorded == &positioned,
            ReplayFidelity::TransitionOnly => self.recorded.transition() == positioned.transition(),
        };
        if reproduced {
            return Ok(RelationalTransitionPublication {
                admission: TransitionAdmission::Replayed(
                    positioned
                        .with_recorded_artifact_digest(self.recorded.recorded_artifact_digest()),
                ),
            });
        }
        Err(ReplayedBranchTransitionDivergence {
            performed: Box::new(positioned),
        })
    }
}

/// A lifecycle operation's denial type: it can name the pre-effect
/// refusals every durable transition shares.
pub trait RelationalBranchTransitionDenial {
    /// The medium proved the transition had no durable effect.
    fn durable_publication_refused(refusal: RelationalDurablePublicationRefusal) -> Self;
    /// No transition sequence remains.
    fn transition_sequence_exhausted() -> Self;
    /// The concrete retained transition backing exceeds the storage layout.
    fn transition_storage_exhausted() -> Self;
    /// A restored transition differs from this owner's original sequence.
    fn transition_original_prefix_mismatch() -> Self;
}

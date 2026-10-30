use std::sync::Arc;
use worth_proof::DurablyCommitted;

use super::PositionedBranchTransition;
use crate::durability::medium::{
    RelationalDurableMediumBinding, RelationalDurablePublicationRefusal, RelationalDurableStepStop,
    RelationalDurableTransitionPublication,
};

/// A transition's subject stays bound to its distinct publication authority
/// until the stream applies it. Replay is not a new medium publication.
pub(crate) enum RelationalTransitionPublication {
    MemoryOnly(PositionedBranchTransition),
    Replayed(PositionedBranchTransition),
    Durable(DurablyCommitted<RelationalDurableTransitionPublication>),
}

impl RelationalTransitionPublication {
    pub(crate) fn cutover<Output>(
        self,
        apply: impl FnOnce(PositionedBranchTransition) -> Output,
    ) -> Output {
        match self {
            Self::MemoryOnly(positioned) | Self::Replayed(positioned) => apply(positioned),
            Self::Durable(committed) => {
                committed.consume(|publication| apply(publication.into_positioned()))
            }
        }
    }
}

/// What makes a positioned branch transition durable before its cutover.
///
/// The fork, archive and delete operations run once with either step: a live
/// transition publishes, a replayed one proves it is the transition durable
/// history recorded. Everything else, from preparation to cutover, is the
/// same function in both.
pub(crate) trait RelationalBranchTransitionDurableStep {
    /// Why the step did not admit the cutover.
    type Stop;

    /// Make `positioned` durable, or say why its cutover must not run.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop>;
}

/// The step of a live transition: publish to the runtime's medium when it
/// owns one. A memory-only runtime records the transition in its own stream
/// only.
pub(crate) struct LiveBranchTransitionStep<'request, 'authority> {
    medium: Option<Arc<RelationalDurableMediumBinding>>,
    policy: worth_execution::ExecutionAllocationPolicy<'request, 'authority>,
}

impl<'request, 'authority> LiveBranchTransitionStep<'request, 'authority> {
    pub(crate) fn new(
        medium: Option<Arc<RelationalDurableMediumBinding>>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
    ) -> Self {
        Self {
            medium,
            policy: execution.allocation_policy(),
        }
    }
}

impl RelationalBranchTransitionDurableStep for LiveBranchTransitionStep<'_, '_> {
    type Stop = RelationalDurableStepStop;

    /// Cost: one artifact encode and one medium publication with a medium.
    fn take(
        &self,
        positioned: PositionedBranchTransition,
    ) -> Result<RelationalTransitionPublication, Self::Stop> {
        match &self.medium {
            Some(medium) => medium
                .publish_transition(positioned, self.policy)
                .map(RelationalTransitionPublication::Durable),
            None => Ok(RelationalTransitionPublication::MemoryOnly(positioned)),
        }
    }
}

/// The step of a transition recovery replays. It is already durable; the
/// runtime must perform exactly the transition durable history recorded, so
/// a replay that would diverge installs nothing.
pub(crate) struct ReplayedBranchTransitionStep<'recorded> {
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
    pub(crate) fn positioned(recorded: &'recorded PositionedBranchTransition) -> Self {
        Self {
            recorded,
            fidelity: ReplayFidelity::Positioned,
        }
    }

    /// Reproduce `recorded`'s transition, wherever the partial history
    /// places it.
    pub(crate) fn transition_only(recorded: &'recorded PositionedBranchTransition) -> Self {
        Self {
            recorded,
            fidelity: ReplayFidelity::TransitionOnly,
        }
    }
}

/// The transition a replay would have performed instead of the recorded one.
#[derive(Debug, Clone)]
pub(crate) struct ReplayedBranchTransitionDivergence {
    pub(crate) performed: Box<PositionedBranchTransition>,
}

impl RelationalBranchTransitionDurableStep for ReplayedBranchTransitionStep<'_> {
    type Stop = ReplayedBranchTransitionDivergence;

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
            return Ok(RelationalTransitionPublication::Replayed(positioned));
        }
        Err(ReplayedBranchTransitionDivergence {
            performed: Box::new(positioned),
        })
    }
}

/// A lifecycle operation's denial type: it can name the two pre-effect
/// refusals every durable transition shares.
pub(crate) trait RelationalBranchTransitionDenial {
    /// The medium proved the transition had no durable effect.
    fn durable_publication_refused(refusal: RelationalDurablePublicationRefusal) -> Self;
    /// No transition sequence remains.
    fn transition_sequence_exhausted() -> Self;
}

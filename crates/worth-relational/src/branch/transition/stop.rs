use crate::durability::medium::RelationalDurabilityUnresolved;

/// Why a fork, archive or delete did not take effect.
///
/// Every variant leaves no live effect: nothing was installed, no branch
/// changed posture and no name was retired. Only `DurabilityUnresolved`
/// leaves a durable effect possible; its fate is decided by recovery, and
/// until the runtime is reopened every later commit and transition refuses
/// with the same answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchTransitionStop<D> {
    /// The transition was refused before any effect, including
    /// `DurablePublicationRefused` when the medium proved it had none.
    Denied(D),
    /// The caller's request was cancelled or its deadline passed while the
    /// transition waited for the patch-position reservation, before any
    /// effect.
    RequestStopped(worth_execution::WorkCeilingDenial),
    /// The medium could not tell whether the transition became durable.
    DurabilityUnresolved(RelationalDurabilityUnresolved),
}

impl<D> RelationalBranchTransitionStop<D> {
    /// The denial, when the transition was refused before any effect.
    pub fn denied(&self) -> Option<&D> {
        match self {
            Self::Denied(denial) => Some(denial),
            Self::RequestStopped(_) | Self::DurabilityUnresolved(_) => None,
        }
    }

    /// The unresolved durable fate, when the medium could not tell.
    pub fn durability_unresolved(&self) -> Option<&RelationalDurabilityUnresolved> {
        match self {
            Self::DurabilityUnresolved(unresolved) => Some(unresolved),
            Self::Denied(_) | Self::RequestStopped(_) => None,
        }
    }
}

impl<D: super::RelationalBranchTransitionDenial>
    From<
        crate::runtime::BranchTransitionRecordError<
            D,
            crate::durability::medium::RelationalDurableStepStop,
        >,
    > for RelationalBranchTransitionStop<D>
{
    /// How a live transition that did not take effect answers its caller.
    fn from(
        error: crate::runtime::BranchTransitionRecordError<
            D,
            crate::durability::medium::RelationalDurableStepStop,
        >,
    ) -> Self {
        use crate::durability::medium::RelationalDurableStepStop as Step;
        use crate::runtime::BranchTransitionRecordError as Record;
        match error {
            Record::Denied(denial) => Self::Denied(denial),
            Record::ReservationWaitStopped(stop) => Self::RequestStopped(stop),
            Record::SequenceCapacityExhausted => Self::Denied(D::transition_sequence_exhausted()),
            Record::Step(Step::Refused(refusal)) => {
                Self::Denied(D::durable_publication_refused(refusal))
            }
            Record::Step(Step::Unresolved(unresolved)) => Self::DurabilityUnresolved(unresolved),
        }
    }
}

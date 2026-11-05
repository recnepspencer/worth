//! Live writer phase moves with the actual durable medium, never with a caller.
use super::*;
use worth_proof::{HomeProgramTransitionBindingCommitted, HomeProgramTransitionRequired};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalHomeProgramWriteDenial {
    TransitionRequired,
    TransitionAlreadyBound,
    TransitionNotRequested,
    MediumMoved,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
}

#[derive(Debug)]
pub(super) enum RelationalHomeProgramPhase {
    Unrestricted,
    Pending(HomeProgramTransitionRequired),
    Bound {
        required: HomeProgramTransitionRequired,
        completed: HomeProgramTransitionBindingCommitted,
    },
    Refused(worth_proof::HomeProgramTransitionAdmissionDenial),
}

/// Private owning transfer. Recovery moves the original phase with the Box.
pub(crate) struct RelationalDurableMediumTransfer {
    pub(super) medium: Box<dyn RelationalDurableMedium>,
    pub(super) phase: RelationalHomeProgramPhase,
}
impl RelationalDurableMediumTransfer {
    pub(crate) fn fresh(medium: Box<dyn RelationalDurableMedium>) -> Self {
        let phase = match medium.home_transition_required().cloned() {
            None => RelationalHomeProgramPhase::Unrestricted,
            Some(required) => match medium.home_transition_authority() {
                Some(authority) => match authority.admit_required(&required) {
                    Ok(()) => RelationalHomeProgramPhase::Pending(required),
                    Err(cause) => RelationalHomeProgramPhase::Refused(cause),
                },
                None => RelationalHomeProgramPhase::Refused(
                    worth_proof::HomeProgramTransitionAdmissionDenial::ForeignOwner,
                ),
            },
        };
        Self { medium, phase }
    }
}

impl RelationalDurableMediumBinding {
    pub(crate) fn admit_home_program_write(
        &self,
        step: Option<&crate::runtime::AdmittedRelationalHomeProgramStep>,
        codec: Option<&crate::durability::RecoveredCodecPublicationCustody>,
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let state = self.state();
        if state.medium.is_none() {
            return Err(Denial::MediumMoved);
        }
        match state
            .home_phase
            .as_ref()
            .expect("a live medium retains its phase")
        {
            RelationalHomeProgramPhase::Unrestricted => match step {
                None => Ok(()),
                Some(_) => Err(Denial::TransitionNotRequested),
            },
            RelationalHomeProgramPhase::Pending(_) => match (step, codec) {
                (Some(step), None) => self
                    .transition
                    .as_ref()
                    .ok_or(Denial::TransitionNotRequested)?
                    .admit_step(&step.admission, &step.commitment, &step.association)
                    .map_err(Denial::Admission),
                (None, Some(_)) => Ok(()),
                (None, None) | (Some(_), Some(_)) => Err(Denial::TransitionRequired),
            },
            RelationalHomeProgramPhase::Bound { .. } => match step {
                None => Ok(()),
                Some(_) => Err(Denial::TransitionAlreadyBound),
            },
            RelationalHomeProgramPhase::Refused(cause) => Err(Denial::Admission(*cause)),
        }
    }

    pub(crate) fn complete_home_program_transition(
        &self,
        completed: HomeProgramTransitionBindingCommitted,
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let mut state = self.state();
        if state.medium.is_none() {
            return Err(Denial::MediumMoved);
        }
        let authority = self
            .transition
            .as_ref()
            .ok_or(Denial::TransitionNotRequested)?;
        match state
            .home_phase
            .as_ref()
            .expect("a live medium retains its phase")
        {
            RelationalHomeProgramPhase::Pending(required) => {
                authority
                    .admit_binding_committed(required, &completed)
                    .map_err(Denial::Admission)?;
                state.home_phase = Some(RelationalHomeProgramPhase::Bound {
                    required: required.clone(),
                    completed,
                });
                Ok(())
            }
            RelationalHomeProgramPhase::Bound {
                required,
                completed: original,
            } => {
                authority
                    .admit_binding_committed(required, &completed)
                    .map_err(Denial::Admission)?;
                if original.plan_commitment() != completed.plan_commitment() {
                    return Err(Denial::Admission(
                        worth_proof::HomeProgramTransitionAdmissionDenial::CommitmentMismatch,
                    ));
                }
                Ok(())
            }
            RelationalHomeProgramPhase::Unrestricted => Err(Denial::TransitionNotRequested),
            RelationalHomeProgramPhase::Refused(cause) => Err(Denial::Admission(*cause)),
        }
    }
}

//! Live writer phase moves with the actual durable medium, never with a caller.
use super::*;
use worth_proof::{HomeProgramTransitionBindingCommitted, HomeProgramTransitionRequired};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalHomeProgramWriteDenial {
    TransitionRequired,
    BodyMismatch,
    BodyCapacityOverflow,
    BodyEncoding,
    UnsupportedBody,
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
    pub(crate) fn admit_home_program_planning(
        &self,
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let phase = self.home_phase();
        match phase.as_ref().ok_or(Denial::MediumMoved)? {
            RelationalHomeProgramPhase::Pending(required) => self
                .transition
                .as_ref()
                .ok_or(Denial::TransitionNotRequested)?
                .admit_required(required)
                .map_err(Denial::Admission),
            RelationalHomeProgramPhase::Unrestricted => Err(Denial::TransitionNotRequested),
            RelationalHomeProgramPhase::Bound { .. } => Err(Denial::TransitionAlreadyBound),
            RelationalHomeProgramPhase::Refused(cause) => Err(Denial::Admission(*cause)),
        }
    }

    pub(crate) fn admit_completed_home_steps<Source: PartialEq>(
        &self,
        completed: &worth_proof::CompletedHomeProgramTransitionSteps<Source>,
        source: &Source,
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let phase = self.home_phase();
        match phase.as_ref().ok_or(Denial::MediumMoved)? {
            RelationalHomeProgramPhase::Pending(required) => self
                .transition
                .as_ref()
                .ok_or(Denial::TransitionNotRequested)?
                .admit_completed_steps(required, completed, completed.plan_commitment(), source)
                .map_err(Denial::Admission),
            RelationalHomeProgramPhase::Unrestricted => Err(Denial::TransitionNotRequested),
            RelationalHomeProgramPhase::Bound { .. } => Err(Denial::TransitionAlreadyBound),
            RelationalHomeProgramPhase::Refused(cause) => Err(Denial::Admission(*cause)),
        }
    }

    pub(crate) fn install_acknowledged_home_receiver(
        &self,
        completed: HomeProgramTransitionBindingCommitted,
        plan: [u8; 32],
        install: impl FnOnce(),
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let mut phase = self.home_phase();
        let required = match phase.as_ref().ok_or(Denial::MediumMoved)? {
            RelationalHomeProgramPhase::Pending(required) => required.clone(),
            RelationalHomeProgramPhase::Unrestricted => return Err(Denial::TransitionNotRequested),
            RelationalHomeProgramPhase::Bound { .. } => return Err(Denial::TransitionAlreadyBound),
            RelationalHomeProgramPhase::Refused(cause) => return Err(Denial::Admission(*cause)),
        };
        self.transition
            .as_ref()
            .ok_or(Denial::TransitionNotRequested)?
            .admit_binding_committed(&required, &completed)
            .map_err(Denial::Admission)?;
        if completed.plan_commitment() != plan {
            return Err(Denial::Admission(
                worth_proof::HomeProgramTransitionAdmissionDenial::CommitmentMismatch,
            ));
        }
        install();
        *phase = Some(RelationalHomeProgramPhase::Bound {
            required,
            completed,
        });
        Ok(())
    }

    pub(crate) fn requires_home_program_contracts(
        &self,
    ) -> Result<bool, RelationalHomeProgramWriteDenial> {
        let phase = self.home_phase();
        phase
            .as_ref()
            .ok_or(RelationalHomeProgramWriteDenial::MediumMoved)?;
        Ok(self.transition.is_some())
    }

    pub(crate) fn admit_home_program_write(
        &self,
        step: Option<&crate::runtime::AdmittedRelationalHomeProgramStep>,
        codec: Option<&crate::durability::RecoveredCodecPublicationCustody>,
    ) -> Result<(), RelationalHomeProgramWriteDenial> {
        use RelationalHomeProgramWriteDenial as Denial;
        let phase = self.home_phase();
        match phase.as_ref().ok_or(Denial::MediumMoved)? {
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
            RelationalHomeProgramPhase::Bound {
                required,
                completed,
            } => match step {
                None => self
                    .transition
                    .as_ref()
                    .ok_or(Denial::TransitionNotRequested)?
                    .admit_binding_committed(required, completed)
                    .map_err(Denial::Admission),
                Some(_) => Err(Denial::TransitionAlreadyBound),
            },
            RelationalHomeProgramPhase::Refused(cause) => Err(Denial::Admission(*cause)),
        }
    }
}

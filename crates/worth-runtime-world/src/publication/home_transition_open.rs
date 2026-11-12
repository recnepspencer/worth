//! A refused cold-open continuation retains the same original recovered claim.
use crate::recovery::WorldRecoveredUnpublishedEffects;

#[derive(Debug)]
pub enum WorldHomeTransitionUnopenedDenial {
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    OriginalOpenAlreadyDurable,
    Readmission(crate::durability::WorldReadmissionDenial),
    Open(super::WorldOpenAttemptRefusal),
}

#[derive(Debug)]
pub enum WorldHomeTransitionOpenResumeDenial {
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    Recovery(crate::recovery::RuntimeWorldRecoveryDenial),
    Source(super::NoEffectCause),
    Durable(crate::durability::WorldDurableMediumDenial),
    NotHomeBound,
    IdentityScope,
    NativeEffectAlreadyDurable,
    ReleaseInProgress,
}

#[derive(Debug)]
pub struct WorldHomeTransitionOpenResumeRefused {
    pub(crate) cause: WorldHomeTransitionOpenResumeDenial,
    pub(crate) custody: WorldRecoveredUnpublishedEffects,
}
impl WorldHomeTransitionOpenResumeRefused {
    pub fn reason(&self) -> &WorldHomeTransitionOpenResumeDenial {
        &self.cause
    }
    pub fn into_parts(
        self,
    ) -> (
        WorldHomeTransitionOpenResumeDenial,
        WorldRecoveredUnpublishedEffects,
    ) {
        (self.cause, self.custody)
    }
}

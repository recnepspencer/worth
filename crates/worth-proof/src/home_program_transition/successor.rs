//! Exact successor meaning admitted by the attached Home, never by a step caller.
use super::*;

#[derive(Debug)]
pub struct AdmittedHomeProgramTransitionSuccessor<Binding> {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
    binding: Binding,
}
impl<Binding> AdmittedHomeProgramTransitionSuccessor<Binding> {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
}
impl HomeProgramTransitionOwner {
    pub fn admit_successor<Binding>(
        &self,
        commitment: [u8; 32],
        binding: Binding,
    ) -> AdmittedHomeProgramTransitionSuccessor<Binding> {
        AdmittedHomeProgramTransitionSuccessor {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
            binding,
        }
    }
}
impl HomeProgramTransitionAuthority {
    /// The actual Home owner supplies completed recorded meaning. There is no
    /// pending requirement on an ordinary successor reopen, but the original
    /// Home occurrence and serving incarnation must still match this receiver.
    pub fn admit_recorded_successor<Binding>(
        &self,
        successor: &AdmittedHomeProgramTransitionSuccessor<Binding>,
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &successor.occurrence,
            successor.home,
            successor.incarnation,
            &successor.commitment,
            &successor.commitment,
        )
    }

    pub fn admit_successor<Binding>(
        &self,
        required: &HomeProgramTransitionRequired,
        successor: &AdmittedHomeProgramTransitionSuccessor<Binding>,
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit_required(required)?;
        self.admit(
            &successor.occurrence,
            successor.home,
            successor.incarnation,
            &successor.commitment,
            required.binding_commitment(),
        )
    }
}

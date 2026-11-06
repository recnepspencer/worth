//! Move-only legal permission; its owner runtime retains single-issuance state.
use super::{
    HomeProgramTransitionAdmissionDenial, HomeProgramTransitionAuthority,
    HomeProgramTransitionOwner, TransitionOccurrence,
};
use crate::HomeIncarnationFence;
use std::sync::Arc;

/// Move-only permission to begin the original assigned open after readmission.
/// The admitting runtime owns single issuance; this witness holds no live table.
///
/// ```compile_fail,E0382
/// use worth_proof::AdmittedHomeProgramTransitionExecution;
/// fn cannot_repeat(execution: AdmittedHomeProgramTransitionExecution) {
///     let first = execution;
///     let second = execution;
/// }
/// ```
#[derive(Debug)]
pub struct AdmittedHomeProgramTransitionExecution {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
    caller_association: [u8; 32],
}
impl HomeProgramTransitionOwner {
    pub fn admit_execution(
        &self,
        commitment: [u8; 32],
        caller_association: [u8; 32],
    ) -> AdmittedHomeProgramTransitionExecution {
        AdmittedHomeProgramTransitionExecution {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
            caller_association,
        }
    }
}

impl HomeProgramTransitionAuthority {
    pub fn admit_execution(
        &self,
        proof: &AdmittedHomeProgramTransitionExecution,
        commitment: &[u8; 32],
        caller_association: &[u8; 32],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &proof.occurrence,
            proof.home,
            proof.incarnation,
            &proof.commitment,
            commitment,
        )?;
        if &proof.caller_association != caller_association {
            return Err(HomeProgramTransitionAdmissionDenial::CallerAssociationMismatch);
        }
        Ok(())
    }
}

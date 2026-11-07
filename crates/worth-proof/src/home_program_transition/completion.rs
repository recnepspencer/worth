//! Completion permission is bound to the same Home and exact final source.
use super::*;

/// The Home owner issues this after all original step publications settle.
/// The source is port-free identity data, not a current read or write handle.
#[derive(Debug)]
pub struct CompletedHomeProgramTransitionSteps<Source> {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    binding: [u8; 32],
    plan: [u8; 32],
    source: Source,
}
impl<Source> CompletedHomeProgramTransitionSteps<Source> {
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub const fn plan_commitment(&self) -> [u8; 32] {
        self.plan
    }
}
impl HomeProgramTransitionOwner {
    pub fn admit_completed_steps<Source>(
        &self,
        binding: [u8; 32],
        plan: [u8; 32],
        source: Source,
    ) -> CompletedHomeProgramTransitionSteps<Source> {
        CompletedHomeProgramTransitionSteps {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            binding,
            plan,
            source,
        }
    }
}
impl HomeProgramTransitionAuthority {
    pub fn admit_completed_steps<Source: PartialEq>(
        &self,
        required: &HomeProgramTransitionRequired,
        completed: &CompletedHomeProgramTransitionSteps<Source>,
        plan: [u8; 32],
        source: &Source,
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit_required(required)?;
        self.admit(
            &completed.occurrence,
            completed.home,
            completed.incarnation,
            &completed.binding,
            required.binding_commitment(),
        )?;
        if completed.plan != plan {
            return Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch);
        }
        if &completed.source != source {
            return Err(HomeProgramTransitionAdmissionDenial::SourceMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;
    #[test]
    fn completion_requires_original_owner_incarnation_binding_plan_and_final_source() {
        let owner = |incarnation| {
            HomeProgramTransitionOwner::fresh(
                [1; 16],
                HomeIncarnationFence::from_owner(NonZeroU64::new(incarnation).unwrap()),
            )
        };
        let first = owner(1);
        let foreign = owner(1);
        let later = owner(2);
        let required = first.require_transition([7; 32]);
        let authority = first.authority();
        let admit =
            |completed| authority.admit_completed_steps(&required, &completed, [8; 32], &19u64);
        assert_eq!(
            admit(first.admit_completed_steps([7; 32], [8; 32], 19)),
            Ok(())
        );
        assert_eq!(
            admit(foreign.admit_completed_steps([7; 32], [8; 32], 19)),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        assert_eq!(
            admit(later.admit_completed_steps([7; 32], [8; 32], 19)),
            Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
        );
        assert_eq!(
            admit(first.admit_completed_steps([9; 32], [8; 32], 19)),
            Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
        );
        assert_eq!(
            admit(first.admit_completed_steps([7; 32], [9; 32], 19)),
            Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
        );
        assert_eq!(
            admit(first.admit_completed_steps([7; 32], [8; 32], 20)),
            Err(HomeProgramTransitionAdmissionDenial::SourceMismatch)
        );
    }
}

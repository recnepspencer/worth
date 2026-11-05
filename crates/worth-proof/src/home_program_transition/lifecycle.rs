//! Plain lifecycle witnesses. The Home runtime owns publication and live state.
use super::*;

#[derive(Clone, Debug)]
pub struct HomeProgramTransitionRequired {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    binding: [u8; 32],
}

/// Issued by the same Home owner only after the final descriptor binding ack.
#[derive(Clone, Debug)]
pub struct HomeProgramTransitionBindingCommitted {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    binding: [u8; 32],
    plan: [u8; 32],
}

impl HomeProgramTransitionOwner {
    pub fn require_transition(&self, binding: [u8; 32]) -> HomeProgramTransitionRequired {
        HomeProgramTransitionRequired {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            binding,
        }
    }

    pub fn admit_binding_committed(
        &self,
        binding: [u8; 32],
        plan: [u8; 32],
    ) -> HomeProgramTransitionBindingCommitted {
        HomeProgramTransitionBindingCommitted {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            binding,
            plan,
        }
    }
}

impl HomeProgramTransitionBindingCommitted {
    pub const fn plan_commitment(&self) -> [u8; 32] {
        self.plan
    }
}

impl HomeProgramTransitionAuthority {
    pub fn admit_required(
        &self,
        required: &HomeProgramTransitionRequired,
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &required.occurrence,
            required.home,
            required.incarnation,
            &required.binding,
            &required.binding,
        )
    }

    pub fn admit_binding_committed(
        &self,
        required: &HomeProgramTransitionRequired,
        completed: &HomeProgramTransitionBindingCommitted,
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit_required(required)?;
        self.admit(
            &completed.occurrence,
            completed.home,
            completed.incarnation,
            &completed.binding,
            &required.binding,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    #[test]
    fn completion_requires_the_same_home_owner_incarnation_and_requested_binding() {
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
        assert!(authority
            .admit_binding_committed(&required, &first.admit_binding_committed([7; 32], [8; 32]))
            .is_ok());
        assert_eq!(
            authority.admit_binding_committed(
                &required,
                &first.admit_binding_committed([9; 32], [8; 32])
            ),
            Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
        );
        assert_eq!(
            authority.admit_binding_committed(
                &required,
                &foreign.admit_binding_committed([7; 32], [8; 32])
            ),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        assert_eq!(
            later.authority().admit_required(&required),
            Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
        );
    }
}

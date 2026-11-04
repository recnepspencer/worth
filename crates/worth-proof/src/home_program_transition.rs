//! Exact Home-owner admission across transition reservation and execution.
use crate::HomeIncarnationFence;
use std::sync::Arc;

#[derive(Debug)]
struct TransitionOccurrence {
    _retained: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
}

/// Store retains this seal with its actual admitted Home transition owner.
/// A freshly created foreign seal cannot satisfy an already attached receiver.
#[derive(Debug)]
pub struct HomeProgramTransitionOwner {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}

/// Attached once with the real Home medium, never supplied by a step caller.
#[derive(Clone, Debug)]
pub struct HomeProgramTransitionAuthority {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}

/// Permission to reserve original World associations, before intent publication.
/// This is deliberately insufficient for executing a transition step.
#[derive(Debug)]
pub struct AdmittedHomeProgramTransitionPlan {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
}

/// Store issues this only from its original committed or readmitted intent.
/// The receiving owner also checks the complete step-description commitment.
#[derive(Debug)]
pub struct AdmittedHomeProgramTransitionStep {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeProgramTransitionAdmissionDenial {
    StaleIncarnation,
    ForeignOwner,
    CommitmentMismatch,
}

impl HomeProgramTransitionOwner {
    pub fn fresh(home: [u8; 16], incarnation: HomeIncarnationFence) -> Self {
        Self {
            occurrence: Arc::new(TransitionOccurrence { _retained: None }),
            home,
            incarnation,
        }
    }

    /// Couple the runtime's original backing charge to every share of this
    /// seal, including verification authority retained after owner retirement.
    /// Each call still creates a distinct occurrence even for shared backing.
    pub fn from_retained_owner_backing(
        home: [u8; 16],
        incarnation: HomeIncarnationFence,
        retained: Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Self {
        Self {
            occurrence: Arc::new(TransitionOccurrence {
                _retained: Some(retained),
            }),
            home,
            incarnation,
        }
    }

    /// Fixed original seal allocation, including the Arc reference counters.
    /// The runtime separately admits the backing allocation it supplies.
    pub fn retained_occurrence_bytes() -> usize {
        std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<TransitionOccurrence>())
            .expect("the fixed seal layout fits the target pointer width")
            .0
            .pad_to_align()
            .size()
    }

    pub fn authority(&self) -> HomeProgramTransitionAuthority {
        HomeProgramTransitionAuthority {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
        }
    }

    pub fn admit_plan(&self, commitment: [u8; 32]) -> AdmittedHomeProgramTransitionPlan {
        AdmittedHomeProgramTransitionPlan {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
        }
    }

    pub fn admit_step(&self, commitment: [u8; 32]) -> AdmittedHomeProgramTransitionStep {
        AdmittedHomeProgramTransitionStep {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
        }
    }
}

impl HomeProgramTransitionAuthority {
    pub fn admit_plan(
        &self,
        proof: &AdmittedHomeProgramTransitionPlan,
        commitment: &[u8; 32],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &proof.occurrence,
            proof.home,
            proof.incarnation,
            &proof.commitment,
            commitment,
        )
    }

    pub fn admit_step(
        &self,
        proof: &AdmittedHomeProgramTransitionStep,
        commitment: &[u8; 32],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &proof.occurrence,
            proof.home,
            proof.incarnation,
            &proof.commitment,
            commitment,
        )
    }

    fn admit(
        &self,
        occurrence: &Arc<TransitionOccurrence>,
        home: [u8; 16],
        incarnation: HomeIncarnationFence,
        presented: &[u8; 32],
        expected: &[u8; 32],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        use HomeProgramTransitionAdmissionDenial as Denial;
        // Admit Home scope and fence before commitment inspection or owner effects.
        if self.home != home {
            return Err(Denial::ForeignOwner);
        }
        self.incarnation
            .admit(incarnation)
            .map_err(|cause| match cause {
                crate::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
                crate::HomeIncarnationDenial::ForeignHome => Denial::ForeignOwner,
            })?;
        if !Arc::ptr_eq(&self.occurrence, occurrence) {
            return Err(Denial::ForeignOwner);
        }
        if presented != expected {
            return Err(Denial::CommitmentMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    fn owner(incarnation: u64) -> HomeProgramTransitionOwner {
        HomeProgramTransitionOwner::fresh(
            [1; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::new(incarnation).unwrap()),
        )
    }

    #[test]
    fn transition_admission_requires_the_attached_owner_and_exact_commitment() {
        let installed = owner(1);
        let foreign = owner(1);
        let authority = installed.authority();
        let proof = installed.admit_step([7; 32]);
        assert_eq!(authority.admit_step(&proof, &[7; 32]), Ok(()));
        assert_eq!(
            authority.admit_step(&proof, &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
        );
        assert_eq!(
            authority.admit_step(&foreign.admit_step([7; 32]), &[7; 32]),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        let plan = installed.admit_plan([9; 32]);
        assert_eq!(authority.admit_plan(&plan, &[9; 32]), Ok(()));
        assert_eq!(
            authority.admit_plan(&foreign.admit_plan([9; 32]), &[9; 32]),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
    }

    #[test]
    fn earlier_incarnation_transition_proofs_refuse_before_commitment_inspection() {
        let earlier = owner(1);
        let current = owner(2).authority();
        let foreign = HomeProgramTransitionOwner::fresh(
            [2; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap()),
        );
        assert_eq!(
            current.admit_step(&foreign.admit_step([7; 32]), &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        assert_eq!(
            current.admit_step(&earlier.admit_step([7; 32]), &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
        );
        assert_eq!(
            current.admit_plan(&earlier.admit_plan([7; 32]), &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
        );
    }

    #[test]
    fn transition_proofs_retain_original_backing_without_sharing_owner_authority() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        #[derive(Debug)]
        struct Backing(Arc<AtomicUsize>);
        impl Drop for Backing {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let reclaimed = Arc::new(AtomicUsize::new(0));
        let backing = Arc::new(Backing(reclaimed.clone()));
        let fence = HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap());
        let owner = HomeProgramTransitionOwner::from_retained_owner_backing(
            [1; 16],
            fence,
            backing.clone(),
        );
        let other = HomeProgramTransitionOwner::from_retained_owner_backing(
            [1; 16],
            fence,
            backing.clone(),
        );
        let authority = owner.authority();
        let proof = owner.admit_step([7; 32]);
        assert_eq!(
            other.authority().admit_step(&proof, &[7; 32]),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        drop(backing);
        drop(owner);
        drop(other);
        assert_eq!(reclaimed.load(Ordering::SeqCst), 0);
        assert_eq!(authority.admit_step(&proof, &[7; 32]), Ok(()));
        drop(authority);
        assert_eq!(reclaimed.load(Ordering::SeqCst), 0);
        drop(proof);
        assert_eq!(reclaimed.load(Ordering::SeqCst), 1);
    }
}

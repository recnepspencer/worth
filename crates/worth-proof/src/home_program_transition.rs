//! Exact Home-owner admission across transition reservation and execution.
use crate::HomeIncarnationFence;
use std::sync::Arc;
mod catalog;
pub use catalog::AdmittedHomeProgramTransitionCatalog;
mod execution;
pub use execution::AdmittedHomeProgramTransitionExecution;
mod lifecycle;
pub use lifecycle::{HomeProgramTransitionBindingCommitted, HomeProgramTransitionRequired};

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
#[derive(Debug, Clone)]
pub struct AdmittedHomeProgramTransitionStep<Source> {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
    caller_association: [u8; 32],
    body: [u8; 32],
    source: Source,
}

impl<Source> AdmittedHomeProgramTransitionStep<Source> {
    /// Exact port-free source identity supplied by the admitting Home owner.
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The exact authored body retained by the original committed Home intent.
    pub const fn body_commitment(&self) -> [u8; 32] {
        self.body
    }

    /// Exact opaque association encoded by the assigning World owner.
    pub const fn caller_association_commitment(&self) -> [u8; 32] {
        self.caller_association
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeProgramTransitionAdmissionDenial {
    StaleIncarnation,
    ForeignOwner,
    CommitmentMismatch,
    CallerAssociationMismatch,
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

    pub fn admit_step<Source>(
        &self,
        commitment: [u8; 32],
        caller_association: [u8; 32],
        body: [u8; 32],
        source: Source,
    ) -> AdmittedHomeProgramTransitionStep<Source> {
        AdmittedHomeProgramTransitionStep {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
            caller_association,
            body,
            source,
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

    pub fn admit_step<Source>(
        &self,
        proof: &AdmittedHomeProgramTransitionStep<Source>,
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
mod tests;

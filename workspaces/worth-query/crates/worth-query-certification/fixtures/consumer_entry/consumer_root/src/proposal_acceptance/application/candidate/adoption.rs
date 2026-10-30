//! Guarded replacement updates the actual model slot before returning performed cleanup custody.
use super::*;
mod previous;
mod retired;
pub use previous::{Performed, PreviousCloseFailure};
pub use retired::{RetiredAdoption, SettledAdoption};

#[derive(Debug)]
pub enum AdoptionRefusal {
    Authorization(PreparationRefusal),
    Advancement(WorthQueryAdvancementDenial),
    Owner(WorthQueryExpectedProductBranchAdoptionDenial),
}
pub struct AdoptionFailure<'host> {
    pub ready: ReadyToAdopt<'host>,
    pub cause: AdoptionRefusal,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
struct AdoptionEvidence {
    metadata: AdoptionMetadata,
    proposal: StoredProposal,
    completed: Vec<retention_action::Completion>,
    handoff: Option<WorthQueryAdvancementDenial>,
}
pub struct Adopted<'host> {
    evidence: AdoptionEvidence,
    previous: OwnedBranch<'host>,
}
pub enum FinishedAdoption {
    PreviousClosed(Performed),
    RetiredCleanupSettled(SettledAdoption),
}
impl FinishedAdoption {
    pub fn adoption_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        match self {
            Self::PreviousClosed(performed) => performed.adoption_handoff(),
            Self::RetiredCleanupSettled(settled) => settled.adoption_handoff(),
        }
    }
    pub fn previous_close(&self) -> Option<&Closed> {
        match self {
            Self::PreviousClosed(performed) => Some(performed.previous_close()),
            _ => None,
        }
    }
    pub fn cleanup_receipt(&self) -> Option<&WorthQueryProductBranchOwnerCleanupReceipt> {
        match self {
            Self::RetiredCleanupSettled(settled) => Some(settled.cleanup_receipt()),
            _ => None,
        }
    }
}
impl<'host> ReadyToAdopt<'host> {
    pub fn adopt(
        mut self,
        model: &mut Model<'host>,
        caller: &Caller<'_>,
    ) -> Result<Adopted<'host>, AdoptionFailure<'host>> {
        if let Err(cause) = preparation::authorize(model, caller, &self.staging.proposal) {
            return Err(AdoptionFailure {
                ready: self,
                cause: AdoptionRefusal::Authorization(cause),
                handoff: None,
            });
        }
        let mut adopted = None;
        let host = self.staging.host;
        let advancement = host.with_application_advancement(caller.scope, |phase| {
            let result = self
                .staging
                .candidate
                .take()
                .expect("prepared candidate")
                .adopt(&mut model.owner.head, &phase);
            if result.is_ok() {
                // Metadata was allocated before creation. This move cannot fail after adoption.
                model.last_adoption = self.staging.model_metadata.take();
            }
            adopted = Some(result);
        });
        match (adopted, advancement) {
            (Some(Ok(previous)), handoff) => Ok(Adopted {
                evidence: AdoptionEvidence {
                    metadata: self.staging.metadata,
                    proposal: self.staging.proposal,
                    completed: self.staging.completed,
                    handoff: handoff.err(),
                },
                previous: OwnedBranch {
                    host,
                    head: previous,
                },
            }),
            (Some(Err(failure)), handoff) => {
                let (candidate, cause) = failure.into_parts();
                self.staging.candidate = Some(candidate);
                Err(AdoptionFailure {
                    ready: self,
                    cause: AdoptionRefusal::Owner(cause),
                    handoff: handoff.err(),
                })
            }
            (None, Err(cause)) => Err(AdoptionFailure {
                ready: self,
                cause: AdoptionRefusal::Advancement(cause),
                handoff: None,
            }),
            (None, Ok(())) => unreachable!("successful advancement invoked adoption"),
        }
    }
    pub fn close(self, caller: &Caller<'_>) -> Result<Closed, CloseFailure<'host>> {
        self.staging.close(caller)
    }
}
impl<'host> Adopted<'host> {
    pub fn metadata(&self) -> &AdoptionMetadata {
        &self.evidence.metadata
    }
    pub fn proposal(&self) -> &StoredProposal {
        &self.evidence.proposal
    }
    pub fn completed(&self) -> &[retention_action::Completion] {
        &self.evidence.completed
    }
    pub fn previous_branch(&self) -> WorthQueryProductBranch {
        self.previous.branch()
    }
    pub fn handoff(&self) -> Option<&WorthQueryAdvancementDenial> {
        self.evidence.handoff.as_ref()
    }
}

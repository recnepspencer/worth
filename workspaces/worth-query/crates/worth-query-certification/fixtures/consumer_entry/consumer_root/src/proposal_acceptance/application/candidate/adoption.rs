//! Guarded replacement updates the actual model slot before returning performed cleanup custody.
use super::*;

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
pub struct Adopted<'host> {
    pub metadata: AdoptionMetadata,
    pub proposal: StoredProposal,
    pub completed: Vec<retention_action::Completion>,
    pub previous: OwnedBranch<'host>,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
pub struct Performed {
    pub metadata: AdoptionMetadata,
    pub proposal: StoredProposal,
    pub completed: Vec<retention_action::Completion>,
    pub previous: Closed,
    pub adoption_handoff: Option<WorthQueryAdvancementDenial>,
}
pub struct PreviousCloseFailure<'host> {
    pub adopted: Adopted<'host>,
    pub cause: Failure,
    pub handoff: Option<WorthQueryAdvancementDenial>,
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
                metadata: self.staging.metadata,
                proposal: self.staging.proposal,
                completed: self.staging.completed,
                previous: OwnedBranch {
                    host,
                    head: previous,
                },
                handoff: handoff.err(),
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
    pub fn close_previous(
        self,
        caller: &Caller<'_>,
    ) -> Result<Performed, PreviousCloseFailure<'host>> {
        let Self {
            metadata,
            proposal,
            completed,
            previous,
            handoff,
        } = self;
        match previous.close(caller) {
            Ok(previous) => Ok(Performed {
                metadata,
                proposal,
                completed,
                previous,
                adoption_handoff: handoff,
            }),
            Err(failure) => Err(PreviousCloseFailure {
                adopted: Self {
                    metadata,
                    proposal,
                    completed,
                    previous: failure.owner,
                    handoff,
                },
                cause: failure.cause,
                handoff: failure.handoff,
            }),
        }
    }
}

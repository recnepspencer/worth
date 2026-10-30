//! Ordinary close preserves actual adoption evidence and transfers only genuine retired cleanup.
use super::*;
pub struct Performed {
    evidence: AdoptionEvidence,
    previous: Closed,
}
pub struct PreviousCloseFailure<'host> {
    pub adopted: Adopted<'host>,
    pub cause: Failure,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
impl<'host> Adopted<'host> {
    pub fn close_previous(
        self,
        caller: &Caller<'_>,
    ) -> Result<Performed, PreviousCloseFailure<'host>> {
        let Self { evidence, previous } = self;
        match previous.close(caller) {
            Ok(previous) => Ok(Performed { evidence, previous }),
            Err(failure) => Err(PreviousCloseFailure {
                adopted: Self {
                    evidence,
                    previous: failure.owner,
                },
                cause: failure.cause,
                handoff: failure.handoff,
            }),
        }
    }
}
impl Performed {
    pub fn previous_close(&self) -> &Closed {
        &self.previous
    }
    pub fn adoption_handoff(&self) -> Option<WorthQueryAdvancementDenial> {
        self.evidence.handoff
    }
}
impl<'host> PreviousCloseFailure<'host> {
    pub fn into_retired(self) -> Result<RetiredAdoption<'host>, Self> {
        let Self {
            adopted,
            cause,
            handoff,
        } = self;
        match cause {
            Failure::Close(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(
                failure,
            )) => {
                let Adopted { evidence, previous } = adopted;
                let host = previous.host;
                drop(previous); // This bare branch has already retired; it cannot close again.
                Ok(RetiredAdoption::from_retired(
                    evidence,
                    host,
                    failure.into_cleanup(),
                    handoff,
                ))
            }
            cause => Err(Self {
                adopted,
                cause,
                handoff,
            }),
        }
    }
}

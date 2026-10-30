//! Explicit disposal closes the real ordinary operation before the candidate/source owner.
use super::*;

#[derive(Debug)]
pub enum CloseRefusal {
    Operation(WorthQueryOperationCloseDenial),
    Candidate(WorthQueryApplicationProductBranchCloseDenial),
    Advancement(WorthQueryAdvancementDenial),
}
pub struct CloseFailure<'host> {
    pub staging: Staging<'host>,
    pub cause: CloseRefusal,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
impl CloseFailure<'_> {
    /// Abandons staged business evidence/retry only after the owning close has
    /// retired the candidate. The returned original cleanup remains mandatory.
    /// Every pre-retirement refusal returns this entire staging owner unchanged.
    pub fn into_retired_cleanup(
        self,
    ) -> Result<
        (
            WorthQueryApplicationProductBranchCleanup,
            Option<WorthQueryAdvancementDenial>,
        ),
        Self,
    > {
        let Self {
            staging,
            cause,
            handoff,
        } = self;
        match cause {
            CloseRefusal::Candidate(
                WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure),
            ) => {
                let cleanup = failure.into_cleanup();
                drop(staging);
                Ok((cleanup, handoff))
            }
            cause => Err(Self {
                staging,
                cause,
                handoff,
            }),
        }
    }
}
impl<'host> Staging<'host> {
    pub fn close(mut self, caller: &Caller<'_>) -> Result<Closed, CloseFailure<'host>> {
        let request = self
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch());
        if let Some(step) = self.step.take() {
            let refusal = match step {
                Step::Pending(operation) => operation
                    .close(&request)
                    .map_err(|(owner, cause)| (Step::Pending(owner), cause)),
                Step::Completed(completed) => completed
                    .close(&request)
                    .map_err(|(owner, cause)| (Step::Completed(owner), cause)),
                Step::Terminal(terminal) => terminal
                    .close(&request)
                    .map_err(|(owner, cause)| (Step::Terminal(owner), cause)),
            };
            if let Err((step, cause)) = refusal {
                self.step = Some(step);
                return Err(CloseFailure {
                    staging: self,
                    cause: CloseRefusal::Operation(cause),
                    handoff: None,
                });
            }
        }
        // Release our completed read observations while preserving original receipts/results.
        for evidence in &mut self.completed {
            match evidence {
                WorthQuerySourceCompletion::Fresh { retained, .. }
                | WorthQuerySourceCompletion::Recovered { retained, .. } => {
                    drop(retained.take());
                }
            }
        }
        let mut closed = None;
        let advancement = self
            .host
            .with_application_advancement(caller.scope, |phase| {
                closed = Some(self.candidate.take().expect("live candidate").close(&phase));
            });
        match (closed, advancement) {
            (Some(Ok(receipt)), handoff) => Ok(Closed {
                receipt,
                handoff: handoff.err(),
            }),
            (Some(Err(failure)), handoff) => {
                let (candidate, cause) = failure.into_parts();
                self.candidate = Some(candidate);
                Err(CloseFailure {
                    staging: self,
                    cause: CloseRefusal::Candidate(cause),
                    handoff: handoff.err(),
                })
            }
            (None, Err(cause)) => Err(CloseFailure {
                staging: self,
                cause: CloseRefusal::Advancement(cause),
                handoff: None,
            }),
            (None, Ok(())) => unreachable!("successful advancement invoked candidate close"),
        }
    }
}

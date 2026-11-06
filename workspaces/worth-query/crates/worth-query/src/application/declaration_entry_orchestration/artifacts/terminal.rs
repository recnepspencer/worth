use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[cfg(test)]
use super::outcome::WorthQueryDeclarationEntryOrchestrationOutcome;
use super::outcome::{
    WorthQueryDeclarationEntryOrchestrationDeferred, WorthQueryDeclarationEntryOrchestrationDenied,
    WorthQueryDeclarationEntryOrchestrationFailed,
    WorthQueryDeclarationEntryOrchestrationRebindRequired,
    WorthQueryDeclarationEntryOrchestrationStale,
};
use super::refusal::WorthQueryDeclarationEntryOrchestrationRefusal;

pub enum WorthQueryDeclarationEntryOrchestrationTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationEntryOrchestrationDeferred<D, I>),
    Denied(WorthQueryDeclarationEntryOrchestrationDenied<D, I>),
    Stale(WorthQueryDeclarationEntryOrchestrationStale<D, I>),
    RebindRequired(WorthQueryDeclarationEntryOrchestrationRebindRequired<D, I>),
    Failed(WorthQueryDeclarationEntryOrchestrationFailed<D, I>),
    Refused(WorthQueryDeclarationEntryOrchestrationRefusal<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationEntryOrchestrationTerminalError<D, I>
{
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Deferred(outcome) => outcome.reason(),
            Self::Denied(outcome) => outcome.reason(),
            Self::Stale(outcome) => outcome.reason(),
            Self::RebindRequired(outcome) => outcome.reason(),
            Self::Failed(outcome) => outcome.reason(),
            Self::Refused(outcome) => outcome.reason(),
        }
    }
}

#[cfg(test)]
pub(crate) fn terminal_error_from_outcome<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    outcome: WorthQueryDeclarationEntryOrchestrationOutcome<D, I>,
) -> Result<
    crate::application::WorthQueryDeclarationEnvelope<D, I>,
    WorthQueryDeclarationEntryOrchestrationTerminalError<D, I>,
> {
    match outcome {
        WorthQueryDeclarationEntryOrchestrationOutcome::Enveloped(envelope) => Ok(envelope),
        WorthQueryDeclarationEntryOrchestrationOutcome::Deferred(outcome) => {
            Err(WorthQueryDeclarationEntryOrchestrationTerminalError::Deferred(outcome))
        }
        WorthQueryDeclarationEntryOrchestrationOutcome::Denied(outcome) => {
            Err(WorthQueryDeclarationEntryOrchestrationTerminalError::Denied(outcome))
        }
        WorthQueryDeclarationEntryOrchestrationOutcome::Stale(outcome) => Err(
            WorthQueryDeclarationEntryOrchestrationTerminalError::Stale(outcome),
        ),
        WorthQueryDeclarationEntryOrchestrationOutcome::RebindRequired(outcome) => {
            Err(WorthQueryDeclarationEntryOrchestrationTerminalError::RebindRequired(outcome))
        }
        WorthQueryDeclarationEntryOrchestrationOutcome::Failed(outcome) => {
            Err(WorthQueryDeclarationEntryOrchestrationTerminalError::Failed(outcome))
        }
        WorthQueryDeclarationEntryOrchestrationOutcome::Refused(outcome) => {
            Err(WorthQueryDeclarationEntryOrchestrationTerminalError::Refused(outcome))
        }
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryOrchestrationTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

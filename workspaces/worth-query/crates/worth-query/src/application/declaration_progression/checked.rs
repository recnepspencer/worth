use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::admitted::WorthQueryAdmittedDeclarationProgression;
use super::denial::WorthQueryDeclarationProgressionFailed;
use super::denial::{
    WorthQueryDeclarationProgressionDeferred, WorthQueryDeclarationProgressionDenied,
};
use super::rebind::WorthQueryDeclarationProgressionRebindRequired;
use super::stale::WorthQueryDeclarationProgressionStale;

pub enum WorthQueryDeclarationProgressionChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Admitted(WorthQueryAdmittedDeclarationProgression<D, I>),
    Deferred(WorthQueryDeclarationProgressionDeferred<D, I>),
    Denied(WorthQueryDeclarationProgressionDenied<D, I>),
    Stale(WorthQueryDeclarationProgressionStale<D, I>),
    RebindRequired(WorthQueryDeclarationProgressionRebindRequired<D, I>),
    Failed(WorthQueryDeclarationProgressionFailed<D, I>),
}

pub enum WorthQueryDeclarationProgressionTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationProgressionDeferred<D, I>),
    Denied(WorthQueryDeclarationProgressionDenied<D, I>),
    Stale(WorthQueryDeclarationProgressionStale<D, I>),
    RebindRequired(WorthQueryDeclarationProgressionRebindRequired<D, I>),
    Failed(WorthQueryDeclarationProgressionFailed<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationProgressionTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

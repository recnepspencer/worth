use crate::application::{
    WorthQueryDeclarationAdmissionError, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::WorthQueryDeclarationLegalityDenial;

pub enum WorthQueryDeclarationAdmissionOrLegalityError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Admission(WorthQueryDeclarationAdmissionError<D, I>),
    Legality(WorthQueryDeclarationLegalityDenial<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationAdmissionOrLegalityError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

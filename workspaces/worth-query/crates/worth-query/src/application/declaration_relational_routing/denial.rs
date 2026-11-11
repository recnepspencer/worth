use crate::application::{
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationInput,
    WorthQueryDeclarationRelationalAuthorityFamily, WorthQueryDeclarationRelationalTruthClaim,
    WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalRoutingDenialCause {
    EnvelopeNotCoveredForRelationalRouting,
    NonRelationalRoutePlan,
    UnsupportedRelationalTruthClaim,
    RelationalAuthorityUnavailable,
    MissingRequiredAspect,
    AspectConflict,
    RelationalAspectGap,
    RelationalEnvelopeMismatch,
}

impl WorthQueryDeclarationRelationalRoutingDenialCause {
    pub fn reason(self) -> &'static str {
        match self {
            Self::EnvelopeNotCoveredForRelationalRouting => {
                "relational truth routing starts from covered envelope truth rather than non-success crossing posture"
            }
            Self::NonRelationalRoutePlan => {
                "the retained route plan does not currently admit a relational slice"
            }
            Self::UnsupportedRelationalTruthClaim => {
                "this declaration family does not expose a supported relational truth-routing contract"
            }
            Self::RelationalAuthorityUnavailable => {
                "required relational authority capabilities are unavailable in this operating world"
            }
            Self::MissingRequiredAspect => {
                "the retained envelope publication does not expose the required relational semantic slice"
            }
            Self::AspectConflict => {
                "the retained envelope publication conflicts with the required relational semantic slice"
            }
            Self::RelationalAspectGap => {
                "the retained envelope publication only partially covers the required relational semantic slice"
            }
            Self::RelationalEnvelopeMismatch => {
                "the retained envelope truth and relational boundary expectations no longer agree"
            }
        }
    }
}

macro_rules! define_relational_terminal {
    ($name:ident) => {
        pub struct $name<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> {
            envelope: WorthQueryDeclarationEnvelope<D, I>,
            reason: &'static str,
        }

        impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> $name<D, I> {
            pub(crate) fn new(
                envelope: WorthQueryDeclarationEnvelope<D, I>,
                reason: &'static str,
            ) -> Self {
                Self { envelope, reason }
            }

            pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
                &self.envelope
            }

            pub fn reason(&self) -> &'static str {
                self.reason
            }

            #[cfg(test)]
            pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
                self.envelope
            }
        }
    };
}

define_relational_terminal!(WorthQueryDeclarationRelationalRoutingDeferred);
define_relational_terminal!(WorthQueryDeclarationRelationalRoutingFailed);

pub struct WorthQueryDeclarationRelationalRoutingDenied<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    envelope: WorthQueryDeclarationEnvelope<D, I>,
    truth_claim: Option<WorthQueryDeclarationRelationalTruthClaim>,
    authority_family: Option<WorthQueryDeclarationRelationalAuthorityFamily>,
    cause: WorthQueryDeclarationRelationalRoutingDenialCause,
    reason: &'static str,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationRelationalRoutingDenied<D, I>
{
    pub(crate) fn new(
        envelope: WorthQueryDeclarationEnvelope<D, I>,
        truth_claim: Option<WorthQueryDeclarationRelationalTruthClaim>,
        authority_family: Option<WorthQueryDeclarationRelationalAuthorityFamily>,
        cause: WorthQueryDeclarationRelationalRoutingDenialCause,
    ) -> Self {
        Self {
            envelope,
            truth_claim,
            authority_family,
            cause,
            reason: cause.reason(),
        }
    }

    pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
        &self.envelope
    }

    pub fn cause(&self) -> WorthQueryDeclarationRelationalRoutingDenialCause {
        self.cause
    }

    pub fn truth_claim(&self) -> Option<WorthQueryDeclarationRelationalTruthClaim> {
        self.truth_claim
    }

    pub fn authority_family(&self) -> Option<WorthQueryDeclarationRelationalAuthorityFamily> {
        self.authority_family
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }

    #[cfg(test)]
    pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
        self.envelope
    }
}

pub enum WorthQueryDeclarationRelationalRoutingTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationRelationalRoutingDeferred<D, I>),
    Denied(WorthQueryDeclarationRelationalRoutingDenied<D, I>),
    Failed(WorthQueryDeclarationRelationalRoutingFailed<D, I>),
}

pub enum WorthQueryDeclarationEntryRelationalRoutingError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(crate::application::WorthQueryDeclarationEntryEnvelopeError<D, I>),
    Routing(WorthQueryDeclarationRelationalRoutingTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationRelationalRoutingTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryRelationalRoutingError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

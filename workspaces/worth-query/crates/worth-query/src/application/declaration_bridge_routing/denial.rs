use crate::application::{
    WorthQueryDeclarationBridgeContinuationFamily, WorthQueryDeclarationBridgeContinuationRequest,
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationBridgeRoutingDenialCause {
    EnvelopeNotCoveredForBridgeRouting,
    NonBridgeRoutePlan,
    UnsupportedContinuationMode,
    UnsupportedTruthContext,
    BridgeAuthorityUnavailable,
    MissingRequiredAspect,
    AspectConflict,
    AuthorityAspectGap,
    AuthorityAspectAmbiguity,
    BridgeEnvelopeMismatch,
    BasisLifecycleMismatch,
}

impl WorthQueryDeclarationBridgeRoutingDenialCause {
    pub fn reason(self) -> &'static str {
        match self {
            Self::EnvelopeNotCoveredForBridgeRouting => {
                "bridge continuation routing starts from covered envelope truth rather than non-success crossing posture"
            }
            Self::NonBridgeRoutePlan => {
                "the retained route plan does not currently admit a bridge slice"
            }
            Self::UnsupportedContinuationMode => {
                "this declaration family does not expose a supported bridge continuation-routing contract"
            }
            Self::UnsupportedTruthContext => {
                "the requested truth context is not supported for this bridge continuation family"
            }
            Self::BridgeAuthorityUnavailable => {
                "required bridge continuation capabilities are unavailable in this operating world"
            }
            Self::MissingRequiredAspect => {
                "the retained envelope publication does not expose the required bridge semantic slice"
            }
            Self::AspectConflict => {
                "the retained envelope publication conflicts with the required bridge semantic slice"
            }
            Self::AuthorityAspectGap => {
                "the retained envelope publication only partially covers the required bridge semantic slice"
            }
            Self::AuthorityAspectAmbiguity => {
                "multiple bridge mappings claim the same retained semantic slice"
            }
            Self::BridgeEnvelopeMismatch => {
                "the retained envelope truth and bridge continuation boundary expectations no longer agree"
            }
            Self::BasisLifecycleMismatch => {
                "the retained basis lifecycle posture does not satisfy this bridge continuation mode"
            }
        }
    }
}

macro_rules! define_bridge_terminal {
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

define_bridge_terminal!(WorthQueryDeclarationBridgeRoutingDeferred);
define_bridge_terminal!(WorthQueryDeclarationBridgeRoutingFailed);

pub struct WorthQueryDeclarationBridgeRoutingDenied<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    envelope: WorthQueryDeclarationEnvelope<D, I>,
    continuation_request: Option<WorthQueryDeclarationBridgeContinuationRequest>,
    continuation_family: Option<WorthQueryDeclarationBridgeContinuationFamily>,
    cause: WorthQueryDeclarationBridgeRoutingDenialCause,
    reason: &'static str,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationBridgeRoutingDenied<D, I>
{
    pub(crate) fn new(
        envelope: WorthQueryDeclarationEnvelope<D, I>,
        continuation_request: Option<WorthQueryDeclarationBridgeContinuationRequest>,
        continuation_family: Option<WorthQueryDeclarationBridgeContinuationFamily>,
        cause: WorthQueryDeclarationBridgeRoutingDenialCause,
    ) -> Self {
        Self {
            envelope,
            continuation_request,
            continuation_family,
            cause,
            reason: cause.reason(),
        }
    }

    pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
        &self.envelope
    }

    pub fn cause(&self) -> WorthQueryDeclarationBridgeRoutingDenialCause {
        self.cause
    }

    pub fn continuation_request(&self) -> Option<WorthQueryDeclarationBridgeContinuationRequest> {
        self.continuation_request
    }

    pub fn continuation_family(&self) -> Option<WorthQueryDeclarationBridgeContinuationFamily> {
        self.continuation_family
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }

    #[cfg(test)]
    pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
        self.envelope
    }
}

pub enum WorthQueryDeclarationBridgeRoutingTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationBridgeRoutingDeferred<D, I>),
    Denied(WorthQueryDeclarationBridgeRoutingDenied<D, I>),
    Failed(WorthQueryDeclarationBridgeRoutingFailed<D, I>),
}

pub enum WorthQueryDeclarationEntryBridgeRoutingError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(crate::application::WorthQueryDeclarationEntryEnvelopeError<D, I>),
    Routing(WorthQueryDeclarationBridgeRoutingTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationBridgeRoutingTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryBridgeRoutingError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

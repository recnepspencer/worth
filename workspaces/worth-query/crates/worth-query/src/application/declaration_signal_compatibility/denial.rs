use crate::application::{
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationInput,
    WorthQueryDeclarationSignalExecutionFamily, WorthQueryDomainEntryMarker,
};
use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationSignalCompatibilityDenialCause {
    EnvelopeNotCoveredForSignalCompatibility,
    SignalFamilyUnsupported,
    SignalBasisMismatch,
    MissingRequiredAspect,
    AspectConflict,
    AuthorityAspectGap,
    SignalCompatibilityMismatch,
    SignalExecutionFamilyUnavailable,
}

impl WorthQueryDeclarationSignalCompatibilityDenialCause {
    pub fn reason(self) -> &'static str {
        match self {
            Self::EnvelopeNotCoveredForSignalCompatibility => {
                "signal compatibility starts from covered envelope truth rather than non-success crossing posture"
            }
            Self::SignalFamilyUnsupported => {
                "this declaration family is not structurally signal-compatible"
            }
            Self::SignalBasisMismatch => {
                "the retained envelope truth does not currently satisfy the required basis-sensitive signal continuation posture"
            }
            Self::MissingRequiredAspect => {
                "the retained envelope publication does not expose the required signal dependency slice"
            }
            Self::AspectConflict => {
                "the retained envelope publication conflicts with the required signal dependency slice"
            }
            Self::AuthorityAspectGap => {
                "the retained envelope publication only partially covers the required signal dependency slice"
            }
            Self::SignalCompatibilityMismatch => {
                "the retained envelope truth and signal compatibility boundary expectations no longer agree"
            }
            Self::SignalExecutionFamilyUnavailable => {
                "this declaration family does not expose a supported signal compatibility contract"
            }
        }
    }
}

macro_rules! define_signal_terminal {
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

            pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
                self.envelope
            }
        }
    };
}

define_signal_terminal!(WorthQueryDeclarationSignalCompatibilityDeferred);
define_signal_terminal!(WorthQueryDeclarationSignalCompatibilityFailed);

pub struct WorthQueryDeclarationSignalCompatibilityDenied<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    envelope: WorthQueryDeclarationEnvelope<D, I>,
    execution_family: Option<WorthQueryDeclarationSignalExecutionFamily>,
    basis_families: Vec<BasisFamily>,
    cause: WorthQueryDeclarationSignalCompatibilityDenialCause,
    reason: &'static str,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationSignalCompatibilityDenied<D, I>
{
    pub(crate) fn new(
        envelope: WorthQueryDeclarationEnvelope<D, I>,
        execution_family: Option<WorthQueryDeclarationSignalExecutionFamily>,
        basis_families: Vec<BasisFamily>,
        cause: WorthQueryDeclarationSignalCompatibilityDenialCause,
    ) -> Self {
        Self {
            envelope,
            execution_family,
            basis_families,
            cause,
            reason: cause.reason(),
        }
    }

    pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
        &self.envelope
    }

    pub fn cause(&self) -> WorthQueryDeclarationSignalCompatibilityDenialCause {
        self.cause
    }

    pub fn execution_family(&self) -> Option<WorthQueryDeclarationSignalExecutionFamily> {
        self.execution_family
    }

    pub fn basis_families(&self) -> &[BasisFamily] {
        &self.basis_families
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }

    pub(crate) fn into_envelope(self) -> WorthQueryDeclarationEnvelope<D, I> {
        self.envelope
    }
}

pub enum WorthQueryDeclarationSignalCompatibilityTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationSignalCompatibilityDeferred<D, I>),
    Denied(WorthQueryDeclarationSignalCompatibilityDenied<D, I>),
    Failed(WorthQueryDeclarationSignalCompatibilityFailed<D, I>),
}

pub enum WorthQueryDeclarationEntrySignalCompatibilityError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(crate::application::WorthQueryDeclarationEntryEnvelopeError<D, I>),
    Compatibility(WorthQueryDeclarationSignalCompatibilityTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationSignalCompatibilityTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntrySignalCompatibilityError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

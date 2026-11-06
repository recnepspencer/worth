use crate::application::{
    WorthQueryDeclarationInput, WorthQueryDeclarationRouteIntent,
    WorthQueryDeclarationRoutePlanDenialCause, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::artifact::WorthQueryDeclarationReceipt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationReceiptDenialCause {
    MissingRoutePlan,
    UnsupportedReceiptKind,
    ReceiptMaterializationMismatch,
    RouteIntegrityMismatch,
}

impl WorthQueryDeclarationReceiptDenialCause {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingRoutePlan => "missing_route_plan",
            Self::UnsupportedReceiptKind => "unsupported_receipt_kind",
            Self::ReceiptMaterializationMismatch => "receipt_materialization_mismatch",
            Self::RouteIntegrityMismatch => "route_integrity_mismatch",
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::MissingRoutePlan => {
                "receipt construction requires retained route truth rather than loose declaration evidence"
            }
            Self::UnsupportedReceiptKind => {
                "this declaration route kind is not yet a supported Query receipt crossing"
            }
            Self::ReceiptMaterializationMismatch => {
                "the receipt boundary could not materialize a matching foundational receipt artifact"
            }
            Self::RouteIntegrityMismatch => {
                "the retained route proof and receipt boundary truth no longer agree"
            }
        }
    }
}

macro_rules! define_receipt_terminal {
    ($name:ident) => {
        pub struct $name<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> {
            receipt: WorthQueryDeclarationReceipt<D, I>,
            route_intent: Option<WorthQueryDeclarationRouteIntent>,
            reason: &'static str,
        }

        impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> $name<D, I> {
            pub(crate) fn new(
                receipt: WorthQueryDeclarationReceipt<D, I>,
                route_intent: Option<WorthQueryDeclarationRouteIntent>,
                reason: &'static str,
            ) -> Self {
                Self {
                    receipt,
                    route_intent,
                    reason,
                }
            }

            pub fn receipt(&self) -> &WorthQueryDeclarationReceipt<D, I> {
                &self.receipt
            }

            pub fn route_intent(&self) -> Option<WorthQueryDeclarationRouteIntent> {
                self.route_intent
            }

            pub fn reason(&self) -> &'static str {
                self.reason
            }
        }
    };
}

define_receipt_terminal!(WorthQueryDeclarationReceiptDeferred);
define_receipt_terminal!(WorthQueryDeclarationReceiptFailed);

pub struct WorthQueryDeclarationReceiptDenied<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    receipt: WorthQueryDeclarationReceipt<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
    route_cause: Option<WorthQueryDeclarationRoutePlanDenialCause>,
    receipt_cause: Option<WorthQueryDeclarationReceiptDenialCause>,
    reason: &'static str,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationReceiptDenied<D, I>
{
    pub(crate) fn from_route_cause(
        receipt: WorthQueryDeclarationReceipt<D, I>,
        route_intent: Option<WorthQueryDeclarationRouteIntent>,
        cause: WorthQueryDeclarationRoutePlanDenialCause,
    ) -> Self {
        Self {
            receipt,
            route_intent,
            route_cause: Some(cause),
            receipt_cause: None,
            reason: cause.reason(),
        }
    }

    pub(crate) fn from_receipt_cause(
        receipt: WorthQueryDeclarationReceipt<D, I>,
        route_intent: Option<WorthQueryDeclarationRouteIntent>,
        cause: WorthQueryDeclarationReceiptDenialCause,
    ) -> Self {
        Self {
            receipt,
            route_intent,
            route_cause: None,
            receipt_cause: Some(cause),
            reason: cause.reason(),
        }
    }

    pub fn receipt(&self) -> &WorthQueryDeclarationReceipt<D, I> {
        &self.receipt
    }

    pub fn route_intent(&self) -> Option<WorthQueryDeclarationRouteIntent> {
        self.route_intent
    }

    pub fn route_cause(&self) -> Option<WorthQueryDeclarationRoutePlanDenialCause> {
        self.route_cause
    }

    pub fn receipt_cause(&self) -> Option<WorthQueryDeclarationReceiptDenialCause> {
        self.receipt_cause
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

pub enum WorthQueryDeclarationReceiptTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationReceiptDeferred<D, I>),
    Denied(WorthQueryDeclarationReceiptDenied<D, I>),
    Failed(WorthQueryDeclarationReceiptFailed<D, I>),
}

pub enum WorthQueryDeclarationEntryReceiptError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(crate::application::WorthQueryDeclarationEntryProgressionError<D, I>),
    Receipt(WorthQueryDeclarationReceiptTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationReceiptTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryReceiptError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

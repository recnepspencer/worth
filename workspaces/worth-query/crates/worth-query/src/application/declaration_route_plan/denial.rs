use crate::application::{
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationEntryProgressionError,
    WorthQueryDeclarationFoundationalEvidence, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::{
    contract::WorthQueryDeclarationRouteContract, intent::WorthQueryDeclarationRouteIntent,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRoutePlanDenialCause {
    WrongAdmittedWorld,
    EvidenceMismatch,
    MissingRequiredAspect,
    AspectConflict,
    IntentRequired,
    IntentForbidden,
    IntentConflictsWithRouteContract,
    NoAllowedRoutes,
    ForbiddenRouteCombination,
}

impl WorthQueryDeclarationRoutePlanDenialCause {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WrongAdmittedWorld => "wrong_admitted_world",
            Self::EvidenceMismatch => "evidence_mismatch",
            Self::MissingRequiredAspect => "missing_required_aspect",
            Self::AspectConflict => "aspect_conflict",
            Self::IntentRequired => "intent_required",
            Self::IntentForbidden => "intent_forbidden",
            Self::IntentConflictsWithRouteContract => "intent_conflicts_with_route_contract",
            Self::NoAllowedRoutes => "no_allowed_routes",
            Self::ForbiddenRouteCombination => "forbidden_route_combination",
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::WrongAdmittedWorld => {
                "route planning requires retained proof from the same admitted world"
            }
            Self::EvidenceMismatch => {
                "route planning requires matching admitted progression and foundational evidence"
            }
            Self::MissingRequiredAspect => {
                "route planning requires route-relevant semantic slices that were not visibly covered"
            }
            Self::AspectConflict => {
                "route planning found conflicting route-relevant semantic slices in retained evidence"
            }
            Self::IntentRequired => {
                "the declaration route contract requires explicit caller route intent"
            }
            Self::IntentForbidden => {
                "the declaration route contract forbids caller-owned route narrowing"
            }
            Self::IntentConflictsWithRouteContract => {
                "the declaration route contract does not allow deferred routing here"
            }
            Self::NoAllowedRoutes => {
                "the declaration route contract admitted no concrete lower-authority routes"
            }
            Self::ForbiddenRouteCombination => {
                "the declaration route contract forbids plural lower-authority routing"
            }
        }
    }
}

macro_rules! define_route_terminal {
    ($name:ident) => {
        pub struct $name<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> {
            progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
            evidence: WorthQueryDeclarationFoundationalEvidence<D, I>,
            route_intent: Option<WorthQueryDeclarationRouteIntent>,
            route_contract: WorthQueryDeclarationRouteContract,
            reason: &'static str,
        }

        impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>> $name<D, I> {
            pub(crate) fn new(
                progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
                evidence: WorthQueryDeclarationFoundationalEvidence<D, I>,
                route_intent: Option<WorthQueryDeclarationRouteIntent>,
                route_contract: WorthQueryDeclarationRouteContract,
                reason: &'static str,
            ) -> Self {
                Self {
                    progressed,
                    evidence,
                    route_intent,
                    route_contract,
                    reason,
                }
            }

            pub fn reason(&self) -> &'static str {
                self.reason
            }

            pub fn route_intent(&self) -> Option<WorthQueryDeclarationRouteIntent> {
                self.route_intent
            }

            pub fn route_contract(&self) -> WorthQueryDeclarationRouteContract {
                self.route_contract
            }

            pub fn declaration_family_key(&self) -> &'static str {
                self.progressed.declaration_family_key()
            }

            pub fn progressed_declaration(
                &self,
            ) -> &WorthQueryAdmittedDeclarationProgression<D, I> {
                &self.progressed
            }

            pub fn foundational_evidence(
                &self,
            ) -> &WorthQueryDeclarationFoundationalEvidence<D, I> {
                &self.evidence
            }

            pub(crate) fn into_parts(
                self,
            ) -> (
                WorthQueryAdmittedDeclarationProgression<D, I>,
                WorthQueryDeclarationFoundationalEvidence<D, I>,
                Option<WorthQueryDeclarationRouteIntent>,
                WorthQueryDeclarationRouteContract,
                &'static str,
            ) {
                (
                    self.progressed,
                    self.evidence,
                    self.route_intent,
                    self.route_contract,
                    self.reason,
                )
            }
        }
    };
}

define_route_terminal!(WorthQueryDeclarationRoutePlanDeferred);
define_route_terminal!(WorthQueryDeclarationRoutePlanFailed);

pub struct WorthQueryDeclarationRoutePlanDenied<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    evidence: WorthQueryDeclarationFoundationalEvidence<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
    route_contract: WorthQueryDeclarationRouteContract,
    cause: WorthQueryDeclarationRoutePlanDenialCause,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationRoutePlanDenied<D, I>
{
    pub(crate) fn new(
        progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
        evidence: WorthQueryDeclarationFoundationalEvidence<D, I>,
        route_intent: Option<WorthQueryDeclarationRouteIntent>,
        route_contract: WorthQueryDeclarationRouteContract,
        cause: WorthQueryDeclarationRoutePlanDenialCause,
    ) -> Self {
        Self {
            progressed,
            evidence,
            route_intent,
            route_contract,
            cause,
        }
    }

    pub fn cause(&self) -> WorthQueryDeclarationRoutePlanDenialCause {
        self.cause
    }

    pub fn reason(&self) -> &'static str {
        self.cause.reason()
    }

    pub fn route_intent(&self) -> Option<WorthQueryDeclarationRouteIntent> {
        self.route_intent
    }

    pub fn route_contract(&self) -> WorthQueryDeclarationRouteContract {
        self.route_contract
    }

    pub fn declaration_family_key(&self) -> &'static str {
        self.progressed.declaration_family_key()
    }

    pub fn progressed_declaration(&self) -> &WorthQueryAdmittedDeclarationProgression<D, I> {
        &self.progressed
    }

    pub fn foundational_evidence(&self) -> &WorthQueryDeclarationFoundationalEvidence<D, I> {
        &self.evidence
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedDeclarationProgression<D, I>,
        WorthQueryDeclarationFoundationalEvidence<D, I>,
        Option<WorthQueryDeclarationRouteIntent>,
        WorthQueryDeclarationRouteContract,
        WorthQueryDeclarationRoutePlanDenialCause,
    ) {
        (
            self.progressed,
            self.evidence,
            self.route_intent,
            self.route_contract,
            self.cause,
        )
    }
}

pub enum WorthQueryDeclarationRoutePlanTerminalError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Deferred(WorthQueryDeclarationRoutePlanDeferred<D, I>),
    Denied(WorthQueryDeclarationRoutePlanDenied<D, I>),
    Failed(WorthQueryDeclarationRoutePlanFailed<D, I>),
}

pub enum WorthQueryDeclarationEntryRoutePlanError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(WorthQueryDeclarationEntryProgressionError<D, I>),
    RoutePlan(WorthQueryDeclarationRoutePlanTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationRoutePlanTerminalError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryRoutePlanError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}

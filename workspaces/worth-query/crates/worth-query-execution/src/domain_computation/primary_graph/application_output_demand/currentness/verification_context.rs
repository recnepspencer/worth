//! Resolve retained fact coordinates before any successful native mark can answer.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial as Indexed;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason as Requirement;
use crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason as Reason;

pub(super) enum VerificationContext<'a> {
    Native(&'a Fact),
    Indexed(&'a Fact, Indexed),
    Other(Option<Requirement>),
}
impl<'a> VerificationContext<'a> {
    pub(super) fn resolve(
        facts: &'a crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        requirement: Option<Requirement>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        match requirement {
            Some(Requirement::NativeFactRevisionUnavailable(ordinal)) => facts
                .get(ordinal)
                .map(Self::Native)
                .ok_or_else(|| missing(Reason::NativeFactRevisionUnavailable)),
            Some(Requirement::IndexedSelectionFactDenied(ordinal, cause)) => facts
                .get(ordinal)
                .map(|fact| Self::Indexed(fact, cause))
                .ok_or_else(|| missing(Reason::IndexedSelectionFactDenied(cause))),
            None
            | Some(
                Requirement::NativeRevisionUnavailable
                | Requirement::IndexedSelectionDenied(_)
                | Requirement::UnsupportedFact
                | Requirement::CheckpointRestore
                | Requirement::DifferentBranch
                | Requirement::BeforeReadBasis
                | Requirement::RetainedDeliveryGap
                | Requirement::MissingSettlement
                | Requirement::DeclaredChangeUnavailable
                | Requirement::RegistrationIncomplete
                | Requirement::ComputationSuperseded,
            ) => Ok(Self::Other(requirement)),
        }
    }
    pub(super) fn describe(
        self,
        error: WorthQueryOutputDemandDenial,
    ) -> WorthQueryOutputDemandDenial {
        let subject = match self {
            Self::Native(fact) => format!(
                "{}; native revision unavailable for: {}",
                error.subject(),
                fact.locator_identity()
            ),
            Self::Indexed(fact, denial) => format!(
                "{}; indexed selection reobservation denied: {denial:?}; fact: {}",
                error.subject(),
                fact.locator_identity()
            ),
            Self::Other(Some(reason)) => format!(
                "{}; output requires full verification: {reason:?}",
                error.subject()
            ),
            Self::Other(None) => return error,
        };
        // Keep the unique cleanup and all original evidence when adding description.
        error.with_subject(subject)
    }
}
fn missing(reason: Reason) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedVerificationFactUnavailable(reason),
        "retained verification requirement names an unavailable fact",
    )
}

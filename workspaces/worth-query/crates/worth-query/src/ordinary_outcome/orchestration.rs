use crate::application::{
    WorthQueryDeclarationEntryOrchestrationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationTerminalError, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker,
};

use super::{
    WorthQueryOrdinaryCheckedTopology, WorthQueryOrdinaryNextStep, WorthQueryOrdinaryOutcome,
    WorthQueryOrdinaryPosture, WorthQueryOrdinaryPostureKind,
};

#[cfg(test)]
pub(crate) fn ordinary_outcome_from_orchestration_terminal<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    terminal: WorthQueryDeclarationEntryOrchestrationTerminalError<D, I>,
) -> WorthQueryOrdinaryOutcome<crate::application::WorthQueryDeclarationEnvelope<D, I>> {
    match terminal {
        WorthQueryDeclarationEntryOrchestrationTerminalError::EvidenceIdentityDerivationDenied(
            cause,
        ) => WorthQueryOrdinaryOutcome::Refused(WorthQueryOrdinaryPosture::new(
            format!("evidence identity refused: {cause:?}"),
            WorthQueryOrdinaryPostureKind::Refused,
            WorthQueryOrdinaryNextStep::NarrowInput,
            WorthQueryOrdinaryCheckedTopology::identity_refused(cause),
        )),
        WorthQueryDeclarationEntryOrchestrationTerminalError::Deferred(value) => {
            WorthQueryOrdinaryOutcome::Deferred(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::Deferred,
                WorthQueryOrdinaryNextStep::RetryLater,
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    None,
                ),
            ))
        }
        WorthQueryDeclarationEntryOrchestrationTerminalError::Denied(value) => {
            WorthQueryOrdinaryOutcome::Denied(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::Denied,
                WorthQueryOrdinaryNextStep::InspectCheckedLane,
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    None,
                ),
            ))
        }
        WorthQueryDeclarationEntryOrchestrationTerminalError::Stale(value) => {
            WorthQueryOrdinaryOutcome::Stale(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::Stale,
                WorthQueryOrdinaryNextStep::RefreshBasis,
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    None,
                ),
            ))
        }
        WorthQueryDeclarationEntryOrchestrationTerminalError::RebindRequired(value) => {
            WorthQueryOrdinaryOutcome::RebindRequired(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::RebindRequired,
                WorthQueryOrdinaryNextStep::RebindContext,
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    None,
                ),
            ))
        }
        WorthQueryDeclarationEntryOrchestrationTerminalError::Failed(value) => {
            WorthQueryOrdinaryOutcome::Failed(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::Failed,
                WorthQueryOrdinaryNextStep::EscalateFailure,
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    None,
                ),
            ))
        }
        WorthQueryDeclarationEntryOrchestrationTerminalError::Refused(value) => {
            WorthQueryOrdinaryOutcome::Refused(WorthQueryOrdinaryPosture::new(
                value.reason(),
                WorthQueryOrdinaryPostureKind::Refused,
                refusal_next_step(value.refusal_class()),
                WorthQueryOrdinaryCheckedTopology::orchestration(
                    value.stop_stage(),
                    value.retained_digest().map(str::to_owned),
                    Some(value.refusal_class()),
                ),
            ))
        }
    }
}

#[cfg(test)]
fn refusal_next_step(
    class: WorthQueryDeclarationEntryOrchestrationRefusalClass,
) -> WorthQueryOrdinaryNextStep {
    match class {
        WorthQueryDeclarationEntryOrchestrationRefusalClass::UnsupportedAutomation => {
            WorthQueryOrdinaryNextStep::CheckSupport
        }
        WorthQueryDeclarationEntryOrchestrationRefusalClass::ExplicitIntentRequired => {
            WorthQueryOrdinaryNextStep::NarrowInput
        }
        WorthQueryDeclarationEntryOrchestrationRefusalClass::StrongerProofRequired => {
            WorthQueryOrdinaryNextStep::InspectProofLane
        }
        WorthQueryDeclarationEntryOrchestrationRefusalClass::AuthorityTransitionRequired => {
            WorthQueryOrdinaryNextStep::UseExplicitHandoff
        }
        WorthQueryDeclarationEntryOrchestrationRefusalClass::ExpensiveWorkNotAdmittedByDefault => {
            WorthQueryOrdinaryNextStep::UseExplicitHandoff
        }
        WorthQueryDeclarationEntryOrchestrationRefusalClass::PreparedButNotExecutedContinuation => {
            WorthQueryOrdinaryNextStep::UseExplicitHandoff
        }
    }
}

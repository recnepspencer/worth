use crate::application::{
    WorthQueryAdmittedDeclarationProgression,
    WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationInput,
    WorthQueryDeclarationRouteIntent, WorthQueryDeclarationRoutePlanChecked,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
#[cfg(test)]
use crate::application::{
    WorthQueryDeclarationRoutePlan, WorthQueryDeclarationRoutePlanTerminalError,
};

use super::common::route_orchestration_identity;
#[cfg(test)]
use super::common::route_terminal_from_checked;
use super::transcript::WorthQueryDeclarationRouteOrchestrationTranscript;
use crate::application::{
    worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle,
    WorthQueryDeclarationEntryProductChecked,
};

#[cfg(test)]
pub(crate) fn worth_query_declaration_route_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> Result<WorthQueryDeclarationRoutePlan<D, I>, WorthQueryDeclarationRoutePlanTerminalError<D, I>>
{
    match worth_query_checked_declaration_route_orchestration_from_progressed_on_handle(
        handle,
        progressed,
        route_intent,
    ) {
        WorthQueryDeclarationRoutePlanChecked::Planned(plan) => Ok(plan),
        other => Err(route_terminal_from_checked(other)),
    }
}

pub(crate) fn worth_query_checked_declaration_route_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> WorthQueryDeclarationRoutePlanChecked<D, I> {
    match worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
        handle,
        progressed,
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
        WorthQueryDeclarationEntryOrchestrationProduct::RoutePlan,
        route_intent,
    )
    .checked
    {
        WorthQueryDeclarationEntryProductChecked::RoutePlan(checked) => checked,
        _ => panic!("route orchestration must project the route product"),
    }
}

pub(crate) fn worth_query_declaration_route_orchestration_from_progressed_proof_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> WorthQueryDeclarationRouteOrchestrationTranscript<D, I> {
    let lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::ProofVisibleTranscript,
            WorthQueryDeclarationEntryOrchestrationProduct::RoutePlan,
            route_intent,
        );
    let checked = match lowered.checked {
        WorthQueryDeclarationEntryProductChecked::RoutePlan(checked) => checked,
        _ => panic!("route orchestration proof must project the route product"),
    };
    let outcome_identity = route_orchestration_identity(&checked);
    WorthQueryDeclarationRouteOrchestrationTranscript::new(
        lowered.plan,
        checked,
        lowered.step_records,
        outcome_identity,
    )
}

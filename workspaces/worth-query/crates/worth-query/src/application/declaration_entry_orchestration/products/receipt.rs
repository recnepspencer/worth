use crate::application::{
    WorthQueryAdmittedDeclarationProgression,
    WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationInput,
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationRouteIntent,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
#[cfg(test)]
use crate::application::{WorthQueryDeclarationReceipt, WorthQueryDeclarationReceiptTerminalError};

use super::common::receipt_orchestration_identity;
#[cfg(test)]
use super::common::receipt_terminal_from_checked;
use super::transcript::WorthQueryDeclarationReceiptOrchestrationTranscript;
use crate::application::{
    worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle,
    WorthQueryDeclarationEntryProductChecked,
};

#[cfg(test)]
pub(crate) fn worth_query_declaration_receipt_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> Result<WorthQueryDeclarationReceipt<D, I>, WorthQueryDeclarationReceiptTerminalError<D, I>> {
    match worth_query_checked_declaration_receipt_orchestration_from_progressed_on_handle(
        handle,
        progressed,
        route_intent,
    ) {
        WorthQueryDeclarationReceiptChecked::Issued(receipt) => Ok(receipt),
        other => Err(receipt_terminal_from_checked(other)),
    }
}

pub(crate) fn worth_query_checked_declaration_receipt_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> WorthQueryDeclarationReceiptChecked<D, I> {
    match worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
        handle,
        progressed,
        WorthQueryDeclarationEntryOrchestrationExposureLevel::Checked,
        WorthQueryDeclarationEntryOrchestrationArtifactPolicy::CheckedOutcomeOnly,
        WorthQueryDeclarationEntryOrchestrationProduct::Receipt,
        route_intent,
    )
    .checked
    {
        WorthQueryDeclarationEntryProductChecked::Receipt(checked) => checked,
        _ => panic!("receipt orchestration must project the receipt product"),
    }
}

pub(crate) fn worth_query_declaration_receipt_orchestration_from_progressed_proof_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> WorthQueryDeclarationReceiptOrchestrationTranscript<D, I> {
    let lowered =
        worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle(
            handle,
            progressed,
            WorthQueryDeclarationEntryOrchestrationExposureLevel::ProofVisible,
            WorthQueryDeclarationEntryOrchestrationArtifactPolicy::ProofVisibleTranscript,
            WorthQueryDeclarationEntryOrchestrationProduct::Receipt,
            route_intent,
        );
    let checked = match lowered.checked {
        WorthQueryDeclarationEntryProductChecked::Receipt(checked) => checked,
        _ => panic!("receipt orchestration proof must project the receipt product"),
    };
    let outcome_identity = receipt_orchestration_identity(&checked);
    WorthQueryDeclarationReceiptOrchestrationTranscript::new(
        lowered.plan,
        checked,
        lowered.step_records,
        outcome_identity,
    )
}

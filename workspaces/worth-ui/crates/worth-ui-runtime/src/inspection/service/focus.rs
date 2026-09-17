pub(crate) fn why_focus_moved(
    owner: Option<&crate::runtime::focus::UiFocusRuntimeState>,
) -> Option<worth_ui_inspection::UiFocusMovedInspectionSummary> {
    owner?.last_transition().map(project_transition)
}

pub(crate) fn why_focus_restoration_failed(
    owner: Option<&crate::runtime::focus::UiFocusRuntimeState>,
) -> Option<worth_ui_inspection::UiFocusRestorationFailedInspectionSummary> {
    let receipt = owner?.last_restoration_failure()?;
    Some(
        worth_ui_inspection::UiFocusRestorationFailedInspectionSummary::new(
            source(receipt),
            worth_ui_inspection::UiFocusRestorationFailureInspectionReason::NoEligibleParticipant,
            worth_ui_inspection::UiRuntimeServiceInspectionCost::latest_record(1, 1),
        ),
    )
}

fn project_transition(
    receipt: crate::runtime::focus::UiFocusTransitionReceipt,
) -> worth_ui_inspection::UiFocusMovedInspectionSummary {
    worth_ui_inspection::UiFocusMovedInspectionSummary::new(
        source(receipt),
        receipt
            .previous()
            .map(|focus| focus.mounted_instance().diagnostic_value()),
        receipt
            .current()
            .map(|focus| focus.mounted_instance().diagnostic_value()),
        match receipt.cause() {
            #[cfg(any(test, feature = "certification-support"))]
            crate::runtime::focus::UiFocusCause::Direct => {
                worth_ui_inspection::UiFocusMoveInspectionCause::Direct
            }
            crate::runtime::focus::UiFocusCause::KeyboardTraversal => {
                worth_ui_inspection::UiFocusMoveInspectionCause::KeyboardTraversal
            }
            crate::runtime::focus::UiFocusCause::RovingMovement => {
                worth_ui_inspection::UiFocusMoveInspectionCause::RovingMovement
            }
            crate::runtime::focus::UiFocusCause::PortalInitial => {
                worth_ui_inspection::UiFocusMoveInspectionCause::PortalInitial
            }
            crate::runtime::focus::UiFocusCause::PortalRestoration => {
                worth_ui_inspection::UiFocusMoveInspectionCause::PortalRestoration
            }
            crate::runtime::focus::UiFocusCause::RebindPreserved => {
                worth_ui_inspection::UiFocusMoveInspectionCause::RebindPreserved
            }
            crate::runtime::focus::UiFocusCause::RebindFallback => {
                worth_ui_inspection::UiFocusMoveInspectionCause::RebindFallback
            }
        },
        match receipt.outcome() {
            crate::runtime::focus::UiFocusOutcome::Moved => {
                worth_ui_inspection::UiFocusMoveInspectionOutcome::Moved
            }
            crate::runtime::focus::UiFocusOutcome::Unchanged => {
                worth_ui_inspection::UiFocusMoveInspectionOutcome::Unchanged
            }
            crate::runtime::focus::UiFocusOutcome::Cleared => {
                worth_ui_inspection::UiFocusMoveInspectionOutcome::Cleared
            }
            crate::runtime::focus::UiFocusOutcome::NoEligibleParticipant => {
                worth_ui_inspection::UiFocusMoveInspectionOutcome::NoEligibleParticipant
            }
        },
        receipt.participants_visited(),
        worth_ui_inspection::UiRuntimeServiceInspectionCost::latest_record(1, 1),
    )
}

fn source(
    receipt: crate::runtime::focus::UiFocusTransitionReceipt,
) -> worth_ui_inspection::UiRuntimeServiceInspectionSource {
    worth_ui_inspection::UiRuntimeServiceInspectionSource::new(
        worth_ui_inspection::UiRuntimeServiceInspectionFamily::Focus,
        receipt
            .current()
            .or(receipt.previous())
            .map(|focus| focus.mounted_instance().diagnostic_value()),
        receipt.revision(),
    )
}

use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui::facade::intent::{UiIntentExecutionTransitionPosture, UiIntentProductOutcome};
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentPostureObservation;

use super::super::super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};

#[derive(Clone, Copy)]
pub(super) enum PlatformPulseIntentConsequenceKind {
    QueryAction,
    Portal,
}

pub(in crate::native_application) struct PlatformPulsePendingIntentConsequence {
    pub(super) attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
    pub(super) idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    pub(super) kind: PlatformPulseIntentConsequenceKind,
}

impl PlatformPulseApplicationRuntime {
    pub(super) fn finish_intent_consequence_publication(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
        kind: PlatformPulseIntentConsequenceKind,
        receipt: worth_ui::facade::intent::UiIntentConsequencePublicationReceipt,
    ) -> bool {
        let focus = receipt.focus_publication();
        let rebind = receipt.into_rebind();
        match kind {
            PlatformPulseIntentConsequenceKind::QueryAction => {
                if focus.is_some() {
                    self.fail_intent_settlement(
                        "query consequence unexpectedly carried a semantic Focus publication",
                    );
                    return false;
                }
                self.finish_action_query_publication(shell, attempt, idempotency, rebind)
            }
            PlatformPulseIntentConsequenceKind::Portal => {
                let Some(focus) = focus else {
                    self.fail_intent_settlement(
                        "portal consequence omitted its semantic Focus publication",
                    );
                    return false;
                };
                self.finish_portal_publication(shell, attempt, idempotency, rebind, focus)
            }
        }
    }

    fn finish_portal_publication(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
        focus: worth_ui::facade::app::UiSemanticFocusPublicationReceipt,
    ) -> bool {
        let Some(mounted) = receipt.mounted_publication() else {
            self.fail_intent_settlement("portal consequence omitted mounted publication");
            return false;
        };
        let posture = PlatformPulseIntentPostureObservation::completed(attempt, idempotency);
        let command_transition = super::super::latest_command_transition(shell);
        if let Err(error) =
            self.publisher
                .intent_posture_published(posture, mounted, command_transition)
        {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        if let Err(error) = self.publisher.semantic_focus_published(focus, mounted) {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        if self
            .intent_evidence_index
            .retire_execution(attempt, idempotency)
            .is_none()
        {
            self.fail_intent_settlement("portal consequence lost its retained intent evidence");
            return false;
        }
        if let Err(denial) = self.visual_identity.refresh_after_presentation_replacement(
            shell,
            self.presentation_tick,
            std::time::Instant::now(),
        ) {
            self.fail_visual_identity(denial);
            return false;
        }
        self.refresh_product_story(shell)
    }
}

pub(super) fn consequence_kind(
    posture: UiIntentExecutionTransitionPosture,
) -> Option<PlatformPulseIntentConsequenceKind> {
    let UiIntentExecutionTransitionPosture::Completed { outcome } = posture else {
        return None;
    };
    if outcome == worth_ui_platform_pulse::intent::PlatformPulseActionOutcome::SCHEMA {
        Some(PlatformPulseIntentConsequenceKind::QueryAction)
    } else if worth_ui_platform_pulse::product_world::platform_pulse_portal_story_transition(
        outcome,
    )
    .is_some()
    {
        Some(PlatformPulseIntentConsequenceKind::Portal)
    } else {
        None
    }
}

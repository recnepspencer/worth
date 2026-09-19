use super::{WorthUiNativeApplicationShell, WorthUiNativeIntentPosture};

#[must_use]
pub enum WorthUiNativeIntentTerminalPostureOutcome {
    NotTerminal,
    RecoveryRetained,
    Prepared(WorthUiNativeIntentPosture),
    MissingExecutionBasis,
    PostureIdentityExhausted,
}

impl WorthUiNativeApplicationShell {
    /// Translate a framework-owned terminal execution transition into mounted
    /// posture authority without exposing its retained graph/target basis.
    pub fn prepare_native_intent_terminal_posture(
        &self,
        transition: &crate::facade::intent::UiIntentExecutionTransition,
    ) -> WorthUiNativeIntentTerminalPostureOutcome {
        use crate::facade::intent::UiIntentExecutionTransitionPosture as Posture;

        let kind = match transition.posture() {
            Posture::RejectedBeforeEffect { .. }
            | Posture::FailedBeforeEffect { .. }
            | Posture::TimedOutBeforeEffect { .. } => {
                crate::fact_contract::UiIntentPostureKind::Denied
            }
            Posture::CancelledBeforeEffect { .. } => {
                crate::fact_contract::UiIntentPostureKind::Cancelled
            }
            Posture::Partial { .. } | Posture::Indeterminate { .. } => {
                return WorthUiNativeIntentTerminalPostureOutcome::RecoveryRetained;
            }
            Posture::Started
            | Posture::PendingBeforeEffect
            | Posture::PendingEffectMayHaveBegun
            | Posture::Completed { .. } => {
                return WorthUiNativeIntentTerminalPostureOutcome::NotTerminal;
            }
        };
        let Some(basis) = transition.posture_basis() else {
            return WorthUiNativeIntentTerminalPostureOutcome::MissingExecutionBasis;
        };
        let reference = crate::fact_contract::UiIntentPostureReference::Attempt {
            attempt: transition.attempt(),
            idempotency: transition.idempotency(),
        };
        let Some((observation, commit)) =
            self.session
                .intent_postures
                .prepare(basis.graph_node, basis.target, reference, kind)
        else {
            return WorthUiNativeIntentTerminalPostureOutcome::PostureIdentityExhausted;
        };
        WorthUiNativeIntentTerminalPostureOutcome::Prepared(WorthUiNativeIntentPosture::new(
            observation,
            commit,
            kind,
        ))
    }
}

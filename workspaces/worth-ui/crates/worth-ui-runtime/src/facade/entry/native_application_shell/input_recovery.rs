use crate::facade::entry::native_observation_settlement::UiNativeObservationIngressSettlement;
use crate::runtime::interaction::{
    UiHostInteractionIngressOutcome, UiInteractionLifecycleStopReason,
    UiInteractionObservationDenial,
};

#[cfg(all(test, feature = "certification-support"))]
#[path = "input_recovery/tests.rs"]
mod tests;

impl super::WorthUiNativeApplicationShell {
    pub(crate) fn cancel_exhausted_native_input(
        &mut self,
        grant: &worth_ui_host_native::UiNativeInputRecoveryGrant,
    ) -> Result<UiNativeObservationIngressSettlement, ()> {
        if self.session.host_session.identity().as_u64() != grant.host_session() {
            return Err(());
        }
        let previous_input = self.session.interaction.active_input_binding();
        let settlement = self
            .session
            .interaction
            .cancel_all(UiInteractionLifecycleStopReason::ObservationInvalid);
        self.session.clear_displaced_input_recipient(previous_input);
        self.session.settle_scroll_after_attention_loss();
        Ok(UiNativeObservationIngressSettlement::from_outcomes(
            vec![UiHostInteractionIngressOutcome::Denied(UiInteractionObservationDenial::new(
                crate::facade::observation_report::UiHostObservationReportDenial::HostInputRetentionExhausted,
                settlement,
            ))].into_boxed_slice(), Default::default(),
        ))
    }
}

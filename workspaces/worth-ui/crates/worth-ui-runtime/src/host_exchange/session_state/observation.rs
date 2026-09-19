use super::WorthUiHostExchangeSessionState;

impl WorthUiHostExchangeSessionState {
    pub(crate) fn retire_delivered_observation_batch(
        &mut self,
        core: worth_ui_host_contract::UiHostObservationCanonicalCore,
    ) {
        self.observations.retire_delivered_batch(core);
    }

    pub(crate) fn validate_observation_batch(
        &mut self,
        batch: worth_ui_host_contract::UiHostObservationBatch,
        host_session: u64,
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        mounted: crate::mounting::UiMountedObservationValidationBasis<'_>,
    ) -> crate::facade::observation_report::UiHostObservationReportOutcome {
        self.observations.validate(
            batch,
            crate::host_exchange::observation_report_validation::UiHostObservationValidationContext {
                host_session,
                protocol,
                mounted,
            },
        )
    }

    pub(crate) fn retained_observation_report_count(&self) -> usize {
        self.observations.retained_report_count()
    }

    pub(crate) fn retained_observation_byte_count(&self) -> usize {
        self.observations.retained_byte_count()
    }

    pub(crate) fn quarantined_observation_batch_count(&self) -> usize {
        self.observations.quarantined_batch_count()
    }

    pub(crate) fn quarantined_observation_byte_count(&self) -> usize {
        self.observations.quarantined_byte_count()
    }

    pub(crate) fn observation_work_report(
        &self,
    ) -> crate::facade::observation_report::UiHostObservationWorkReport {
        self.observations.work_report()
    }

    pub(crate) fn record_rejected_frame(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) {
        self.observations.record_rejected_frame(frame);
    }

    pub(crate) fn record_never_presented_frame(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) {
        self.observations.record_never_presented_frame(frame);
    }

    pub(crate) fn record_indeterminate_frame(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        bindings: &[worth_ui_host_contract::UiSurfaceBindingGeneration],
    ) {
        self.observations
            .record_indeterminate_frame(frame, bindings);
    }

    pub(crate) fn record_presented_frame(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) {
        self.observations.record_presented_frame(frame);
    }

    pub(crate) fn observation_retention_snapshot(
        &self,
    ) -> crate::host_exchange::observation_report_validation::UiHostObservationRetentionSnapshot
    {
        self.observations.retention_snapshot()
    }
}

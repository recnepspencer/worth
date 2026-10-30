use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn host_measurement_ingress(
        &self,
    ) -> crate::facade::measurement_exchange::WorthUiHostMeasurementIngress {
        self.host_exchange.measurement_ingress()
    }

    pub(crate) fn begin_host_measurement(
        &mut self,
        intent: crate::facade::measurement_exchange::UiHostMeasurementIntent,
        now: u64,
    ) -> crate::facade::measurement_exchange::UiHostMeasurementOutcome {
        if let Some(binding) = intent.binding()
            && (self.mounted.validate_binding(binding).is_err()
                || self
                    .mounted
                    .observation_validation_basis()
                    .binding_requires_reconciliation(binding))
        {
            return denied(
                crate::facade::measurement_exchange::UiHostMeasurementDenial::UnknownSurfaceBinding,
            );
        }
        let current = self.measurement_truth(true);
        self.host_exchange.begin_measurement(
            intent,
            current,
            self.host_session.capability_report(),
            now,
        )
    }

    pub(crate) fn complete_host_measurement(
        &mut self,
        observation: worth_ui_host_contract::UiHostMeasurementObservation,
        now: u64,
    ) -> crate::facade::measurement_exchange::UiHostMeasurementOutcome {
        let binding_is_live = self
            .host_exchange
            .pending_measurement_binding(observation.request_identity())
            .is_none_or(|binding| {
                binding.is_none_or(|binding| {
                    self.mounted.validate_binding(binding).is_ok()
                        && !self
                            .mounted
                            .observation_validation_basis()
                            .binding_requires_reconciliation(binding)
                })
            });
        let current = self.measurement_truth(binding_is_live);
        self.host_exchange
            .complete_measurement(observation, current, now)
    }

    pub(crate) fn cancel_host_measurement(
        &mut self,
        identity: worth_ui_host_contract::UiMeasurementRequestIdentity,
    ) -> crate::facade::measurement_exchange::UiHostMeasurementOutcome {
        self.host_exchange.cancel_measurement(identity)
    }

    pub(crate) fn expire_host_measurements(
        &mut self,
        now: u64,
    ) -> Box<[crate::facade::measurement_exchange::UiHostMeasurementOutcome]> {
        self.host_exchange.expire_measurements(now)
    }

    pub(crate) fn pending_host_measurement_count(&self) -> usize {
        self.host_exchange.pending_measurement_count()
    }

    pub(crate) fn pending_host_measurement_bytes(&self) -> usize {
        self.host_exchange.pending_measurement_bytes()
    }

    pub(crate) fn complete_enqueued_host_measurements(
        &mut self,
    ) -> Box<[crate::facade::measurement_exchange::UiHostMeasurementOutcome]> {
        self.host_exchange
            .drain_measurement_ingress()
            .into_iter()
            .map(|completion| {
                self.complete_host_measurement(
                    completion.observation().clone(),
                    completion.observed_at(),
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn measurement_truth(
        &self,
        pending_binding_is_live: bool,
    ) -> crate::host_exchange::measurement_admission::UiHostMeasurementCurrentTruth {
        crate::host_exchange::measurement_admission::UiHostMeasurementCurrentTruth::new(
            self.host_session.identity().as_u64(),
            self.application.allocation_truth_revision(),
            self.host_session
                .output_adapter()
                .measurement_environment_report(),
            self.host_session
                .capability_report()
                .observation_generation(),
            pending_binding_is_live,
        )
    }
}

fn denied(
    denial: crate::facade::measurement_exchange::UiHostMeasurementDenial,
) -> crate::facade::measurement_exchange::UiHostMeasurementOutcome {
    crate::facade::measurement_exchange::UiHostMeasurementOutcome::Denied(denial)
}

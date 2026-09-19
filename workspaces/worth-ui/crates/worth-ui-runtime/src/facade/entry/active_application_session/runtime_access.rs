impl super::WorthUiActiveApplicationSession {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn lookup_consumed_fact_for_certification(
        &self,
        fact: &crate::fact_contract::UiProducedFact,
    ) -> Result<crate::graph::UiGraphFactLookupReceipt, crate::graph::UiGraphFactLookupDenial> {
        self.application.lookup_consumed_fact(fact)
    }

    pub(crate) fn source_event_ingress(
        &self,
        provider: crate::runtime::WorthUiSourceProvider,
    ) -> crate::runtime::WorthUiSourceEventIngress {
        self.application.source_event_ingress(provider)
    }

    pub(crate) fn viewport_measurement_witnesses(
        &self,
    ) -> Box<[crate::evidence::UiHostMeasurementAuthorityWitness]> {
        self.application.viewport_measurement_witnesses()
    }

    pub(crate) fn host_measurement_capability(
        &self,
    ) -> crate::facade::WorthUiHostMeasurementCapability {
        self.host_session.measurement_capability()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_runtime(&self) -> crate::runtime::WorthUiActiveRuntimeObservation {
        self.application.inspect_active_runtime()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_query_state_residue(
        &self,
    ) -> crate::runtime::WorthUiStateQueryResidueScan {
        self.application.inspect_query_state_residue()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn refresh_query_change_for_certification(
        &mut self,
        request: worth_ui_query_binding::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome,
        worth_ui_query_binding::WorthUiOperationLiveRefreshError,
    > {
        self.application
            .refresh_query_change_for_certification(request)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn query_change_state_for_certification(
        &self,
        reference: &worth_ui_query_binding::WorthUiInstalledQueryBindingReference,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveChangeObservation,
        worth_ui_query_binding::WorthUiQueryViewExecutionEvidenceDenial,
    > {
        self.application
            .query_change_state_for_certification(reference)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn measurement_basis_sources_for_certification(
        &self,
    ) -> Box<[crate::declaration::UiDeclaredMeasurementBasisSource]> {
        self.application
            .measurement_basis_sources_for_certification()
    }
}

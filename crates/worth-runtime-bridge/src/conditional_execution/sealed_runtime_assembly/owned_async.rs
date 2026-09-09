use super::*;

impl BridgeSealedRuntimeAssembly {
    pub fn installed_owned_async_declaration(
        &self,
        identity: &str,
    ) -> Option<crate::facade::LoweredBridgeAsyncSourceDeclaration> {
        self.runtime.installed_owned_async_declaration(identity)
    }

    /// Admit through Bridge's lower Signal/truth basis contract.
    ///
    /// Runtime World qualification is established by the composition owner
    /// before calling this operation; this operation claims only Bridge and
    /// Signal request authority.
    #[doc(hidden)]
    pub fn admit_owned_async_request_identity(
        &self,
        declaration: &crate::facade::LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: crate::facade::BridgeAsyncRequestTruthViewBasis,
    ) -> Result<
        super::super::BridgeOwnedAsyncRequestAdmission,
        crate::facade::BridgeAsyncRequestIdentityRejection,
    > {
        self.runtime
            .admit_owned_async_request_identity(declaration, signal_basis, truth_basis)
    }

    pub fn validate_owned_async_completion_envelope(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<
        crate::facade::ValidatedBridgeAsyncCompletionEnvelope,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .validate_owned_async_completion_envelope(request, raw)
    }

    pub fn admit_owned_async_completion(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        validated: &crate::facade::ValidatedBridgeAsyncCompletionEnvelope,
    ) -> Result<
        super::super::BridgeOwnedAsyncCompletionAdmission,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .admit_owned_async_completion(request, validated)
    }

    pub fn admit_owned_async_effects_indeterminate(
        &self,
        observation: super::super::BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<
        super::super::BridgeOwnedAsyncCompletionAdmission,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .admit_owned_async_effects_indeterminate(observation)
    }

    pub fn retire_owned_async_request(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<bool, crate::facade::BridgeAsyncCompletionRejection> {
        self.runtime.retire_owned_async_request(request)
    }

    pub fn advance_owned_async_request_to_timeout(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        coordinate: u64,
    ) -> Result<
        super::super::BridgeOwnedAsyncTimeoutAdmission,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .advance_owned_async_request_to_timeout(request, coordinate)
    }

    pub fn schedule_owned_async_retry(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        timeout: &super::super::BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<
        super::super::BridgeOwnedAsyncRetrySchedule,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime.schedule_owned_async_retry(request, timeout)
    }

    pub fn advance_owned_async_retry(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        schedule: &super::super::BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<
        super::super::BridgeOwnedAsyncRetryAdmission,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .advance_owned_async_retry(request, schedule, coordinate)
    }

    /// Revalidate through Bridge's lower Signal/truth basis contract.
    #[doc(hidden)]
    pub fn revalidate_owned_async_request(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: crate::facade::BridgeAsyncRequestTruthViewBasis,
    ) -> Result<
        super::super::BridgeOwnedAsyncRevalidationAdmission,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .revalidate_owned_async_request(request, signal_basis, truth_basis)
    }

    pub fn owned_async_active_request_count(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, crate::facade::BridgeAsyncCompletionRejection> {
        self.runtime.owned_async_active_request_count(request)
    }

    pub fn admit_owned_async_supersession<'a>(
        &self,
        prior: &'a super::super::BridgeOwnedAsyncRequestAdmission,
        displacing: &'a super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        super::super::BridgeOwnedAsyncSupersessionAdmission<'a>,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .admit_owned_async_supersession(prior, displacing)
    }

    pub fn validate_owned_async_request_occurrence<'a>(
        &self,
        request: &'a super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        &'a crate::facade::AdmittedBridgeAsyncRequestIdentity,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime
            .validate_owned_async_request_occurrence(request)
    }

    pub fn order_owned_async_completion_report(
        &self,
        completion: &super::super::BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<
        crate::facade::BridgeMixedCauseOrdering,
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime.order_owned_async_completion_report(completion)
    }

    pub fn owned_async_declaration_count(&self) -> usize {
        self.runtime.owned_async_declaration_count()
    }
}

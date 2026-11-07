//! Read-only current-product control, independent of navigation and publication.
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::authorization::{
    admitted_request::AdmittedRequestCurrentness, WorthQueryAdmissionLapse,
    WorthQueryOperationScopeEntityBinding,
};
use crate::domain_computation::primary_graph::{
    provider::WorthQueryApplicationBranchCommitLane, WorthQueryAdvancementDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{branch::RelationalBranchBasisDenial, runtime::RelationalRuntime};
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};
pub(super) mod head;

#[derive(Debug)]
pub(in crate::domain_computation) enum CurrentReadDenial {
    ForeignPhase,
    HeadDiverged,
    NativeBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    Stopped(WorthQueryAdvancementDenial),
    Authorization(WorthQueryAdmissionLapse),
    Busy,
}

/// Borrows genuine admission; it grants neither a historical basis nor a write.
#[derive(Clone, Copy)]
pub(in crate::domain_computation) struct CurrentReadControl<'a> {
    current: &'a WorthQueryProductObservationLease,
    request: &'a AdmittedRequestCurrentness,
    principal: &'a WorthQueryOperationScopeEntityBinding,
}
impl<'a> CurrentReadControl<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        current: &'a WorthQueryProductObservationLease,
        request: &'a AdmittedRequestCurrentness,
        principal: &'a WorthQueryOperationScopeEntityBinding,
    ) -> Self {
        Self {
            current,
            request,
            principal,
        }
    }
    pub(in crate::domain_computation) fn current(&self) -> &WorthQueryProductObservationLease {
        self.current
    }
    pub(in crate::domain_computation) fn admits_request(
        &self,
        request: &WorthQueryRequestScope,
    ) -> bool {
        self.request.same_request(request)
    }
    pub(in crate::domain_computation) fn admits_principal(
        &self,
        principal: &WorthQueryOperationScopeEntityBinding,
    ) -> bool {
        self.principal == principal
    }
    pub(in crate::domain_computation) fn request_lapse(&self) -> Option<WorthQueryAdmissionLapse> {
        self.request.lapse()
    }
    pub(super) fn admitted_request(&self) -> &AdmittedRequestCurrentness {
        self.request
    }
    pub(in crate::domain_computation::primary_graph) fn confirm_in_execution<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, CurrentReadDenial> {
        self.confirm_in_execution_preserving(execution, runtime, value)
            .map_err(|(_, cause)| cause)
    }
    pub(in crate::domain_computation::primary_graph) fn confirm_in_execution_preserving<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, (T, CurrentReadDenial)> {
        deliver(head::confirm_in_execution(
            runtime,
            execution,
            self.current,
            Some(self.request),
            value,
        ))
    }
    /// Native is already borrowed. Never block on an earlier commit lane.
    pub(in crate::domain_computation::primary_graph) fn confirm_in_borrowed<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        lane: &WorthQueryApplicationBranchCommitLane,
        native: &RelationalRuntime,
        value: T,
    ) -> Result<T, CurrentReadDenial> {
        self.confirm_in_borrowed_preserving(execution, runtime, lane, native, value)
            .map_err(|(_, cause)| cause)
    }
    pub(in crate::domain_computation::primary_graph) fn confirm_in_borrowed_preserving<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        lane: &WorthQueryApplicationBranchCommitLane,
        native: &RelationalRuntime,
        value: T,
    ) -> Result<T, (T, CurrentReadDenial)> {
        let result = {
            let Some(coordination) = lane.try_enter() else {
                return Err((value, CurrentReadDenial::Busy));
            };
            head::confirm_in_native(
                runtime,
                execution,
                self.current,
                Some(self.request),
                &coordination,
                native,
                value,
            )
        };
        deliver(result)
    }
}
fn deliver<T>(
    result: Result<(T, Option<WorthQueryAdmissionLapse>), (T, CurrentReadDenial)>,
) -> Result<T, (T, CurrentReadDenial)> {
    match result {
        Ok((value, None)) => Ok(value),
        Ok((value, Some(lapse))) => Err((value, CurrentReadDenial::Authorization(lapse))),
        Err(cause) => Err(cause),
    }
}

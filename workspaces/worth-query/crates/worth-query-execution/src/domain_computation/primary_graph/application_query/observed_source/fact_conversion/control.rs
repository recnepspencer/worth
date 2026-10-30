//! Attempt-local request borrowing, never retained in a pending expectation.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct SourceFactExpansionControl<
    'request,
    'authority,
> {
    execution: Option<ExecutionRequest<'request, 'authority>>,
    storage: StorageControl<'request, 'authority>,
}
impl<'request, 'authority> SourceFactExpansionControl<'request, 'authority> {
    pub(in crate::domain_computation::primary_graph) fn in_request(
        execution: ExecutionRequest<'request, 'authority>,
        request: Option<&'request WorthQueryRequestScope>,
    ) -> Self {
        Self {
            execution: Some(execution),
            storage: StorageControl::new(execution.allocation_policy(), request),
        }
    }
    pub(super) fn legacy(storage: StorageControl<'request, 'authority>) -> Self {
        Self {
            execution: None,
            storage,
        }
    }
    pub(super) fn storage(self) -> StorageControl<'request, 'authority> {
        self.storage
    }
    pub(super) fn admit<Q>(
        self,
        source: &WorthQueryObservedSource<Q>,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        if self.execution.is_none() {
            source.require_supported_fact_expansion(subject)?;
        }
        self.poll(subject)
    }
    pub(super) fn poll(self, subject: &str) -> Result<(), WorthQuerySourceExpectationDenial> {
        let denied =
            |cause| WorthQuerySourceExpectationDenial::source_retention_denied(subject, cause);
        if let Some(execution) = self.execution {
            execution.check_live().map_err(|cause| {
                denied(StoreDenial::ExecutionRequest(WorkCeilingDenial::Stopped(
                    cause,
                )))
            })?;
        }
        self.storage.check_live().map_err(denied)
    }
}
impl<'request, 'authority> SourceFactExpansionControl<'request, 'authority> {
    pub(in crate::domain_computation::primary_graph) fn for_phase<
        S: worth_query_installation::facade::ApplicationSchema,
    >(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &'request WorthQueryAdvancementPhase<'authority>,
        request: Option<&'request WorthQueryRequestScope>,
        subject: &str,
    ) -> Result<Self, WorthQuerySourceExpectationDenial> {
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|_| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::ForeignOperation,
                    subject,
                )
            })?;
        Ok(Self::in_request(execution, request))
    }
}

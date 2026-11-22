//! Attempt-local request borrowing, never retained in a pending expectation.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    WorthQueryAdvancementPhase,
};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;

#[derive(Clone, Copy)]
pub struct SourceFactExpansionControl<'request, 'authority> {
    execution: Option<ExecutionRequest<'request, 'authority>>,
    storage: StorageControl<'request, 'authority>,
}
impl<'request, 'authority> SourceFactExpansionControl<'request, 'authority> {
    pub fn in_request(
        execution: ExecutionRequest<'request, 'authority>,
        request: Option<&'request WorthQueryRequestScope>,
    ) -> Self {
        Self {
            execution: Some(execution),
            storage: StorageControl::new(execution.allocation_policy(), request)
                .with_execution(execution),
        }
    }
    pub fn legacy(storage: StorageControl<'request, 'authority>) -> Self {
        Self {
            execution: None,
            storage,
        }
    }
    pub fn storage(self) -> StorageControl<'request, 'authority> {
        self.storage
    }
    pub fn admit<Q>(
        self,
        source: &WorthQueryObservedSource<Q>,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        if self.execution.is_none() {
            source.require_supported_fact_expansion(subject)?;
        }
        self.poll(subject)
    }
    pub fn poll(self, subject: &str) -> Result<(), WorthQuerySourceExpectationDenial> {
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
    pub fn for_product_phase(
        product_runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
        phase: &'request WorthQueryAdvancementPhase<'authority>,
        request: Option<&'request WorthQueryRequestScope>,
        subject: &str,
    ) -> Result<Self, WorthQuerySourceExpectationDenial> {
        let execution = phase.execution_request_for(product_runtime).map_err(|_| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                subject,
            )
        })?;
        Ok(Self::in_request(execution, request))
    }
}

mod court_doors_for_test;

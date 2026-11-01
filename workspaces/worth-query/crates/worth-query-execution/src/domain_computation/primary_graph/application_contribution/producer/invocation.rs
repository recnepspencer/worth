//! Per-call liveness is distinct from persistent installed producer provenance.
use super::WorthQueryProducerCommitAuthority;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryNavigationDenial, WorthQueryOutputDemandDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn confirm_history_source<S: ApplicationSchema, T>(
    history: Option<HistoryProducerInvocation<'_>>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    value: T,
) -> Result<T, WorthQueryOutputDemandDenial> {
    let Some(history) = history else {
        return Ok(value);
    };
    history
        .confirm_in_execution(execution, runtime, value)
        .map_err(history_source_denial)
}

pub(super) fn history_source_denial(
    cause: WorthQueryHistoryNavigationDenial,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::history_source(cause)
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ProducerInvocationAuthority<'invocation> {
    mode: WorthQueryProducerCommitAuthority,
    history: Option<HistoryProducerInvocation<'invocation>>,
}

impl<'invocation> ProducerInvocationAuthority<'invocation> {
    pub(in crate::domain_computation::primary_graph) fn confirm_source<S: ApplicationSchema, T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, WorthQueryOutputDemandDenial> {
        confirm_history_source(self.history, execution, runtime, value)
    }

    pub(in crate::domain_computation::primary_graph) fn for_mode(
        &self,
        mode: WorthQueryProducerCommitAuthority,
    ) -> Self {
        Self {
            mode,
            history: self.history,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn history(
        mode: WorthQueryProducerCommitAuthority,
        history: HistoryProducerInvocation<'invocation>,
    ) -> Self {
        Self {
            mode,
            history: Some(history),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn mode(
        &self,
    ) -> &WorthQueryProducerCommitAuthority {
        &self.mode
    }

    pub(in crate::domain_computation::primary_graph) fn history_control(
        &self,
    ) -> Option<HistoryProducerInvocation<'invocation>> {
        self.history
    }
}

impl From<WorthQueryProducerCommitAuthority> for ProducerInvocationAuthority<'_> {
    fn from(mode: WorthQueryProducerCommitAuthority) -> Self {
        Self {
            mode,
            history: None,
        }
    }
}

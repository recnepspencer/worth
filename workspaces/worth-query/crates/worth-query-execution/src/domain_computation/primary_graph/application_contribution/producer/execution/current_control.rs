//! Re-enter the actual C occurrence without relabeling the producer's D truth.
use super::*;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::SharedSelectedProductOperation;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn select_invocation_control<
    'runtime,
    Schema: ApplicationSchema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    invocation: &ProducerInvocationAuthority<'_>,
) -> Result<Option<SharedSelectedProductOperation<'runtime, Schema>>, ProducerExecutionStop> {
    select_history_control(execution, runtime, invocation.history_control())
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn select_history_control<
    'runtime,
    Schema: ApplicationSchema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    history: Option<HistoryProducerInvocation<'_>>,
) -> Result<Option<SharedSelectedProductOperation<'runtime, Schema>>, ProducerExecutionStop> {
    let Some(history) = history else {
        return Ok(None);
    };
    if let Some(lapse) = history.request_lapse() {
        return Err(request_authority_denied(
            "history producer request",
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                lapse.denial_kind(),
                "history producer request",
            ),
        ));
    }
    let current = history.current().observation();
    let branch = WorthQueryProductBranch::from_occurrence(current.lifecycle_incarnation());
    let selected = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|cause| {
            WorthQueryOutputDemandDenial::product_selection(cause, "history control")
        })?;
    if selected.product().observation() != current {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            "history control changed",
        )
        .into());
    }
    selected
        .prepare_shared_query_basis()
        .map(Some)
        .map_err(|(_, stop)| source_readmission::ready_resource_denial("history control", stop))
}

//! Publish the presentation host's initial source event before async admission.
use super::super::WorthUiPresentationAsyncInstallationError as Denial;
use worth_query::facade::runtime;
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    transactions::WorkerIntentBatch,
};

/// A new Native runtime has no committed source. Its explicit initialization
/// event uses the ordinary prepare/compare-and-publish path, without dummy
/// model records or a fabricated commit/snapshot identity.
pub(super) fn initialize() -> Result<RelationalRuntime, Denial> {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        crate::query_runtime_resources::UI_EXECUTION_POLICY,
        |phase| initialize_in_request(&phase.execution_request()),
    )
    .map_err(|cause| {
        Denial::Runtime(Box::new(runtime::WorthQueryRuntimeError::Advancement(
            cause,
        )))
    })?
}

fn initialize_in_request(
    execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
) -> Result<RelationalRuntime, Denial> {
    let source = RelationalRuntimeApi::builder()
        .runtime_name("worth-ui-presentation-async")
        .build()
        .map_err(|denial| Denial::NativeSourceConstruction(Box::new(denial)))?;
    let basis = source
        .observe_branch(&source.main_branch_identity())
        .map_err(|denial| Denial::NativeSourceBasis(Box::new(denial)))?
        .1;
    let mut transaction = source
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .map_err(|denial| Denial::NativeSourceAdmission(Box::new(denial)))?;
    transaction
        .push_batch(
            WorkerIntentBatch::new("presentation-host-source-initialization"),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(|denial| Denial::NativeSourceStaging(Box::new(denial)))?;
    transaction
        .commit(execution, &source)
        .map_err(|denial| Denial::NativeSourcePublication(Box::new(denial)))?;
    Ok(source)
}

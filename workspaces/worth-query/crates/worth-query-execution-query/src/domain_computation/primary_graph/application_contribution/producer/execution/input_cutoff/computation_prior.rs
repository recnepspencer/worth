//! Borrow the producer's actual C computation record under this request.
use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

/// A retained partitioned run takes the record at the address it will replace.
/// This is independent of whole-output input cutoff. Producers without a
/// retained computation perform no lookup and pay no lookup charge.
pub fn prepare_computation_prior<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    partition: [u8; 32],
    edition: InstalledProducerEdition,
) -> Result<Option<ComputationPrior>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    if !edition.retains_computation() {
        return Ok(None);
    }
    let handle = &runtime.runtime.primary_provider.graph_ref();
    let mut prior = handle
        .output_lineage_ref()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .computation_prior_at_address::<Operation<Schema, Binding>>(
            scope,
            selected.product().observation(),
            partition,
            edition,
            execution,
        )
        .map_err(cutoff_admission_denial)?;
    handle
        .with_runtime(execution, |relational| {
            prior.observe_changes(
                &handle.source_owner_ref().invalidation_owner,
                relational,
                selected.application_basis().snapshot_handle(),
            )
        })
        .map_err(cutoff_admission_denial)?;
    Ok(Some(prior))
}

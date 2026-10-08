//! Reacquire a committed output at the live product head or its retained lineage.
use super::*;

pub(super) fn reacquire_current_committed_output<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Result<Option<WorthQueryProductObservationLease>, WorthQueryOutputDemandDenial>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let committed = receipt.committed_product_publication();
    let selected = runtime
        .on_branch(receipt.product_branch())
        .select()
        .map_err(|error| {
            WorthQueryOutputDemandDenial::product_selection(
                error,
                "settled output product observation could not be reacquired",
            )
        })?;
    let observation = selected.product().observation();
    let original_publication_is_current = committed.is_selected_at(observation);
    let retained_output_is_current = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .current_output_matches_receipt(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            observation,
            receipt,
        );
    Ok(
        (original_publication_is_current || retained_output_is_current)
            .then(|| selected.product().read_lease()),
    )
}

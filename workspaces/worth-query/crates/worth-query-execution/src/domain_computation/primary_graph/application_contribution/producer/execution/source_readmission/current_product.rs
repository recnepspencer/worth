use worth_runtime_world::facade::ProductBranchCurrentnessFailure;
use worth_runtime_world::facade::RuntimeWorldBranchAdmissionDenial;

use super::*;

/// Authenticate an already prepared permission against the exact issued head.
/// The guarded callback only seals currentness; actor verification and its
/// variable work run afterward against the pinned native Product root.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn is_current_selected_product<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<bool, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let expected = selected.product().observation();

    let checked = runtime
        .product_runtime
        .owner
        .observation_port()
        .while_product_branch_current(expected, (), |(), _head| true);
    match checked {
        Ok(current) => Ok(current),
        Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(())) => Ok(false),

        Err(ProductBranchCurrentnessFailure::AdmissionDenied {
            denial: world_denial,
            ..
        }) => {
            let kind = match world_denial {
                worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                    RuntimeWorldBranchAdmissionDenial::CurrentnessAccountingOverflow,
                ) => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                _ => WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            };
            Err(denial(kind, Binding::IDENTITY).into())
        }
    }
}

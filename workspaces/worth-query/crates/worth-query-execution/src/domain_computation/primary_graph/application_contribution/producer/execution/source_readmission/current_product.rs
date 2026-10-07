use worth_runtime_world::facade::{
    CurrentProductHead, ProductBranchCurrentnessFailure, RuntimeWorldBranchAdmissionDenial,
};

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
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let expected = selected.product().observation();
    admission
        .charge_external_work(1)
        .map_err(|stop| resource_denial::<Schema, Binding>(stop))?;
    let comparison = u64::try_from(CurrentProductHead::comparison_work_bound(expected))
        .map_err(|_| work_denial::<Schema, Binding>())?;
    admission
        .charge_external_work(comparison)
        .map_err(|stop| resource_denial::<Schema, Binding>(stop))?;

    let mut first_stop = None;
    let checked = runtime
        .product_runtime
        .owner
        .observation_port()
        .while_product_branch_current_admitted(
            expected,
            (),
            &mut |work| match admission.charge_external_work(work) {
                Ok(()) => true,
                Err(stop) => {
                    first_stop = Some(stop);
                    false
                }
            },
            |(), _head| true,
        );
    match checked {
        Ok(current) => Ok(current),
        Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(())) => Ok(false),
        Err(ProductBranchCurrentnessFailure::PreparationDenied(())) => {
            Err(resource_denial::<Schema, Binding>(
                first_stop.expect("owner preparation refusal retains its typed stop"),
            ))
        }
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

fn work_denial<Schema, Binding>() -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        Binding::IDENTITY,
    )
    .into()
}

fn resource_denial<Schema, Binding>(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, Binding::IDENTITY).into()
}

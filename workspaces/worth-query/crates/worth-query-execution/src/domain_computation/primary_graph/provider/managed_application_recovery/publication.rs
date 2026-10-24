use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    PlannedProductReferenceSuccessor, ProductUnpublishedRecoveryHandle,
};

use super::{
    PublicationDenial, PublicationOutcome, WorthQueryManagedApplicationRecoveryDenial as Denial,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use crate::domain_computation::primary_graph::provider::unpublished_idempotency::ManagedUnpublishedRecoveryGuard;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecovery,
};

/// Reserves Query delivery before either ordinary or conditional owner continuation.
pub(super) fn publish<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    binding: &WorthQueryProductPublicationBinding,
    recovery: &WorthQueryProductUnpublishedRecovery,
    unpublished: WorthQueryProductUnpublishedApplication,
    guard: &mut ManagedUnpublishedRecoveryGuard,
) -> Result<
    Result<
        PublicationOutcome<
            (
                worth_runtime_world::facade::PerformedCompositePublication,
                Option<u64>,
            ),
            WorthQueryProductUnpublishedApplication,
        >,
        PublicationDenial<worth_runtime_world::facade::NoEffectCompositePublication>,
    >,
    Denial,
> {
    let provider = &runtime.primary_provider;
    let world_recovery = binding.recovery();
    let disposition = provider.unpublished_idempotency_disposition();
    if guard.attempt_mut().ordinary_adoption_supported() {
        let prepared = binding
            .prepare_settled_relational_adoption(&unpublished, request)
            .map_err(adoption_denial)?;
        reserve_delivery(
            guard,
            provider,
            prepared.planned_successor(),
            prepared.unpublished_recovery_handle(),
        )?;
        drop(unpublished);
        return Ok(prepared.execute(execution).map(|outcome| match outcome {
            PublicationOutcome::Performed(publication) => {
                PublicationOutcome::Performed((publication, None))
            }
            PublicationOutcome::ProductUnpublished(effects) => {
                PublicationOutcome::ProductUnpublished(
                    WorthQueryProductUnpublishedApplication::new(
                        effects,
                        world_recovery,
                        disposition,
                    ),
                )
            }
        }));
    }

    // Release the inspection before the native owner takes its exclusive recovery lease.
    drop(unpublished);
    let gate = runtime
        .product_runtime
        .activations
        .gate(binding.observation().branch_identity())
        .map_err(|_| Denial::ProductUnavailable)?;
    let _publication = gate
        .begin_publication()
        .map_err(|_| Denial::ProductUnavailable)?;
    let bridge_root = runtime.bridge.conditional_operations();
    let bridge = bridge_root
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let prepared = binding
        .prepare_conditional_definition_adoption(recovery.record_handle(), &bridge, request)
        .map_err(adoption_denial)?;
    reserve_delivery(
        guard,
        provider,
        prepared.planned_successor(),
        prepared.unpublished_recovery_handle(),
    )?;
    Ok(prepared
        .execute(execution, &bridge)
        .map(|outcome| match outcome {
            PublicationOutcome::Performed((publication, lowering)) => {
                PublicationOutcome::Performed((
                    publication,
                    Some(lowering.signal_definition_generation()),
                ))
            }
            PublicationOutcome::ProductUnpublished(effects) => {
                PublicationOutcome::ProductUnpublished(
                    WorthQueryProductUnpublishedApplication::new_conditional_definition(
                        effects,
                        world_recovery,
                        disposition,
                    ),
                )
            }
        }))
}

fn reserve_delivery(
    guard: &mut ManagedUnpublishedRecoveryGuard,
    provider: &WorthQueryPrimaryGraphProvider,
    successor: &PlannedProductReferenceSuccessor,
    handle: ProductUnpublishedRecoveryHandle,
) -> Result<(), Denial> {
    guard
        .attempt_mut()
        .prepare_fresh_slot(provider, successor)
        .map_err(Denial::Demand)?;
    guard.reserve_successor(&handle).map_err(Denial::from)?;
    guard
        .attempt_mut()
        .prepare_fresh_terminal(handle)
        .map_err(Denial::ProviderAdmission)
}

fn adoption_denial(
    denial: PublicationDenial<
        worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
    >,
) -> Denial {
    match denial {
        PublicationDenial::Stopped(stop) => Denial::ExecutionDenied(advancement_denial(stop)),
        PublicationDenial::Refused(denial) => Denial::Adoption(denial),
    }
}

//! Consume the admitted Bridge successor inside the same original durable World attempt.
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner;
use worth_runtime_world::facade::{
    WorldDurablePublicationOutcome, WorldPreparedPublicationWithSignal,
};

pub(super) fn publish(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    guard: &mut QueryDurablePreparationGuard,
    product: &WorthQueryProductPublicationBinding,
    prepared: WorldPreparedPublicationWithSignal,
) -> WorldDurablePublicationOutcome {
    let Some(change) = guard
        .preparation_mut()
        .query
        .attempt
        .take_conditional_definition()
    else {
        return WorldDurablePublicationOutcome::RecoveryRequired {
            cause: worth_runtime_world::facade::WorldDurablePublicationDenial::ConditionalDefinitionRequired,
            custody: prepared.cancel(),
        };
    };
    let parts = change.into_parts();
    if parts.expected_product != *product.observation() {
        return WorldDurablePublicationOutcome::RecoveryRequired {
            cause: worth_runtime_world::facade::WorldDurablePublicationDenial::ConditionalDefinitionExpectedProduct,
            custody: prepared.cancel(),
        };
    }
    let ConditionalOwner::Authored(bridge) = parts.bridge else {
        return WorldDurablePublicationOutcome::RecoveryRequired {
            cause: worth_runtime_world::facade::WorldDurablePublicationDenial::BridgeDefinitionOwnerRequired,
            custody: prepared.cancel(),
        };
    };
    let branch = worth_foundational::facade::DurableSignalBranchIdentity::from_recorded(
        bridge.signal_owner().binding().owner(),
        product.observation().basis().signal_basis().branch_id().0,
    );
    let definition=match bridge.prepare_conditional_definition_publication(
        &execution,branch,&parts.predecessor,parts.request) {
        Ok(prepared)=>prepared,
        Err(cause) => return WorldDurablePublicationOutcome::RecoveryRequired {
            cause: worth_runtime_world::facade::WorldDurablePublicationDenial::BridgeDefinitionPreparation(Box::new(cause)),
            custody: prepared.cancel(),
        },
    };
    let gate = match parts
        .activations
        .gate(product.observation().branch_identity())
    {
        Ok(gate) => gate,
        Err(cause) => {
            guard.preparation_mut().query_denial = Some(Cause::ConditionalActivation(cause));
            return WorldDurablePublicationOutcome::RecoveryRequired {
                cause:
                    worth_runtime_world::facade::WorldDurablePublicationDenial::MovementAdmission,
                custody: prepared.cancel(),
            };
        }
    };
    guard.preparation_mut().authored_completion = Some(definition.completion());
    // Gate admission precedes consuming the original World phase. No second
    // Native candidate or Signal body is constructed by a retry.
    let mut original = Some((prepared, definition));
    match gate.publish(|| {
        let (prepared, definition) = original
            .take()
            .expect("the activation gate invokes its publication once");
        product
            .publication()
            .execute_durable_authored_conditional_definition(
                execution,
                prepared,
                definition,
                guard.preparation_mut().control.cancellation(),
            )
    }) {
        Ok(outcome) => outcome,
        Err(cause) => {
            guard.preparation_mut().query_denial = Some(Cause::ConditionalActivation(cause));
            WorldDurablePublicationOutcome::RecoveryRequired {
                cause:
                    worth_runtime_world::facade::WorldDurablePublicationDenial::MovementAdmission,
                custody: original
                    .take()
                    .expect("gate refusal never invoked the original")
                    .0
                    .cancel(),
            }
        }
    }
}

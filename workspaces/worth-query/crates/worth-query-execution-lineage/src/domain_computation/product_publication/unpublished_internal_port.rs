//! Crate-boundary internal port for `WorthQueryProductUnpublishedApplication`.
//! Never exported on `worth-query-execution::facade` or Host.

use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryPort,
    RuntimeWorldUnpublishedConditionalDefinition,
};

use super::unpublished::WorthQueryProductUnpublishedApplication;
use crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition;

pub trait ProductUnpublishedApplicationInternalPort: Sized {
    fn from_effects(
        effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self;

    fn from_recovered(
        effects: worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self;

    fn from_conditional_definition(
        effects: RuntimeWorldUnpublishedConditionalDefinition,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self;

    fn is_recovered(&self) -> bool;

    fn retains_conditional_definition(&self) -> bool;

    fn recovery_handle(&self) -> ProductUnpublishedRecoveryHandle;

    fn prepare_settled_relational_adoption(
        &self,
        cancellation: &worth_runtime_world::facade::RuntimeWorldCancellationToken,
        deadline: Option<worth_runtime_world::facade::RuntimeWorldInstant>,
    ) -> Result<
        worth_runtime_world::facade::PreparedCompositePublicationWithoutSignal,
        worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
    >;
}

impl ProductUnpublishedApplicationInternalPort for WorthQueryProductUnpublishedApplication {
    fn from_effects(
        effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self::new(effects, recovery, disposition)
    }

    fn from_recovered(
        effects: worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self::new_recovered(effects, recovery, disposition)
    }

    fn from_conditional_definition(
        effects: RuntimeWorldUnpublishedConditionalDefinition,
        recovery: RuntimeWorldRecoveryPort,
        disposition: WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self::new_conditional_definition(effects, recovery, disposition)
    }

    fn is_recovered(&self) -> bool {
        Self::is_recovered(self)
    }

    fn retains_conditional_definition(&self) -> bool {
        Self::retains_conditional_definition(self)
    }

    fn recovery_handle(&self) -> ProductUnpublishedRecoveryHandle {
        Self::recovery_handle(self)
    }

    fn prepare_settled_relational_adoption(
        &self,
        cancellation: &worth_runtime_world::facade::RuntimeWorldCancellationToken,
        deadline: Option<worth_runtime_world::facade::RuntimeWorldInstant>,
    ) -> Result<
        worth_runtime_world::facade::PreparedCompositePublicationWithoutSignal,
        worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
    > {
        Self::prepare_settled_relational_adoption(self, cancellation, deadline)
    }
}

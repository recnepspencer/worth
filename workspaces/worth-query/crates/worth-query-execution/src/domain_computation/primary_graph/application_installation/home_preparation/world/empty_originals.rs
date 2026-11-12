//! World alone releases recovered opens that retain no Native owner effect.
use super::*;
use worth_runtime_world::facade::{
    WorldRecoveredReleaseOutcome as Outcome, WorldRecoveredUnpublishedEffects,
};
use QueryHomeCustodyReadmissionDenial as Denial;

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn release_empty_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: WorldRecoveredUnpublishedEffects,
    ) -> Result<(), Denial> {
        let application = self.runtime.runtime();
        let reservation = application
            .runtime
            .product_runtime
            .reserve_owner_cleanup_for_application()
            .map_err(|_| Denial::OriginalMemoryAdmission)?;
        let recovery = application.runtime.product_runtime.owner.recovery_port();
        match crate::domain_computation::primary_graph::home_original_progression::release_original(
            &recovery, execution, effects,
        ) {
            Outcome::Released(released) => {
                let (original, cleanup) = released.into_parts();
                assert!(
                    original.native_receipt().is_none(),
                    "World released the same empty original"
                );
                drop(original);
                let identity = reservation.install_unpublished(cleanup);
                let cleanup = crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup::new(application.runtime.product_runtime.clone(), identity);
                let mut retirements = crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements::default();
                cleanup
                    .retry_in_request_with_retirements(execution, &mut retirements)
                    .map_err(|_| Denial::OriginalMemoryAdmission)?;
                Ok(())
            }
            Outcome::AdmissionRefused { cause, .. } => Err(Denial::Recovery(cause)),
            Outcome::Refused { cause, .. } => Err(Denial::ReleasedSource(cause)),
            Outcome::Pending(_) => {
                execution.consult().map_err(Denial::Execution)?;
                Err(Denial::UnclassifiedWorldCustody)
            }
            Outcome::EvidenceRequired(_) | Outcome::Indeterminate(_) => {
                Err(Denial::UnclassifiedWorldCustody)
            }
        }
    }
}

//! Release a cold original only after the actual World disposition commits.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::WorldRecoveredReleaseOutcome as Outcome;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn prepare_recovered_product_release(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, recovery: WorthQueryProductUnpublishedRecovery,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
        let handle = recovery.record_handle().clone();
        let port = self.product_runtime.owner.recovery_port();
        let effects = match port.inspect_recovered_effects(&handle) {
            Ok(effects) => effects,
            Err(cause) => return Err(recovery::recovery_failure(recovery, cause.into())),
        };
        let reservation = match self.product_runtime.reserve_owner_cleanup_for_application() {
            Ok(reservation) => reservation,
            Err(_) => {
                return Err(recovery::recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::CleanupCapacityExhausted,
                ))
            }
        };
        let cause = match port.release_recovered_effects(execution, effects) {
            Outcome::Released(released) => {
                self.primary_provider
                    .unpublished_idempotency_disposition()
                    .finish_recovered(&handle);
                let identity = reservation.install_unpublished(released.into_cleanup());
                return Ok(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup::new(self.product_runtime.clone(), identity));
            }
            Outcome::AdmissionRefused { cause, custody } => {
                drop(custody);
                return Err(recovery::recovery_failure(recovery, cause.into()));
            }
            Outcome::Pending(custody) => {
                drop(custody);
                WorthQueryRecoveredProductReleaseDenial::Pending
            }
            Outcome::Refused { cause, custody } => {
                drop(custody);
                WorthQueryRecoveredProductReleaseDenial::Refused(cause)
            }
            Outcome::EvidenceRequired(custody) => {
                drop(custody);
                WorthQueryRecoveredProductReleaseDenial::EvidenceRequired
            }
            Outcome::Indeterminate(custody) => {
                drop(custody);
                WorthQueryRecoveredProductReleaseDenial::Indeterminate
            }
        };
        Err(recovery::recovered_failure(recovery, cause))
    }
}

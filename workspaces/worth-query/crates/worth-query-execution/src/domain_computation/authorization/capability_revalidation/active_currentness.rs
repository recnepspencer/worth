//! Current principal and capability decision facts at the retained native basis.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationRuntimeCore;

impl WorthQueryApplicationRuntimeCore {
    pub(super) fn validate_active_capability_currentness(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        authorization: &WorthQueryRetainedCapabilityAuthorization,
        refresh: &WorthQueryCapabilityRefresh<'_>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if snapshot.branch_id() != refresh.product.relational_basis().identity().branch_id() {
            return Err(inconsistent_refresh(refresh));
        }
        if !authorization
            .principal()
            .remains_current_in(execution, runtime, snapshot)
            .map_err(|kind| {
                WorthQueryOperationAuthorizationDenial::new(kind, "principal currentness")
            })?
        {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StalePrincipal,
                ),
                refresh.installed.contract().name(),
            ));
        }
        if !authorization
            .decision()
            .remains_current_in(execution, runtime, snapshot, self.authorization.bridge())
            .map_err(|cause| {
                crate::domain_computation::authorization::denial::relational_read_denial(
                    cause,
                    "authorization currentness",
                )
            })?
        {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StaleAuthorization,
                ),
                refresh.installed.contract().name(),
            ));
        }
        Ok(())
    }
}

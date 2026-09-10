use super::*;

impl super::super::super::WorthQueryWorkspace {
    pub(crate) fn readmits_current_shared_invalidation_epoch(
        &self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        maintenance_ordinal: u64,
        impact: &Arc<crate::domain_installation::WorthQueryImpactDecision>,
        invalidation_seed: &Arc<crate::domain_installation::WorthQuerySharedInvalidationSeed>,
        sharing: &Arc<crate::domain_installation::WorthQueryAdmittedProjectionSharing>,
    ) -> bool {
        self.runtime.readmits_current_shared_invalidation_epoch(
            readmission,
            maintenance_ordinal,
            impact,
            invalidation_seed,
            sharing,
        )
    }

    pub(crate) fn drain_shared_projection_lease(
        &mut self,
        capability: &Arc<super::super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Result<WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDrainFailure> {
        let mut counters = WorthQuerySharedProjectionDrainCounters {
            workspace_capability_checks: 1,
            ..WorthQuerySharedProjectionDrainCounters::default()
        };
        if let Err(error) = self
            .runtime
            .admit_managed_live_capability(capability, "shared-projection-owner")
        {
            return Err(WorthQuerySharedProjectionDrainFailure { error, counters });
        }
        counters.abandoned_owner_index_lookups = 1;
        match self
            .runtime
            .reap_abandoned_shared_projection_leases_for_owner(readmission.owner)
        {
            Ok(reaped) => counters.abandoned_leases_reaped = reaped,
            Err(error) => {
                return Err(WorthQuerySharedProjectionDrainFailure { error, counters });
            }
        }
        self.runtime
            .drain_shared_projection_lease(readmission, counters)
    }
}

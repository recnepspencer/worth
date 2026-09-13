use crate::basis_lifecycle::BasisOperationLane;

use super::{
    WorthQueryAdmittedConsumerInvalidation, WorthQueryConsumerInvalidationAdmissionStop,
    WorthQueryConsumerInvalidationCounters, WorthQueryConsumerInvalidationDelta,
    WorthQueryConsumerInvalidationDeltaStopKind,
};

impl<D: 'static, O, F, L: BasisOperationLane>
    crate::domain_installation::WorthQuerySharedLiveProjectionLease<D, O, F, L>
{
    pub fn admit_consumer_invalidation_delta(
        &self,
        delta: WorthQueryConsumerInvalidationDelta,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<
        WorthQueryAdmittedConsumerInvalidation<'_>,
        WorthQueryConsumerInvalidationAdmissionStop,
    > {
        let mut counters = WorthQueryConsumerInvalidationCounters::default();
        let readmission = self.readmission();
        let installation_generation = self
            .snapshot()
            .bound_operation()
            .operation()
            .installation_generation();
        counters.live_source_authority_checks = 1;
        let source_is_current = super::super::operation_execution::validate_live_source_authority(
            self.snapshot(),
            workspace,
        )
        .is_ok();
        if !source_is_current {
            return Err(admission_stop(delta, counters));
        }
        counters.delta_authority_readmission_checks = 1;
        if !delta
            .authority
            .readmits(&readmission, installation_generation)
        {
            return Err(admission_stop(delta, counters));
        }
        counters.epoch_readmission_checks = 1;
        if !workspace.readmits_current_shared_invalidation_epoch(
            readmission,
            delta.maintenance_ordinal,
            &delta.impact,
            &delta.epoch_work,
            &delta.sharing,
        ) {
            return Err(admission_stop(delta, counters));
        }
        counters.sharing_readmission_checks = 1;
        if !delta.sharing.readmits_lease(
            readmission.source_identity,
            crate::domain_installation::operation_authority_chain::operation_phase_basis(
                self.snapshot().bound_operation().authority_proof(),
            ),
            readmission.closure,
        ) {
            return Err(admission_stop(delta, counters));
        }
        Ok(WorthQueryAdmittedConsumerInvalidation { delta, readmission })
    }
}

fn admission_stop(
    delta: WorthQueryConsumerInvalidationDelta,
    counters: WorthQueryConsumerInvalidationCounters,
) -> WorthQueryConsumerInvalidationAdmissionStop {
    WorthQueryConsumerInvalidationAdmissionStop {
        kind: WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease,
        delta,
        counters,
    }
}

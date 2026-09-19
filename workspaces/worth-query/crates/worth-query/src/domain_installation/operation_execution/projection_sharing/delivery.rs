use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::{
    WorthQuerySharedExecutionOwnerIdentity, WorthQuerySharedProjectionLeaseIdentity,
    WorthQueryWorkspace,
};

use super::{WorthQueryAdmittedProjectionSharing, WorthQuerySharedLiveProjectionLease};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQuerySharedProjectionDeliveryCounters {
    pub owner_drain_calls: usize,
    pub underlying_maintenance_passes: usize,
    pub lease_index_visits: usize,
    pub fanout_targets: usize,
    pub this_lease_view: usize,
    pub this_lease_semantic_delivery: usize,
    pub conditional_compute_contacts: usize,
    pub impact_classifications: usize,
    pub unrelated_owner_scans: usize,
    pub unrelated_lease_scans: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQuerySharedProjectionDrainCounters {
    pub workspace_capability_checks: usize,
    pub abandoned_owner_index_lookups: usize,
    pub abandoned_leases_reaped: usize,
    pub runtime_affinity_checks: usize,
    pub owner_index_lookups: usize,
    pub lease_index_lookups: usize,
    pub sharing_readmission_checks: usize,
    pub retained_epoch_checks: usize,
    pub retained_epoch_pending_lookups: usize,
    pub route_index_lookups: usize,
    pub owner_maintenance_drains: usize,
    pub epoch_compilations: usize,
    pub unrelated_owner_scans: usize,
    pub unrelated_lease_scans: usize,
}

pub struct WorthQuerySharedProjectionDelivery {
    owner: WorthQuerySharedExecutionOwnerIdentity,
    lease: WorthQuerySharedProjectionLeaseIdentity,
    maintenance_ordinal: u64,
    delivery: Arc<crate::ordinary::live::WorthQueryManagedLiveDelivery>,
    impact: Arc<crate::domain_installation::WorthQueryImpactDecision>,
    impact_closure:
        Arc<crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>,
    conditional_provenance: Arc<[crate::domain_installation::WorthQueryConditionalProvenance]>,
    invalidation_seed: Arc<super::WorthQuerySharedInvalidationSeed>,
    _sharing: Arc<WorthQueryAdmittedProjectionSharing>,
    source_identity: String,
    lease_affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
    counters: WorthQuerySharedProjectionDeliveryCounters,
    drain_counters: WorthQuerySharedProjectionDrainCounters,
}

pub(crate) struct WorthQuerySharedProjectionEpochEvidence {
    pub(crate) maintenance_ordinal: u64,
    pub(crate) delivery: Arc<crate::ordinary::live::WorthQueryManagedLiveDelivery>,
    pub(crate) impact: Arc<crate::domain_installation::WorthQueryImpactDecision>,
    pub(crate) impact_closure:
        Arc<crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>,
    pub(crate) conditional_provenance:
        Arc<[crate::domain_installation::WorthQueryConditionalProvenance]>,
    pub(crate) invalidation_seed: Arc<super::WorthQuerySharedInvalidationSeed>,
    pub(crate) sharing: Arc<WorthQueryAdmittedProjectionSharing>,
    pub(crate) counters: WorthQuerySharedProjectionDeliveryCounters,
}

pub(crate) struct WorthQuerySharedProjectionLeaseViewAuthority {
    pub(crate) owner: WorthQuerySharedExecutionOwnerIdentity,
    pub(crate) lease: WorthQuerySharedProjectionLeaseIdentity,
    pub(crate) source_identity: String,
    pub(crate) lease_affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
    pub(crate) drain_counters: WorthQuerySharedProjectionDrainCounters,
}

pub(crate) enum WorthQuerySharedImpactReadmissionDenial {
    Lease,
    Impact,
}

impl WorthQuerySharedProjectionDelivery {
    pub(crate) fn from_epoch_evidence(
        epoch: WorthQuerySharedProjectionEpochEvidence,
        lease_view: WorthQuerySharedProjectionLeaseViewAuthority,
    ) -> Self {
        Self {
            owner: lease_view.owner,
            lease: lease_view.lease,
            maintenance_ordinal: epoch.maintenance_ordinal,
            delivery: epoch.delivery,
            impact: epoch.impact,
            impact_closure: epoch.impact_closure,
            conditional_provenance: epoch.conditional_provenance,
            invalidation_seed: epoch.invalidation_seed,
            _sharing: epoch.sharing,
            source_identity: lease_view.source_identity,
            lease_affinity: lease_view.lease_affinity,
            counters: epoch.counters,
            drain_counters: lease_view.drain_counters,
        }
    }

    pub const fn owner_identity(&self) -> WorthQuerySharedExecutionOwnerIdentity {
        self.owner
    }

    pub const fn lease_identity(&self) -> WorthQuerySharedProjectionLeaseIdentity {
        self.lease
    }

    pub const fn maintenance_ordinal(&self) -> u64 {
        self.maintenance_ordinal
    }

    pub fn delivery(&self) -> &crate::ordinary::live::WorthQueryManagedLiveDelivery {
        &self.delivery
    }

    pub fn impact(&self) -> &crate::domain_installation::WorthQueryImpactDecision {
        &self.impact
    }

    pub fn conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        &self.conditional_provenance
    }

    pub const fn counters(&self) -> WorthQuerySharedProjectionDeliveryCounters {
        self.counters
    }

    pub const fn drain_counters(&self) -> WorthQuerySharedProjectionDrainCounters {
        self.drain_counters
    }

    pub fn shares_invalidation_epoch_with(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.maintenance_ordinal == other.maintenance_ordinal
            && Arc::ptr_eq(&self.invalidation_seed, &other.invalidation_seed)
    }

    pub fn retains_same_impact_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.impact, &other.impact)
    }

    pub fn invalidation_epoch_counters(
        &self,
    ) -> crate::domain_installation::WorthQueryConsumerInvalidationEpochCounters {
        self.invalidation_seed.counters()
    }

    pub(crate) fn conditional_provenance_arc(
        &self,
    ) -> &Arc<[crate::domain_installation::WorthQueryConditionalProvenance]> {
        &self.conditional_provenance
    }

    pub(crate) fn invalidation_seed(&self) -> &Arc<super::WorthQuerySharedInvalidationSeed> {
        &self.invalidation_seed
    }

    pub(crate) fn sharing_admission(&self) -> &Arc<WorthQueryAdmittedProjectionSharing> {
        &self._sharing
    }

    /// Inspection is public, but consequence authority remains sealed behind
    /// exact owner/lease readmission because the decision is owner-closure-bound.
    #[allow(dead_code)] // Phase 20 is the first consequence-authority consumer.
    pub(crate) fn readmit_impact_for_lease(
        &self,
        readmission: &super::WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Result<
        &Arc<crate::domain_installation::WorthQueryImpactDecision>,
        WorthQuerySharedImpactReadmissionDenial,
    > {
        let lease_readmitted = self.owner == readmission.owner
            && self.lease == readmission.lease
            && self.source_identity == readmission.source_identity
            && self.lease_affinity.binding_identity == readmission.binding_identity
            && self.lease_affinity.capability_identity == readmission.capability_identity
            && self._sharing.readmits_lease(
                readmission.source_identity,
                &self.lease_affinity,
                readmission.closure,
            );
        if !lease_readmitted {
            return Err(WorthQuerySharedImpactReadmissionDenial::Lease);
        }
        let impact_readmitted = self
            .impact
            .readmit_managed_delivery(&self.impact_closure, &self.delivery);
        if !impact_readmitted {
            return Err(WorthQuerySharedImpactReadmissionDenial::Impact);
        }
        Ok(&self.impact)
    }
}

#[derive(Debug)]
pub struct WorthQuerySharedProjectionDrainStop {
    error: crate::runtime::WorthQueryRuntimeError,
    counters: WorthQuerySharedProjectionDrainCounters,
}

impl WorthQuerySharedProjectionDrainStop {
    pub(crate) fn new(
        error: crate::runtime::WorthQueryRuntimeError,
        counters: WorthQuerySharedProjectionDrainCounters,
    ) -> Self {
        Self { error, counters }
    }

    pub fn error(&self) -> &crate::runtime::WorthQueryRuntimeError {
        &self.error
    }

    pub const fn counters(&self) -> WorthQuerySharedProjectionDrainCounters {
        self.counters
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySharedLiveProjectionLease<D, O, F, L> {
    pub fn drain(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDrainStop> {
        workspace
            .drain_shared_projection_lease(self.workspace_capability(), self.readmission())
            .map_err(|stopped| {
                WorthQuerySharedProjectionDrainStop::new(stopped.error, stopped.counters)
            })
    }
}

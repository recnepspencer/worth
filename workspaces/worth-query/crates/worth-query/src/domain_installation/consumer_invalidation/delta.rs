use std::sync::Arc;

use super::authority::WorthQueryConsumerInvalidationAuthority;
use super::counters::{
    WorthQueryConsumerInvalidationCounters, WorthQueryConsumerInvalidationEpochCounters,
};
use super::meaning::{
    WorthQueryConsumerInvalidationCause, WorthQueryConsumerInvalidationContinuation,
    WorthQueryConsumerInvalidationDisposition, WorthQueryConsumerInvalidationLocality,
};

pub struct WorthQueryConsumerInvalidationDelta {
    pub(super) authority: WorthQueryConsumerInvalidationAuthority,
    pub(super) maintenance_ordinal: u64,
    pub(super) impact: Arc<crate::domain_installation::WorthQueryImpactDecision>,
    pub(super) conditional_provenance:
        Arc<[crate::domain_installation::WorthQueryConditionalProvenance]>,
    pub(super) sharing:
        Arc<crate::domain_installation::operation_execution::WorthQueryAdmittedProjectionSharing>,
    pub(super) epoch_work:
        Arc<crate::domain_installation::operation_execution::WorthQuerySharedInvalidationSeed>,
    pub(super) affected_native_keys: Vec<crate::domain_installation::WorthQueryNativeAccessKey>,
    pub(super) disposition: WorthQueryConsumerInvalidationDisposition,
    pub(super) cause: WorthQueryConsumerInvalidationCause,
    pub(super) locality: WorthQueryConsumerInvalidationLocality,
    pub(super) continuation: WorthQueryConsumerInvalidationContinuation,
    pub(super) counters: WorthQueryConsumerInvalidationCounters,
}

impl WorthQueryConsumerInvalidationDelta {
    pub const fn authority(&self) -> &WorthQueryConsumerInvalidationAuthority {
        &self.authority
    }

    pub const fn maintenance_ordinal(&self) -> u64 {
        self.maintenance_ordinal
    }

    pub fn impact(&self) -> &crate::domain_installation::WorthQueryImpactDecision {
        &self.impact
    }

    pub fn conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        &self.conditional_provenance
    }

    pub fn affected_native_keys(&self) -> &[crate::domain_installation::WorthQueryNativeAccessKey] {
        &self.affected_native_keys
    }

    pub fn affected_entity_identities(
        &self,
    ) -> &[crate::memory_workspace::WorthQueryEntityIdentity] {
        self.epoch_work.affected_entities()
    }

    pub const fn disposition(&self) -> WorthQueryConsumerInvalidationDisposition {
        self.disposition
    }

    pub const fn cause(&self) -> &WorthQueryConsumerInvalidationCause {
        &self.cause
    }

    pub const fn locality(&self) -> WorthQueryConsumerInvalidationLocality {
        self.locality
    }

    pub const fn continuation(&self) -> WorthQueryConsumerInvalidationContinuation {
        self.continuation
    }

    pub const fn counters(&self) -> WorthQueryConsumerInvalidationCounters {
        self.counters
    }

    pub fn epoch_counters(&self) -> WorthQueryConsumerInvalidationEpochCounters {
        self.epoch_work.counters()
    }

    pub fn shares_epoch_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.epoch_work, &other.epoch_work)
    }

    pub fn retains_same_impact_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.impact, &other.impact)
    }

    pub fn retains_same_compatibility_evidence_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sharing, &other.sharing)
    }

    pub(super) fn compatibility_continuity(
        &self,
    ) -> crate::domain_installation::operation_execution::WorthQueryProjectionSharingContinuity
    {
        self.sharing.continuity()
    }
}

pub struct WorthQueryAdmittedConsumerInvalidation<'lease> {
    pub(super) delta: WorthQueryConsumerInvalidationDelta,
    pub(super) readmission:
        crate::domain_installation::operation_execution::WorthQuerySharedProjectionLeaseReadmission<
            'lease,
        >,
}

impl WorthQueryAdmittedConsumerInvalidation<'_> {
    pub const fn delta(&self) -> &WorthQueryConsumerInvalidationDelta {
        &self.delta
    }

    pub(crate) fn remains_current(&self, workspace: &crate::runtime::WorthQueryWorkspace) -> bool {
        workspace.readmits_current_shared_invalidation_epoch(
            self.readmission,
            self.delta.maintenance_ordinal,
            &self.delta.impact,
            &self.delta.epoch_work,
            &self.delta.sharing,
        )
    }
}

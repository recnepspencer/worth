use std::collections::BTreeSet;
use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryLiveProjectionRefresh, WorthQueryLiveProjectionRefreshError,
    WorthQuerySettledDomainProjection, WorthQuerySharedProjectionLeaseReadmission,
};
use crate::live::WorthQueryMaintenanceScope;

#[derive(Clone)]
pub(crate) struct WorthQueryCurrentSharedConsumerDeliveryPolicy {
    pub(crate) policy: crate::live::WorthQuerySharedConsumerDeliveryPolicy,
    pub(crate) generation: u64,
}

pub(crate) enum WorthQuerySharedPrimaryOwnerRefreshStop {
    Runtime(super::super::WorthQueryRuntimeError),
    Refresh(WorthQueryLiveProjectionRefreshError),
}

impl super::super::WorthQueryWorkspace {
    pub(crate) fn admit_shared_consumer_delivery_policy(
        &mut self,
        capability: &Arc<super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        policy: crate::live::WorthQuerySharedConsumerDeliveryPolicy,
    ) -> Result<u64, super::super::WorthQueryRuntimeError> {
        self.runtime
            .admit_managed_live_capability(capability, "shared-consumer-delivery-policy")?;
        self.runtime
            .admit_shared_consumer_delivery_policy(readmission, policy)
            .ok_or_else(
                || super::super::WorthQueryRuntimeError::LiveSubscriptionInstallation {
                    view_name: "shared-consumer-delivery-policy".to_owned(),
                    stage: "policy-admission",
                    message: "shared consumer policy no longer binds a current lease".to_owned(),
                },
            )
    }

    pub(crate) fn current_shared_consumer_delivery_policy(
        &self,
        capability: &Arc<super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Option<WorthQueryCurrentSharedConsumerDeliveryPolicy> {
        self.runtime
            .admit_managed_live_capability(capability, "shared-consumer-delivery-policy")
            .ok()?;
        self.runtime
            .current_shared_consumer_delivery_policy(readmission)
    }

    pub(crate) fn refresh_shared_primary_owner<
        D: 'static,
        O: 'static,
        F: 'static,
        L: BasisOperationLane,
    >(
        &mut self,
        capability: &Arc<super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        source: &WorthQuerySettledDomainProjection<D, O, F, L>,
        scope: &WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<WorthQueryLiveProjectionRefresh, WorthQuerySharedPrimaryOwnerRefreshStop> {
        self.runtime
            .admit_managed_live_capability(capability, "shared-primary-invalidation")
            .map_err(WorthQuerySharedPrimaryOwnerRefreshStop::Runtime)?;
        self.runtime
            .reap_abandoned_shared_projection_leases_for_owner(readmission.owner)
            .map_err(WorthQuerySharedPrimaryOwnerRefreshStop::Runtime)?;
        let mut counters = crate::domain_installation::WorthQuerySharedProjectionDrainCounters {
            workspace_capability_checks: 1,
            ..Default::default()
        };
        let (owner_identity, owner) = self
            .runtime
            .take_shared_owner_for_primary_delivery(readmission, &mut counters)
            .map_err(WorthQuerySharedPrimaryOwnerRefreshStop::Runtime)?;
        let refreshed = crate::domain_installation::refresh_granular_source(
            source,
            owner.handle(),
            scope,
            basis,
            self,
        )
        .map_err(WorthQuerySharedPrimaryOwnerRefreshStop::Refresh);
        self.runtime
            .restore_shared_owner_after_primary_delivery(owner_identity, owner);
        refreshed
    }

    pub(crate) fn readmits_shared_primary_lease(
        &self,
        capability: &Arc<super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> bool {
        self.runtime
            .admit_managed_live_capability(capability, "shared-primary-invalidation")
            .is_ok()
            && self.runtime.readmits_shared_primary_lease(readmission)
    }

    pub(crate) fn current_shared_primary_leases(
        &mut self,
        capability: &Arc<super::super::WorthQueryManagedLiveWorkspaceCapability>,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Result<
        BTreeSet<super::WorthQuerySharedProjectionLeaseIdentity>,
        super::super::WorthQueryRuntimeError,
    > {
        self.runtime
            .admit_managed_live_capability(capability, "shared-primary-invalidation")?;
        self.runtime
            .reap_abandoned_shared_projection_leases_for_owner(readmission.owner)?;
        self.runtime
            .current_shared_primary_leases(readmission)
            .ok_or_else(
                || super::super::WorthQueryRuntimeError::LiveSubscriptionInstallation {
                    view_name: "shared-primary-invalidation".to_owned(),
                    stage: "shared-primary-lease-set-readmission",
                    message: "shared primary owner no longer readmits the exact consumer lease"
                        .to_owned(),
                },
            )
    }
}

impl super::super::WorthQueryRuntime {
    fn admit_shared_consumer_delivery_policy(
        &mut self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        policy: crate::live::WorthQuerySharedConsumerDeliveryPolicy,
    ) -> Option<u64> {
        if !self.readmits_shared_primary_lease(readmission) {
            return None;
        }
        let record = self
            .shared_projection_owners
            .owners
            .get_mut(&readmission.owner)?
            .leases
            .get_mut(&readmission.lease)?;
        record.consumer_delivery_policy_generation =
            record.consumer_delivery_policy_generation.checked_add(1)?;
        record.consumer_delivery_policy = Some(policy);
        Some(record.consumer_delivery_policy_generation)
    }

    fn current_shared_consumer_delivery_policy(
        &self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Option<WorthQueryCurrentSharedConsumerDeliveryPolicy> {
        if !self.readmits_shared_primary_lease(readmission) {
            return None;
        }
        let record = self
            .shared_projection_owners
            .owners
            .get(&readmission.owner)?
            .leases
            .get(&readmission.lease)?;
        Some(WorthQueryCurrentSharedConsumerDeliveryPolicy {
            policy: record.consumer_delivery_policy.clone()?,
            generation: record.consumer_delivery_policy_generation,
        })
    }

    fn current_shared_primary_leases(
        &self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> Option<BTreeSet<super::WorthQuerySharedProjectionLeaseIdentity>> {
        if !self.readmits_shared_primary_lease(readmission) {
            return None;
        }
        Some(
            self.shared_projection_owners
                .owners
                .get(&readmission.owner)?
                .leases
                .keys()
                .copied()
                .collect(),
        )
    }

    fn readmits_shared_primary_lease(
        &self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    ) -> bool {
        if readmission.owner.runtime_authority() != self.authority_identity.as_u64()
            || readmission.lease.runtime_authority() != self.authority_identity.as_u64()
        {
            return false;
        }
        let Some(owner) = self.shared_projection_owners.owners.get(&readmission.owner) else {
            return false;
        };
        let Some(record) = owner.leases.get(&readmission.lease) else {
            return false;
        };
        record.source_identity == readmission.source_identity
            && record.affinity.binding_identity == readmission.binding_identity
            && record.affinity.capability_identity == readmission.capability_identity
            && owner.admission.readmits_lease(
                readmission.source_identity,
                &record.affinity,
                readmission.closure,
            )
            && owner.admission.readmits_lease(
                owner.admission.subject_source_identity(),
                owner.admission.subject_affinity(),
                &owner.closure,
            )
    }
}

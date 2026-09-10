use crate::domain_installation::{
    WorthQuerySharedProjectionDrainCounters, WorthQuerySharedProjectionLeaseReadmission,
};

use super::registry::WorthQuerySharedProjectionOwner;
use super::WorthQuerySharedExecutionOwnerIdentity;

impl super::super::WorthQueryRuntime {
    pub(crate) fn take_shared_owner_for_primary_delivery(
        &mut self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        counters: &mut WorthQuerySharedProjectionDrainCounters,
    ) -> Result<
        (
            WorthQuerySharedExecutionOwnerIdentity,
            WorthQuerySharedProjectionOwner,
        ),
        super::super::WorthQueryRuntimeError,
    > {
        let owner_identity = readmission.owner;
        counters.runtime_affinity_checks = 1;
        if owner_identity.runtime_authority() != self.authority_identity.as_u64() {
            return Err(delivery_error(
                "shared execution owner belongs to a foreign runtime",
            ));
        }
        counters.owner_index_lookups = 1;
        let Some(owner) = self.shared_projection_owners.owners.remove(&owner_identity) else {
            return Err(delivery_error("shared execution owner is not active"));
        };
        counters.lease_index_lookups = 1;
        counters.sharing_readmission_checks = 1;
        let exact = owner.leases.get(&readmission.lease).is_some_and(|record| {
            record.source_identity == readmission.source_identity
                && record.affinity.binding_identity == readmission.binding_identity
                && record.affinity.capability_identity == readmission.capability_identity
                && owner.admission.readmits_lease(
                    readmission.source_identity,
                    &record.affinity,
                    readmission.closure,
                )
        });
        counters.retained_epoch_checks = 1;
        counters.retained_epoch_pending_lookups = usize::from(owner.epoch.is_some());
        let epoch_ready = owner
            .epoch
            .as_ref()
            .is_none_or(|epoch| epoch.pending.is_empty());
        if !exact || !epoch_ready {
            self.shared_projection_owners
                .owners
                .insert(owner_identity, owner);
            return Err(delivery_error(if exact {
                "every indexed lease must observe the current epoch before owner delivery"
            } else {
                "shared projection lease no longer readmits its exact source and closure"
            }));
        }
        Ok((owner_identity, owner))
    }

    pub(crate) fn restore_shared_owner_after_primary_delivery(
        &mut self,
        owner_identity: WorthQuerySharedExecutionOwnerIdentity,
        owner: WorthQuerySharedProjectionOwner,
    ) {
        let replaced = self
            .shared_projection_owners
            .owners
            .insert(owner_identity, owner);
        debug_assert!(replaced.is_none());
    }
}

fn delivery_error(detail: &str) -> super::super::WorthQueryRuntimeError {
    super::super::WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: "shared-primary-invalidation".into(),
        stage: "shared-owner-admission",
        message: detail.into(),
    }
}

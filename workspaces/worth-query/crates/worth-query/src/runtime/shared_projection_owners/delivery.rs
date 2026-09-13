use std::collections::BTreeSet;
use std::sync::Arc;

use crate::domain_installation::{
    WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDeliveryCounters,
    WorthQuerySharedProjectionDrainCounters, WorthQuerySharedProjectionLeaseReadmission,
};

use super::registry::WorthQuerySharedProjectionOwner;
use super::{WorthQuerySharedExecutionOwnerIdentity, WorthQuerySharedProjectionLeaseIdentity};

mod workspace_admission;

pub(super) struct WorthQuerySharedProjectionEpoch {
    pub(super) ordinal: u64,
    pub(super) delivery: Arc<crate::ordinary::live::WorthQueryManagedLiveDelivery>,
    pub(super) impact: Arc<crate::domain_installation::WorthQueryImpactDecision>,
    pub(super) impact_closure:
        Arc<crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>,
    pub(super) conditional_provenance:
        Arc<[crate::domain_installation::WorthQueryConditionalProvenance]>,
    pub(super) invalidation_seed: Arc<crate::domain_installation::WorthQuerySharedInvalidationSeed>,
    pub(super) admission: Arc<crate::domain_installation::WorthQueryAdmittedProjectionSharing>,
    pub(super) pending: BTreeSet<WorthQuerySharedProjectionLeaseIdentity>,
    pub(super) counters: WorthQuerySharedProjectionDeliveryCounters,
}

impl WorthQuerySharedProjectionEpoch {
    pub(super) fn view(
        &self,
        owner: WorthQuerySharedExecutionOwnerIdentity,
        lease: WorthQuerySharedProjectionLeaseIdentity,
        source_identity: String,
        lease_affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
        drain_counters: WorthQuerySharedProjectionDrainCounters,
    ) -> WorthQuerySharedProjectionDelivery {
        WorthQuerySharedProjectionDelivery::from_epoch_evidence(
            crate::domain_installation::WorthQuerySharedProjectionEpochEvidence {
                maintenance_ordinal: self.ordinal,
                delivery: Arc::clone(&self.delivery),
                impact: Arc::clone(&self.impact),
                impact_closure: Arc::clone(&self.impact_closure),
                conditional_provenance: Arc::clone(&self.conditional_provenance),
                invalidation_seed: Arc::clone(&self.invalidation_seed),
                sharing: Arc::clone(&self.admission),
                counters: self.counters,
            },
            crate::domain_installation::WorthQuerySharedProjectionLeaseViewAuthority {
                owner,
                lease,
                source_identity,
                lease_affinity,
                drain_counters,
            },
        )
    }

    pub(super) fn abandon(&mut self, lease: WorthQuerySharedProjectionLeaseIdentity) {
        self.pending.remove(&lease);
    }
}

pub(crate) struct WorthQuerySharedProjectionDrainFailure {
    pub(crate) error: super::super::WorthQueryRuntimeError,
    pub(crate) counters: WorthQuerySharedProjectionDrainCounters,
}

impl super::super::WorthQueryRuntime {
    pub(crate) fn readmits_current_shared_invalidation_epoch(
        &self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        maintenance_ordinal: u64,
        impact: &Arc<crate::domain_installation::WorthQueryImpactDecision>,
        invalidation_seed: &Arc<crate::domain_installation::WorthQuerySharedInvalidationSeed>,
        sharing: &Arc<crate::domain_installation::WorthQueryAdmittedProjectionSharing>,
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
        let Some(epoch) = owner.epoch.as_ref() else {
            return false;
        };
        record.source_identity == readmission.source_identity
            && record.affinity.binding_identity == readmission.binding_identity
            && record.affinity.capability_identity == readmission.capability_identity
            && epoch.ordinal == maintenance_ordinal
            && !epoch.pending.contains(&readmission.lease)
            && Arc::ptr_eq(&epoch.impact, impact)
            && Arc::ptr_eq(&epoch.invalidation_seed, invalidation_seed)
            && Arc::ptr_eq(&epoch.admission, sharing)
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

    pub(crate) fn drain_shared_projection_lease(
        &mut self,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        mut counters: WorthQuerySharedProjectionDrainCounters,
    ) -> Result<WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDrainFailure> {
        let owner_identity = readmission.owner;
        let lease_identity = readmission.lease;
        counters.runtime_affinity_checks = 1;
        if owner_identity.runtime_authority() != self.authority_identity.as_u64()
            || lease_identity.runtime_authority() != self.authority_identity.as_u64()
        {
            return Err(drain_failure(
                "shared projection lease belongs to a foreign runtime",
                counters,
            ));
        }
        counters.owner_index_lookups = 1;
        let Some(mut owner) = self.shared_projection_owners.owners.remove(&owner_identity) else {
            return Err(drain_failure(
                "shared execution owner is not active",
                counters,
            ));
        };
        let result = self.drain_admitted_shared_owner(
            owner_identity,
            lease_identity,
            &mut owner,
            readmission,
            counters,
        );
        self.shared_projection_owners
            .owners
            .insert(owner_identity, owner);
        result
    }

    fn drain_admitted_shared_owner(
        &mut self,
        owner_identity: WorthQuerySharedExecutionOwnerIdentity,
        lease_identity: WorthQuerySharedProjectionLeaseIdentity,
        owner: &mut WorthQuerySharedProjectionOwner,
        readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
        mut counters: WorthQuerySharedProjectionDrainCounters,
    ) -> Result<WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDrainFailure> {
        let (source_identity, lease_affinity) =
            admit_shared_lease_readmission(owner, lease_identity, readmission, &mut counters)?;
        counters.retained_epoch_checks = 1;
        if let Some(delivery) = drain_retained_shared_epoch(
            owner,
            owner_identity,
            lease_identity,
            &source_identity,
            &lease_affinity,
            &mut counters,
        )? {
            return Ok(delivery);
        }
        self.open_shared_owner_epoch(owner, lease_identity, &mut counters)?;
        counters.epoch_compilations = 1;
        Ok(owner
            .epoch
            .as_ref()
            .expect("new shared epoch must be retained")
            .view(
                owner_identity,
                lease_identity,
                source_identity,
                lease_affinity,
                counters,
            ))
    }

    fn open_shared_owner_epoch(
        &mut self,
        owner: &mut WorthQuerySharedProjectionOwner,
        lease_identity: WorthQuerySharedProjectionLeaseIdentity,
        counters: &mut WorthQuerySharedProjectionDrainCounters,
    ) -> Result<(), WorthQuerySharedProjectionDrainFailure> {
        self.require_active_shared_owner_route(owner, counters)?;
        counters.owner_maintenance_drains = 1;
        let delivery = self.drain_shared_owner_delivery(owner);
        owner.epoch = Some(compile_next_shared_owner_epoch(
            owner,
            lease_identity,
            delivery,
        ));
        Ok(())
    }

    fn require_active_shared_owner_route(
        &self,
        owner: &WorthQuerySharedProjectionOwner,
        counters: &mut WorthQuerySharedProjectionDrainCounters,
    ) -> Result<(), WorthQuerySharedProjectionDrainFailure> {
        counters.route_index_lookups = 1;
        let target = super::super::WorthQueryLiveArtifactTarget::from_subscription_installation(
            owner.handle.view().subscription_installation(),
        );
        if !self.installed_live_routes.contains_target(&target) {
            return Err(drain_failure(
                "shared owner live route is no longer active",
                *counters,
            ));
        }
        Ok(())
    }

    fn drain_shared_owner_delivery(
        &mut self,
        owner: &WorthQuerySharedProjectionOwner,
    ) -> Arc<crate::ordinary::live::WorthQueryManagedLiveDelivery> {
        let resource_name = owner.handle.name().to_string();
        let batches = self.drain_live_delivery_batches(owner.handle.view());
        Arc::new(
            crate::ordinary::live::WorthQueryManagedLiveDelivery::from_runtime(
                super::super::WorthQueryManagedLiveRuntimeDelivery::new(resource_name, batches),
            ),
        )
    }
}

fn admit_shared_lease_readmission(
    owner: &WorthQuerySharedProjectionOwner,
    lease_identity: WorthQuerySharedProjectionLeaseIdentity,
    readmission: WorthQuerySharedProjectionLeaseReadmission<'_>,
    counters: &mut WorthQuerySharedProjectionDrainCounters,
) -> Result<
    (
        String,
        crate::domain_installation::WorthQueryOperationAuthorityBasis,
    ),
    WorthQuerySharedProjectionDrainFailure,
> {
    counters.lease_index_lookups = 1;
    let Some(record) = owner.leases.get(&lease_identity) else {
        return Err(drain_failure(
            "shared projection lease is not active",
            *counters,
        ));
    };
    counters.sharing_readmission_checks = 2;
    let readmits_exact_pair = record.source_identity == readmission.source_identity
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
        );
    if !readmits_exact_pair {
        return Err(drain_failure(
            "shared projection lease no longer readmits its exact source and closure",
            *counters,
        ));
    }
    Ok((record.source_identity.clone(), record.affinity.clone()))
}

fn drain_retained_shared_epoch(
    owner: &mut WorthQuerySharedProjectionOwner,
    owner_identity: WorthQuerySharedExecutionOwnerIdentity,
    lease_identity: WorthQuerySharedProjectionLeaseIdentity,
    source_identity: &str,
    lease_affinity: &crate::domain_installation::WorthQueryOperationAuthorityBasis,
    counters: &mut WorthQuerySharedProjectionDrainCounters,
) -> Result<Option<WorthQuerySharedProjectionDelivery>, WorthQuerySharedProjectionDrainFailure> {
    let Some(epoch) = owner.epoch.as_mut() else {
        return Ok(None);
    };
    counters.retained_epoch_pending_lookups = 1;
    if epoch.pending.remove(&lease_identity) {
        return Ok(Some(epoch.view(
            owner_identity,
            lease_identity,
            source_identity.to_owned(),
            lease_affinity.clone(),
            *counters,
        )));
    }
    if !epoch.pending.is_empty() {
        return Err(drain_failure(
            "shared maintenance cannot advance until every indexed lease observes the epoch",
            *counters,
        ));
    }
    Ok(None)
}

fn compile_next_shared_owner_epoch(
    owner: &mut WorthQuerySharedProjectionOwner,
    lease_identity: WorthQuerySharedProjectionLeaseIdentity,
    delivery: Arc<crate::ordinary::live::WorthQueryManagedLiveDelivery>,
) -> WorthQuerySharedProjectionEpoch {
    let impact = Arc::new(
        crate::domain_installation::WorthQueryImpactDecision::from_managed_live_delivery(
            &owner.closure,
            &delivery,
        ),
    );
    let maintenance_passes = delivery
        .batches()
        .iter()
        .filter(|batch| batch.maintenance_work().is_some())
        .count();
    let this_lease_semantic_delivery = usize::from(
        !delivery.is_empty()
            && impact.class()
                != crate::domain_installation::WorthQueryImpactClass::UnaffectedOrSuppressed,
    );
    owner.next_maintenance_ordinal += 1;
    let lease_count = owner.leases.len();
    let mut pending = owner.leases.keys().copied().collect::<BTreeSet<_>>();
    pending.remove(&lease_identity);
    let invalidation_seed = Arc::new(
        crate::domain_installation::WorthQuerySharedInvalidationSeed::compile(
            &delivery,
            lease_count,
        ),
    );
    WorthQuerySharedProjectionEpoch {
        ordinal: owner.next_maintenance_ordinal,
        delivery,
        impact,
        impact_closure: Arc::clone(&owner.closure),
        conditional_provenance: Arc::clone(&owner.conditional_provenance),
        invalidation_seed,
        admission: Arc::clone(&owner.admission),
        pending,
        counters: WorthQuerySharedProjectionDeliveryCounters {
            owner_drain_calls: 1,
            underlying_maintenance_passes: maintenance_passes,
            lease_index_visits: lease_count,
            fanout_targets: lease_count,
            this_lease_view: 1,
            this_lease_semantic_delivery,
            conditional_compute_contacts: 0,
            impact_classifications: 0,
            unrelated_owner_scans: 0,
            unrelated_lease_scans: 0,
        },
    }
}

fn drain_error(detail: &str) -> super::super::WorthQueryRuntimeError {
    super::super::WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: "shared-projection-owner".into(),
        stage: "shared-projection-delivery",
        message: detail.into(),
    }
}

fn drain_failure(
    detail: &str,
    counters: WorthQuerySharedProjectionDrainCounters,
) -> WorthQuerySharedProjectionDrainFailure {
    WorthQuerySharedProjectionDrainFailure {
        error: drain_error(detail),
        counters,
    }
}

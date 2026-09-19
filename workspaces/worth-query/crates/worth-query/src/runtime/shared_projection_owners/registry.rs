use std::collections::BTreeMap;

use crate::ordinary::live::WorthQueryManagedLiveHandle;

use super::{
    WorthQuerySharedExecutionOwnerIdentity, WorthQuerySharedProjectionLeaseIdentity,
    WorthQuerySharedProjectionLeaseToken,
};

mod workspace_release;

pub(crate) struct WorthQuerySharedProjectionOwner {
    pub(super) handle: WorthQueryManagedLiveHandle,
    pub(super) _live_receipt: crate::domain_installation::WorthQueryLiveProjectionReceipt,
    pub(super) conditional_provenance:
        std::sync::Arc<[crate::domain_installation::WorthQueryConditionalProvenance]>,
    pub(super) closure: std::sync::Arc<
        crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    >,
    pub(super) admission:
        std::sync::Arc<crate::domain_installation::WorthQueryAdmittedProjectionSharing>,
    pub(super) leases:
        BTreeMap<WorthQuerySharedProjectionLeaseIdentity, WorthQuerySharedProjectionLeaseRecord>,
    pub(super) next_maintenance_ordinal: u64,
    pub(super) epoch: Option<super::delivery::WorthQuerySharedProjectionEpoch>,
}

impl WorthQuerySharedProjectionOwner {
    pub(crate) fn handle(&self) -> &WorthQueryManagedLiveHandle {
        &self.handle
    }
}

pub(super) struct WorthQuerySharedProjectionLeaseRecord {
    pub(super) source_identity: String,
    pub(super) affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
    pub(super) consumer_delivery_policy:
        Option<crate::live::WorthQuerySharedConsumerDeliveryPolicy>,
    pub(super) consumer_delivery_policy_generation: u64,
}

#[derive(Default)]
pub(crate) struct WorthQuerySharedProjectionOwnerRegistry {
    pub(super) owners:
        BTreeMap<WorthQuerySharedExecutionOwnerIdentity, WorthQuerySharedProjectionOwner>,
    next_owner: u64,
    next_lease: u64,
}

pub(crate) struct WorthQuerySharedOwnerRegistration {
    pub(crate) owner: WorthQuerySharedExecutionOwnerIdentity,
    pub(crate) subject: WorthQuerySharedProjectionLeaseToken,
    pub(crate) candidate: Option<WorthQuerySharedProjectionLeaseToken>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedLeaseRelease {
    owner: WorthQuerySharedExecutionOwnerIdentity,
    lease: WorthQuerySharedProjectionLeaseIdentity,
    owner_terminal: bool,
    closeout_identity: Option<crate::evidence_identity::WorthQueryEvidenceIdentity>,
    counters: WorthQuerySharedLeaseReleaseCounters,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQuerySharedLeaseReleaseCounters {
    pub capability_checks: usize,
    pub owner_index_lookups: usize,
    pub lease_index_lookups: usize,
    pub lease_removals: usize,
    pub epoch_abandon_calls: usize,
    pub owner_removals: usize,
    pub close_attempts: usize,
    pub close_completions: usize,
    pub owner_reinsertions: usize,
    pub unrelated_owner_scans: usize,
    pub unrelated_lease_scans: usize,
}

impl WorthQuerySharedLeaseRelease {
    pub const fn owner(&self) -> WorthQuerySharedExecutionOwnerIdentity {
        self.owner
    }

    pub const fn lease(&self) -> WorthQuerySharedProjectionLeaseIdentity {
        self.lease
    }

    pub const fn owner_terminal(&self) -> bool {
        self.owner_terminal
    }

    pub fn closeout_identity(
        &self,
    ) -> Option<&crate::evidence_identity::WorthQueryEvidenceIdentity> {
        self.closeout_identity.as_ref()
    }

    pub const fn counters(&self) -> WorthQuerySharedLeaseReleaseCounters {
        self.counters
    }
}

pub(crate) struct WorthQuerySharedLeaseReleaseError {
    pub(crate) token: WorthQuerySharedProjectionLeaseToken,
    pub(crate) error: super::super::WorthQueryRuntimeError,
    pub(crate) counters: WorthQuerySharedLeaseReleaseCounters,
}

impl WorthQuerySharedProjectionOwnerRegistry {
    fn register(
        &mut self,
        runtime_authority: u64,
        registration: crate::domain_installation::WorthQueryCheckedSharedOwnerRegistration,
    ) -> WorthQuerySharedOwnerRegistration {
        let (handle, live_receipt, conditional_provenance, closure, admission) =
            registration.into_parts();
        self.next_owner += 1;
        let owner = WorthQuerySharedExecutionOwnerIdentity::new(
            runtime_authority,
            self.next_owner,
            self.next_owner,
        );
        let subject = self.issue_token(owner);
        let candidate = admission.candidate().map(|_| self.issue_token(owner));
        let mut leases = BTreeMap::new();
        leases.insert(
            subject.lease(),
            WorthQuerySharedProjectionLeaseRecord {
                source_identity: admission.subject_source_identity().to_string(),
                affinity: admission.subject_affinity().clone(),
                consumer_delivery_policy: None,
                consumer_delivery_policy_generation: 0,
            },
        );
        if let (Some(token), Some((source_identity, affinity))) =
            (candidate.as_ref(), admission.candidate())
        {
            leases.insert(
                token.lease(),
                WorthQuerySharedProjectionLeaseRecord {
                    source_identity: source_identity.to_string(),
                    affinity: affinity.clone(),
                    consumer_delivery_policy: None,
                    consumer_delivery_policy_generation: 0,
                },
            );
        }
        let admission = std::sync::Arc::new(admission);
        let conditional_provenance = std::sync::Arc::from(conditional_provenance);
        self.owners.insert(
            owner,
            WorthQuerySharedProjectionOwner {
                handle,
                _live_receipt: live_receipt,
                conditional_provenance,
                closure,
                admission,
                leases,
                next_maintenance_ordinal: 0,
                epoch: None,
            },
        );
        WorthQuerySharedOwnerRegistration {
            owner,
            subject,
            candidate,
        }
    }

    fn issue_token(
        &mut self,
        owner: WorthQuerySharedExecutionOwnerIdentity,
    ) -> WorthQuerySharedProjectionLeaseToken {
        self.next_lease += 1;
        WorthQuerySharedProjectionLeaseToken::new(
            owner,
            WorthQuerySharedProjectionLeaseIdentity::new(
                owner.runtime_authority(),
                self.next_lease,
                self.next_lease,
            ),
        )
    }
}

impl super::super::WorthQueryRuntime {
    pub(crate) fn register_shared_projection_owner(
        &mut self,
        bundle: crate::domain_installation::WorthQueryCheckedSharedOwnerRegistration,
    ) -> Result<
        WorthQuerySharedOwnerRegistration,
        crate::domain_installation::WorthQueryCheckedSharedOwnerRegistration,
    > {
        let target = super::super::WorthQueryLiveArtifactTarget::from_subscription_installation(
            bundle.handle().view().subscription_installation(),
        );
        if !super::super::WorthQueryManagedLiveWorkspaceCapability::same_instance(
            &self.managed_live_resource_capability,
            bundle.handle().workspace_capability(),
        ) || !self.installed_live_routes.contains_target(&target)
        {
            return Err(bundle);
        }
        Ok(self
            .shared_projection_owners
            .register(self.authority_identity.as_u64(), bundle))
    }

    pub(crate) fn register_singleton_projection_owner(
        &mut self,
        bundle: crate::domain_installation::WorthQueryCheckedSharedOwnerRegistration,
    ) -> Result<
        WorthQuerySharedOwnerRegistration,
        crate::domain_installation::WorthQueryCheckedSharedOwnerRegistration,
    > {
        self.register_shared_projection_owner(bundle)
    }

    pub(crate) fn release_shared_projection_lease(
        &mut self,
        token: WorthQuerySharedProjectionLeaseToken,
    ) -> Result<WorthQuerySharedLeaseRelease, WorthQuerySharedLeaseReleaseError> {
        let mut counters = WorthQuerySharedLeaseReleaseCounters::default();
        let owner_identity = token.owner();
        let lease_identity = token.lease();
        counters.owner_index_lookups = 1;
        let Some(owner) = self
            .shared_projection_owners
            .owners
            .get_mut(&owner_identity)
        else {
            return Err(release_error(
                token,
                "shared execution owner is not active",
                counters,
            ));
        };
        counters.lease_index_lookups = 1;
        if !owner.leases.contains_key(&lease_identity) {
            return Err(release_error(
                token,
                "shared projection lease is not active",
                counters,
            ));
        }
        if owner.leases.len() > 1 {
            owner.leases.remove(&lease_identity);
            counters.lease_removals = 1;
            if let Some(epoch) = owner.epoch.as_mut() {
                epoch.abandon(lease_identity);
                counters.epoch_abandon_calls = 1;
            }
            return Ok(WorthQuerySharedLeaseRelease {
                owner: owner_identity,
                lease: lease_identity,
                owner_terminal: false,
                closeout_identity: None,
                counters,
            });
        }

        counters.owner_removals = 1;
        let mut owner = self
            .shared_projection_owners
            .owners
            .remove(&owner_identity)
            .expect("validated last shared lease must retain its owner");
        counters.close_attempts = 1;
        let closeout = self.close_managed_live_view(
            owner.handle.view(),
            super::super::WorthQueryManagedLiveResourceCloseCause::Disposal,
        );
        match closeout {
            Ok(closeout) => {
                counters.close_completions = 1;
                owner.handle.disarm();
                Ok(WorthQuerySharedLeaseRelease {
                    owner: owner_identity,
                    lease: lease_identity,
                    owner_terminal: true,
                    closeout_identity: Some(closeout.evidence_identity().clone()),
                    counters,
                })
            }
            Err(error) => {
                counters.owner_reinsertions = 1;
                self.shared_projection_owners
                    .owners
                    .insert(owner_identity, owner);
                Err(WorthQuerySharedLeaseReleaseError {
                    token,
                    error,
                    counters,
                })
            }
        }
    }

    pub(crate) fn reap_abandoned_shared_projection_leases(
        &mut self,
    ) -> Result<(), super::super::WorthQueryRuntimeError> {
        let abandoned = self
            .managed_live_resource_capability
            .take_abandoned_shared_projection_leases();
        let mut pending = abandoned.into_iter();
        while let Some(token) = pending.next() {
            if let Err(stopped) = self.release_shared_projection_lease(token) {
                let mut retry = vec![stopped.token];
                retry.extend(pending);
                self.managed_live_resource_capability
                    .restore_abandoned_shared_projection_leases(retry);
                return Err(stopped.error);
            }
        }
        Ok(())
    }

    pub(crate) fn reap_abandoned_shared_projection_leases_for_owner(
        &mut self,
        owner: WorthQuerySharedExecutionOwnerIdentity,
    ) -> Result<usize, super::super::WorthQueryRuntimeError> {
        let abandoned = self
            .managed_live_resource_capability
            .take_abandoned_shared_projection_leases_for_owner(owner);
        let count = abandoned.len();
        let mut pending = abandoned.into_iter();
        while let Some(token) = pending.next() {
            if let Err(stopped) = self.release_shared_projection_lease(token) {
                let mut retry = vec![stopped.token];
                retry.extend(pending);
                self.managed_live_resource_capability
                    .restore_abandoned_shared_projection_leases(retry);
                return Err(stopped.error);
            }
        }
        Ok(count)
    }
}

fn release_error(
    token: WorthQuerySharedProjectionLeaseToken,
    detail: &str,
    counters: WorthQuerySharedLeaseReleaseCounters,
) -> WorthQuerySharedLeaseReleaseError {
    WorthQuerySharedLeaseReleaseError {
        token,
        error: super::super::WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: "shared-projection-owner".into(),
            stage: "shared-projection-lease-release",
            message: detail.into(),
        },
        counters,
    }
}

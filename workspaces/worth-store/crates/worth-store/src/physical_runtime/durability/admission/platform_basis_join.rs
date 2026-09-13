use std::sync::Arc;

use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::{
    durability::{
        PhysicalDurabilityGroupingRuntimeAuthority, PhysicalDurabilityGroupingRuntimeOwner,
        PhysicalDurabilityReopenObservation, PhysicalMutationIdempotencyRuntimeAuthority,
        PhysicalMutationIdempotencyRuntimeOwner, RebuiltPhysicalMutationIdempotency,
    },
    RuntimeIdentity,
};

use super::AdmittedPhysicalDurabilityPolicy;

pub(in crate::physical_runtime) struct PhysicalDurabilityRuntimeOwner {
    policy: AdmittedPhysicalDurabilityPolicy,
    runtime: RuntimeIdentity,
    grouping: Arc<PhysicalDurabilityGroupingRuntimeOwner>,
}

pub(in crate::physical_runtime) struct ReopenedPhysicalDurabilityRuntimeOwner {
    bound: PhysicalDurabilityRuntimeOwner,
    idempotency: Arc<PhysicalMutationIdempotencyRuntimeOwner>,
    reopen: PhysicalDurabilityReopenObservation,
}

impl PhysicalDurabilityRuntimeOwner {
    pub(in crate::physical_runtime) const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime
    }

    pub(in crate::physical_runtime) fn observation(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityObservation {
        crate::physical_runtime::PhysicalDurabilityObservation::new(self.runtime, &self.policy)
    }

    pub(in crate::physical_runtime) fn install_rebuilt_idempotency(
        self,
        rebuilt: RebuiltPhysicalMutationIdempotency,
    ) -> ReopenedPhysicalDurabilityRuntimeOwner {
        let checkpoint = rebuilt.checkpoint_counters();
        let reopen = PhysicalDurabilityReopenObservation::new(
            checkpoint.checkpoint_artifact_bytes(),
            checkpoint.checkpoint_bytes_read(),
            checkpoint.dirty_body_bytes_skipped(),
            checkpoint.binding_records_read(),
            checkpoint.integrity_admissions(),
            rebuilt.wal_members_read(),
        );
        ReopenedPhysicalDurabilityRuntimeOwner {
            bound: self,
            idempotency: rebuilt.into_owner(),
            reopen,
        }
    }
}

impl ReopenedPhysicalDurabilityRuntimeOwner {
    pub(in crate::physical_runtime) const fn runtime_identity(&self) -> RuntimeIdentity {
        self.bound.runtime_identity()
    }

    pub(in crate::physical_runtime) fn observation(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityObservation {
        self.bound.observation().with_reopen(self.reopen)
    }

    pub(in crate::physical_runtime) fn idempotency_authority(
        &self,
    ) -> PhysicalMutationIdempotencyRuntimeAuthority {
        PhysicalMutationIdempotencyRuntimeOwner::authority(&self.idempotency)
    }

    pub(in crate::physical_runtime) fn binding_compaction_authority(
        &self,
    ) -> crate::physical_runtime::durability::PhysicalMutationBindingCompactionRuntimeAuthority
    {
        PhysicalMutationIdempotencyRuntimeOwner::binding_compaction_authority(&self.idempotency)
    }

    pub(in crate::physical_runtime) fn grouping_authority(
        &self,
    ) -> PhysicalDurabilityGroupingRuntimeAuthority {
        PhysicalDurabilityGroupingRuntimeOwner::authority(&self.bound.grouping)
    }

    pub(in crate::physical_runtime) fn into_recovery_basis(
        self,
        completed_unobserved: Box<[crate::physical_runtime::CompletedUnobservedPhysicalMutation]>,
    ) -> Result<
        (
            AdmittedPhysicalDurabilityPolicy,
            crate::physical_runtime::PhysicalRecoveryOperationFates,
        ),
        crate::physical_runtime::PhysicalDurabilityCloseoutDenial,
    > {
        let Self {
            bound,
            idempotency,
            reopen: _reopen,
        } = self;
        let PhysicalDurabilityRuntimeOwner {
            policy,
            runtime: _runtime,
            grouping,
        } = bound;
        drop(grouping);
        let operations =
            PhysicalMutationIdempotencyRuntimeOwner::into_recovery_operation_fates(
                idempotency,
                completed_unobserved,
            )
            .map_err(|denial| match denial {
                crate::physical_runtime::durability::PhysicalIdempotencyCloseoutDenial::LiveCompactionAuthority => {
                    crate::physical_runtime::PhysicalDurabilityCloseoutDenial::LiveIdempotencyCompactionAuthority
                }
            })?;
        Ok((policy, operations))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::physical_runtime) enum PhysicalDurabilityRuntimeRebind {
    StoreIdentityMismatch,
    AdmissionBasisMismatch,
}

pub(in crate::physical_runtime) fn bind_policy_to_runtime(
    policy: AdmittedPhysicalDurabilityPolicy,
    media: &QualifiedFilesystemMedia,
    runtime: RuntimeIdentity,
) -> Result<PhysicalDurabilityRuntimeOwner, PhysicalDurabilityRuntimeRebind> {
    if policy.store_identity() != media.store_identity() {
        return Err(PhysicalDurabilityRuntimeRebind::StoreIdentityMismatch);
    }
    let current = media
        .physical_durability_admission_identity()
        .map_err(|_| PhysicalDurabilityRuntimeRebind::AdmissionBasisMismatch)?;
    if policy.admission_basis_identity() != current {
        return Err(PhysicalDurabilityRuntimeRebind::AdmissionBasisMismatch);
    }
    let grouping = PhysicalDurabilityGroupingRuntimeOwner::new(
        policy.store_identity(),
        runtime,
        policy.identity(),
    );
    Ok(PhysicalDurabilityRuntimeOwner {
        policy,
        runtime,
        grouping,
    })
}

use crate::physical_runtime::record_serving::ServingHealth;
use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    PhysicalRootPublicationWorkAction, PhysicalRootPublicationWorkFailureCause,
    PhysicalRootPublicationWorkScope, PhysicalWorkEffectFate, PhysicalWorkRecoveryDisposition,
    PhysicalWorkSettlementEvidence, RootNamespaceDurablePhysicalMutationMembers,
    RootReplacedPhysicalMutationMembers, SettledPhysicalWork,
};

use super::{PhysicalRootPublicationWorkFailure, PhysicalRootPublicationWorkPort};

pub enum PhysicalRootNamespaceDurabilityOutcome {
    Durable(RootNamespaceDurablePhysicalMutationMembers),
    NotStarted(PhysicalRootNamespaceDurabilityNotStarted),
    InspectionRequired(IndeterminatePhysicalRootNamespaceDurability),
}

pub struct PhysicalRootNamespaceDurabilityNotStarted {
    replaced: RootReplacedPhysicalMutationMembers,
    cause: PhysicalRootNamespaceDurabilityFailureCause,
}

pub struct IndeterminatePhysicalRootNamespaceDurability {
    core: super::super::mutation::RootPublicationPreparedCore,
    replacement: Box<SettledPhysicalWork>,
    namespace_synchronization: Box<SettledPhysicalWork>,
    recovery: PhysicalWorkRecoveryDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootNamespaceDurabilityFailureCause {
    Work(PhysicalRootPublicationWorkFailureCause),
    ProvenNoEffect {
        recovery: PhysicalWorkRecoveryDisposition,
    },
}

pub(in crate::physical_runtime) fn synchronize_root_namespace(
    replaced: RootReplacedPhysicalMutationMembers,
    work: &PhysicalRootPublicationWorkPort,
    health: &ServingHealth,
) -> PhysicalRootNamespaceDurabilityOutcome {
    let scope = PhysicalRootPublicationWorkScope::new(
        replaced.identity(),
        PhysicalRootPublicationWorkAction::SynchronizeParentNamespace,
    )
    .expect("parent namespace synchronization is a valid root-publication action");
    let settlement = match work.execute(scope) {
        Ok(settlement) => Box::new(settlement),
        Err(failure) => return namespace_not_started(replaced, failure),
    };
    match settlement.evidence() {
        PhysicalWorkSettlementEvidence::PublicationEffect { .. }
            if settlement.evidence().fate() == PhysicalWorkEffectFate::PublicationCompleted =>
        {
            let (core, replacement) = replaced.into_parts();
            PhysicalRootNamespaceDurabilityOutcome::Durable(
                RootNamespaceDurablePhysicalMutationMembers::new(core, replacement, settlement),
            )
        }
        PhysicalWorkSettlementEvidence::NoEffect(_) => {
            PhysicalRootNamespaceDurabilityOutcome::NotStarted(
                PhysicalRootNamespaceDurabilityNotStarted {
                    replaced,
                    cause: PhysicalRootNamespaceDurabilityFailureCause::ProvenNoEffect {
                        recovery: settlement.recovery_disposition(),
                    },
                },
            )
        }
        _ => {
            let (mut core, replacement) = replaced.into_parts();
            core.require_inspection();
            health.revoke();
            let recovery = settlement.recovery_disposition();
            PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(
                IndeterminatePhysicalRootNamespaceDurability {
                    core,
                    replacement,
                    namespace_synchronization: settlement,
                    recovery,
                },
            )
        }
    }
}

fn namespace_not_started(
    replaced: RootReplacedPhysicalMutationMembers,
    failure: PhysicalRootPublicationWorkFailure,
) -> PhysicalRootNamespaceDurabilityOutcome {
    PhysicalRootNamespaceDurabilityOutcome::NotStarted(PhysicalRootNamespaceDurabilityNotStarted {
        replaced,
        cause: PhysicalRootNamespaceDurabilityFailureCause::Work(failure.cause()),
    })
}

impl PhysicalRootNamespaceDurabilityNotStarted {
    pub const fn cause(&self) -> PhysicalRootNamespaceDurabilityFailureCause {
        self.cause
    }

    pub fn replaced(&self) -> &RootReplacedPhysicalMutationMembers {
        &self.replaced
    }

    pub fn into_replaced(self) -> RootReplacedPhysicalMutationMembers {
        self.replaced
    }
}

impl PhysicalRootNamespaceDurabilityOutcome {
    pub(in crate::physical_runtime) fn runtime_released(
        replaced: RootReplacedPhysicalMutationMembers,
    ) -> Self {
        Self::NotStarted(PhysicalRootNamespaceDurabilityNotStarted {
            replaced,
            cause: PhysicalRootNamespaceDurabilityFailureCause::Work(
                PhysicalRootPublicationWorkFailureCause::RuntimeReleased,
            ),
        })
    }
}

impl IndeterminatePhysicalRootNamespaceDurability {
    pub const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.core.group()
    }

    pub fn members(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        self.core.members()
    }

    pub fn replacement_effect_identity(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalEffectIdentity> {
        self.replacement.effect_identity()
    }

    pub fn effect_fate(&self) -> PhysicalWorkEffectFate {
        self.namespace_synchronization.evidence().fate()
    }

    pub const fn recovery_disposition(&self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }
}
#[cfg(test)]
#[path = "namespace_durability_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

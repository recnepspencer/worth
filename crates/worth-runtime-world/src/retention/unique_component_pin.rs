use std::fmt;
use std::sync::Arc;

use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisAdmissionIdentity,
};
use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalBranchBasisAdmissionIdentity};

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::RuntimeWorldOwnerIdentity;

use super::component_obligation::RetentionControlSurface;
use super::ComponentBasisDependencyClass;

/// Exact component value selected by the Runtime World retention owner. The
/// composite basis is only the source of the component value; retention never
/// keys a lease by the composite identity.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ExactComponentBasis<'a> {
    Relational(&'a AdmittedRelationalBranchBasis),
    Signal(&'a AdmittedSignalBranchBasis),
}

/// Independent key used by the unique-pin registry. Descriptors and
/// composite identities are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ExactComponentBasisKey {
    Relational(RelationalBranchBasisAdmissionIdentity),
    Signal(SignalBranchBasisAdmissionIdentity),
}

/// Request to retain one exact component basis under one Runtime World
/// dependency class. A pair of requests represents one composite obligation,
/// but each key is counted independently.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ExactComponentPinRequest<'a> {
    owner: RuntimeWorldOwnerIdentity,
    component: ExactComponentBasis<'a>,
    dependency: ComponentBasisDependencyClass,
}

impl<'a> ExactComponentPinRequest<'a> {
    pub(crate) fn relational(
        basis: &'a AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Self {
        Self {
            owner: basis.owner_identity(),
            component: ExactComponentBasis::Relational(basis.relational_basis()),
            dependency,
        }
    }

    pub(crate) fn signal(
        basis: &'a AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Self {
        Self {
            owner: basis.owner_identity(),
            component: ExactComponentBasis::Signal(basis.signal_basis()),
            dependency,
        }
    }

    pub(crate) const fn owner(self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub(crate) const fn dependency(self) -> ComponentBasisDependencyClass {
        self.dependency
    }

    pub(crate) const fn component(self) -> ExactComponentBasis<'a> {
        self.component
    }

    pub(crate) fn key(self) -> ExactComponentBasisKey {
        match self.component {
            ExactComponentBasis::Relational(basis) => {
                ExactComponentBasisKey::Relational(basis.admission_identity().clone())
            }
            ExactComponentBasis::Signal(basis) => {
                ExactComponentBasisKey::Signal(basis.admission_identity().clone())
            }
        }
    }
}

/// Runtime World-local identity for one live owner lease generation. It is
/// evidence for stale-token detection, not an authority that can mint a lease.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ComponentBasisLeaseIdentity {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl ComponentBasisLeaseIdentity {
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
}

/// One move-only claim on one dependency count. The component-owner lease is
/// deliberately held by the registry entry, so repeated exact uses share one
/// external owner lease without making the claim cloneable.
pub(crate) struct ComponentBasisPinClaim {
    pub(super) owner: RuntimeWorldOwnerIdentity,
    pub(super) key: ExactComponentBasisKey,
    pub(super) dependency: ComponentBasisDependencyClass,
    pub(super) lease_identity: ComponentBasisLeaseIdentity,
    pub(super) control: Arc<dyn RetentionControlSurface>,
}

impl fmt::Debug for ComponentBasisPinClaim {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ComponentBasisPinClaim")
            .field("owner", &self.owner)
            .field("key", &self.key)
            .field("dependency", &self.dependency)
            .field("lease_identity", &self.lease_identity)
            .finish_non_exhaustive()
    }
}

impl ComponentBasisPinClaim {
    pub(super) fn new(
        owner: RuntimeWorldOwnerIdentity,
        key: ExactComponentBasisKey,
        dependency: ComponentBasisDependencyClass,
        lease_identity: ComponentBasisLeaseIdentity,
        control: Arc<dyn RetentionControlSurface>,
    ) -> Self {
        Self {
            owner,
            key,
            dependency,
            lease_identity,
            control,
        }
    }

    pub(crate) const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub(crate) const fn dependency(&self) -> ComponentBasisDependencyClass {
        self.dependency
    }
}

#[cfg(test)]
#[path = "unique_component_pin/tests.rs"]
mod tests;

//! One installed selected checkpoint, or an authenticated absent observation.

use worth_store_physical_backend::{ObservedRecoveryArtifact, RecoveryDiscoveryArtifact};
use worth_store_physical_integrity::VerifiedCheckpointFacts;
use worth_store_recovery_physics::PhysicalCheckpointBase;

use crate::physical_runtime::StoreRecoveryCheckpointBindingBasis;

use super::{PhysicalRecoveryCoordination, SharedRecoveryCheckpoint};

pub(super) enum RecoveryCheckpointSelection {
    Unobserved,
    Absent,
    Selected {
        base: PhysicalCheckpointBase,
        shared: SharedRecoveryCheckpoint,
        binding: StoreRecoveryCheckpointBindingBasis,
    },
}

pub(in crate::physical_runtime) enum RecoveryCheckpointOwnership {
    Absent,
    Present(SharedRecoveryCheckpoint),
}

/// Proof that one recovery coordination observed `checkpoint.current` absent,
/// never damaged or rejected: the only key to the generation-zero basis.
/// Minted once per coordination, by its absent installation; neither cloned
/// nor constructible elsewhere.
#[derive(Debug)]
pub struct AbsentCheckpointWitness {
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    runtime: crate::physical_runtime::RuntimeIdentity,
}

impl AbsentCheckpointWitness {
    /// Whether this witness was minted by `coordination`, the only owner whose
    /// selection it proves absent.
    pub(in crate::physical_runtime) fn binds(
        &self,
        coordination: &PhysicalRecoveryCoordination,
    ) -> bool {
        self.store == coordination.store && self.runtime == coordination.runtime_identity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCheckpointInstallationDenial {
    AlreadyInstalled,
    Unobserved,
    StoreMismatch,
    PoolMismatch,
    CheckpointMismatch,
    BindingMismatch,
    InvalidAbsence,
}

impl PhysicalRecoveryCoordination {
    pub fn install_selected_checkpoint(
        &mut self,
        base: PhysicalCheckpointBase,
        shared: SharedRecoveryCheckpoint,
        binding: StoreRecoveryCheckpointBindingBasis,
    ) -> Result<(), SelectedCheckpointInstallationDenial> {
        use SelectedCheckpointInstallationDenial as Denial;
        if !matches!(
            self.checkpoint_selection,
            RecoveryCheckpointSelection::Unobserved
        ) {
            return Err(Denial::AlreadyInstalled);
        }
        if base.checkpoint().source().identity().store_identity() != self.store {
            return Err(Denial::StoreMismatch);
        }
        if !shared.matches_owner(&self.residency) {
            return Err(Denial::PoolMismatch);
        }
        if *base.checkpoint() != shared.facts() {
            return Err(Denial::CheckpointMismatch);
        }
        if !binding.matches_checkpoint(base.checkpoint()) {
            return Err(Denial::BindingMismatch);
        }
        if !binding.matches_owner(&self.residency) {
            return Err(Denial::PoolMismatch);
        }
        self.checkpoint_selection = RecoveryCheckpointSelection::Selected {
            base,
            shared,
            binding,
        };
        Ok(())
    }

    pub fn install_absent_checkpoint(
        &mut self,
        observed: ObservedRecoveryArtifact,
    ) -> Result<AbsentCheckpointWitness, SelectedCheckpointInstallationDenial> {
        use SelectedCheckpointInstallationDenial as Denial;
        if !matches!(
            self.checkpoint_selection,
            RecoveryCheckpointSelection::Unobserved
        ) {
            return Err(Denial::AlreadyInstalled);
        }
        if observed.store_identity() != self.store
            || observed.artifact() != &RecoveryDiscoveryArtifact::CurrentCheckpoint
            || observed.offset() != 0
            || observed.bytes().is_some()
        {
            return Err(Denial::InvalidAbsence);
        }
        self.checkpoint_selection = RecoveryCheckpointSelection::Absent;
        Ok(AbsentCheckpointWitness {
            store: self.store,
            runtime: self.runtime_identity(),
        })
    }

    pub fn checkpoint(&self) -> Option<&SharedRecoveryCheckpoint> {
        match &self.checkpoint_selection {
            RecoveryCheckpointSelection::Selected { shared, .. } => Some(shared),
            RecoveryCheckpointSelection::Unobserved | RecoveryCheckpointSelection::Absent => None,
        }
    }

    pub(in crate::physical_runtime) fn require_selected_checkpoint(
        &self,
        facts: &VerifiedCheckpointFacts,
    ) -> Result<&SharedRecoveryCheckpoint, SelectedCheckpointInstallationDenial> {
        use SelectedCheckpointInstallationDenial as Denial;
        match &self.checkpoint_selection {
            RecoveryCheckpointSelection::Selected { base, shared, .. }
                if base.checkpoint() == facts
                    && shared.facts() == *facts
                    && shared.matches_owner(&self.residency) =>
            {
                Ok(shared)
            }
            RecoveryCheckpointSelection::Unobserved => Err(Denial::Unobserved),
            _ => Err(Denial::CheckpointMismatch),
        }
    }

    pub(in crate::physical_runtime) fn require_checkpoint_owner(
        &self,
        shared: &SharedRecoveryCheckpoint,
    ) -> Result<(), SelectedCheckpointInstallationDenial> {
        if !shared.matches_owner(&self.residency) {
            return Err(SelectedCheckpointInstallationDenial::PoolMismatch);
        }
        self.require_selected_checkpoint(&shared.facts())
            .map(|_| ())
    }

    pub(in crate::physical_runtime) fn require_observed_checkpoint(
        &self,
    ) -> Result<(), SelectedCheckpointInstallationDenial> {
        if matches!(
            self.checkpoint_selection,
            RecoveryCheckpointSelection::Unobserved
        ) {
            return Err(SelectedCheckpointInstallationDenial::Unobserved);
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn checkpoint_binding_basis(
        &self,
    ) -> Option<&StoreRecoveryCheckpointBindingBasis> {
        match &self.checkpoint_selection {
            RecoveryCheckpointSelection::Selected { binding, .. } => Some(binding),
            RecoveryCheckpointSelection::Unobserved | RecoveryCheckpointSelection::Absent => None,
        }
    }

    /// Binding evidence is retained once beside the same checkpoint backing.
    pub fn owned_recovery_heap_bytes(&self) -> Option<u64> {
        match &self.checkpoint_selection {
            RecoveryCheckpointSelection::Selected {
                shared, binding, ..
            } => binding
                .owned_heap_bytes()?
                .checked_add(shared.owned_heap_bytes()?),
            RecoveryCheckpointSelection::Unobserved | RecoveryCheckpointSelection::Absent => {
                Some(0)
            }
        }
    }
}

impl RecoveryCheckpointSelection {
    pub(super) fn into_ownership(self) -> Option<RecoveryCheckpointOwnership> {
        match self {
            Self::Unobserved => None,
            Self::Absent => Some(RecoveryCheckpointOwnership::Absent),
            Self::Selected { shared, .. } => Some(RecoveryCheckpointOwnership::Present(shared)),
        }
    }
}

impl RecoveryCheckpointOwnership {
    pub(in crate::physical_runtime) fn checkpoint(&self) -> Option<&SharedRecoveryCheckpoint> {
        match self {
            Self::Absent => None,
            Self::Present(shared) => Some(shared),
        }
    }
}

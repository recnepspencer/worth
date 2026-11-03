//! The durable Branch recipe owns its exact Native input independently of history.
use crate::durability::WorldDurableMediumBinding;
use crate::inspection::RuntimeWorldRetentionKey;
use crate::retention::HistoryRetentionObligation;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
mod occurrences;
use occurrences::CheckpointOccurrences;
pub(crate) use occurrences::PreparedCheckpointOccurrence;

/// This holder's obligation was released; other holders may still retain the basis.
#[derive(Debug)]
pub struct WorldCheckpointOriginRetirement {
    keys: [RuntimeWorldRetentionKey; 2],
    occurrences: CheckpointOccurrences,
}
impl WorldCheckpointOriginRetirement {
    /// Exact registry entries now eligible for the owning close's reclamation.
    /// Other live obligations may still retain either entry.
    pub fn retention_keys(&self) -> impl Iterator<Item = &RuntimeWorldRetentionKey> {
        self.keys.iter().chain(self.occurrences.retired_keys())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCheckpointOriginRetirementDenial {
    Busy,
    NotHomeBound,
    OwnerNotClosed,
    NotInstalled,
    AlreadyRetired,
    RecoveredCustody(crate::lifecycle::RuntimeWorldCloseDenial),
}

#[derive(Debug, Default)]
pub(crate) enum WorldCheckpointOriginState {
    #[default]
    Uninstalled,
    Held(WorldCheckpointOrigin),
    Retired,
}

#[derive(Debug)]
pub(crate) struct WorldCheckpointOrigin {
    basis: AdmittedRelationalBranchBasis,
    pins: HistoryRetentionObligation,
    occurrences: CheckpointOccurrences,
}
impl WorldCheckpointOrigin {
    pub(crate) fn new(
        basis: &crate::basis::AdmittedCompositeRuntimeWorldBasis,
        pins: HistoryRetentionObligation,
    ) -> Self {
        assert!(
            pins.matches_basis(basis),
            "the recipe retains its exact component pair"
        );
        Self {
            basis: basis.relational_basis().clone(),
            pins,
            occurrences: CheckpointOccurrences::default(),
        }
    }
    pub(crate) fn retire(self) -> WorldCheckpointOriginRetirement {
        let Self {
            basis,
            pins,
            mut occurrences,
        } = self;
        occurrences.retire();
        let keys = pins.retention_keys();
        drop(pins);
        drop(basis);
        WorldCheckpointOriginRetirement { keys, occurrences }
    }
}

impl WorldDurableMediumBinding {
    /// Move the SAME prepared obligation after actual root installation.
    pub(crate) fn retain_checkpoint_origin(&self, retained: WorldCheckpointOrigin) {
        let mut origin = self
            .checkpoint_origin
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match &*origin {
            WorldCheckpointOriginState::Uninstalled => {
                *origin = WorldCheckpointOriginState::Held(retained);
            }
            WorldCheckpointOriginState::Held(_) | WorldCheckpointOriginState::Retired => {
                panic!("the original Branch recipe installs its custody exactly once");
            }
        }
    }

    /// Move the original request-funded successor pair into the recipe owner.
    pub(crate) fn retain_checkpoint_occurrence(&self, occurrence: PreparedCheckpointOccurrence) {
        let mut origin = self
            .checkpoint_origin
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match &mut *origin {
            WorldCheckpointOriginState::Held(origin) => origin.occurrences.retain(occurrence),
            WorldCheckpointOriginState::Uninstalled | WorldCheckpointOriginState::Retired => {
                panic!("a durable occurrence belongs to an installed live recipe");
            }
        }
    }

    pub(crate) fn take_checkpoint_origin(
        &self,
    ) -> Result<WorldCheckpointOrigin, WorldCheckpointOriginRetirementDenial> {
        use WorldCheckpointOriginRetirementDenial as Denial;
        let mut state = self
            .checkpoint_origin
            .try_lock()
            .map_err(|_| Denial::Busy)?;
        match std::mem::replace(&mut *state, WorldCheckpointOriginState::Retired) {
            WorldCheckpointOriginState::Held(origin) => Ok(origin),
            WorldCheckpointOriginState::Uninstalled => {
                *state = WorldCheckpointOriginState::Uninstalled;
                Err(Denial::NotInstalled)
            }
            WorldCheckpointOriginState::Retired => Err(Denial::AlreadyRetired),
        }
    }
}

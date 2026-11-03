//! The durable Branch recipe owns its exact Native input independently of history.
use crate::durability::WorldDurableMediumBinding;
use crate::inspection::RuntimeWorldRetentionKey;
use crate::retention::HistoryRetentionObligation;
mod occurrences;
use occurrences::CheckpointOccurrences;
pub(crate) use occurrences::PreparedCheckpointOccurrence;

/// This holder's obligation was released; other holders may still retain the basis.
#[derive(Debug)]
pub struct WorldCheckpointOriginRetirement {
    original: WorldCheckpointOrigin,
}
impl WorldCheckpointOriginRetirement {
    /// Exact registry entries now eligible for the owning close's reclamation.
    /// Other live obligations may still retain either entry.
    pub fn retention_keys(&self) -> impl Iterator<Item = &RuntimeWorldRetentionKey> {
        self.original
            .keys
            .iter()
            .chain(self.original.occurrences.retired_keys())
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
    basis: crate::basis::NativeBasisRetirement,
    retired_components: Option<crate::retention::component_obligation::RetiringHistoryComponents>,
    keys: [RuntimeWorldRetentionKey; 2],
    complete: bool,
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
            basis: crate::basis::NativeBasisRetirement::new(basis.relational_basis().clone()),
            retired_components: None,
            keys: pins.retention_keys(),
            complete: false,
            pins,
            occurrences: CheckpointOccurrences::default(),
        }
    }
    pub(crate) fn into_retired(self) -> Result<WorldCheckpointOriginRetirement, Self> {
        if self.complete {
            Ok(WorldCheckpointOriginRetirement { original: self })
        } else {
            Err(self)
        }
    }

    pub(crate) fn advance_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, crate::lifecycle::RuntimeWorldCloseDenial> {
        use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
        use worth_relational::facade::branch::RelationalBranchBasisRetirementDenial as NativeDenial;
        if self.complete {
            return Ok(true);
        }
        request.consult().map_err(Denial::Execution)?;
        if self.retired_components.is_none() {
            self.retired_components = Some(
                self.pins
                    .try_begin_close_retirement()
                    .map_err(|()| Denial::Busy)?,
            );
        }
        if !self
            .basis
            .advance(request, native)
            .map_err(|cause| match cause {
                NativeDenial::Busy => Denial::Busy,
                NativeDenial::Execution(cause) => Denial::Execution(cause),
                NativeDenial::ForeignOwner => Denial::CloseProgressInvariant,
            })?
        {
            return Ok(false);
        }
        if !self
            .retired_components
            .as_mut()
            .expect("original component phases")
            .advance(request)?
        {
            return Ok(false);
        }
        self.complete = self.occurrences.advance_retirement(request, native)?;
        Ok(self.complete)
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

    pub(crate) fn retire_checkpoint_origin(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial> {
        use WorldCheckpointOriginRetirementDenial as Denial;
        let mut state = self
            .checkpoint_origin
            .try_lock()
            .map_err(|_| Denial::Busy)?;
        let origin = match &mut *state {
            WorldCheckpointOriginState::Held(origin) => origin,
            WorldCheckpointOriginState::Uninstalled => return Err(Denial::NotInstalled),
            WorldCheckpointOriginState::Retired => return Err(Denial::AlreadyRetired),
        };
        while !origin
            .advance_retirement(request, native)
            .map_err(Denial::RecoveredCustody)?
        {}
        let WorldCheckpointOriginState::Held(original) =
            std::mem::replace(&mut *state, WorldCheckpointOriginState::Retired)
        else {
            unreachable!("same completed origin")
        };
        original.into_retired().map_err(|_| {
            Denial::RecoveredCustody(
                crate::lifecycle::RuntimeWorldCloseDenial::CloseProgressInvariant,
            )
        })
    }
}

//! The durable Branch recipe owns its exact Native input independently of history.
use crate::durability::WorldDurableMediumBinding;
use crate::retention::HistoryRetentionObligation;
use worth_execution::{drain_cooperative_retirement, try_lock_retirement, CooperativeStep};
mod occurrences;
use occurrences::CheckpointOccurrences;
pub(crate) use occurrences::PreparedCheckpointOccurrence;

/// Completed retirement carrying the funded origin and occurrence allocations
/// until the owning Home releases them.
#[derive(Debug)]
pub struct WorldCheckpointOriginRetirement {
    _root: Option<WorldCheckpointOrigin>,
    _occurrences: CheckpointOccurrences,
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
    Held(WorldCheckpointOrigin, CheckpointOccurrences),
    HeldTail(CheckpointOccurrences),
    Retired,
}

#[derive(Debug)]
pub(crate) struct WorldCheckpointOrigin {
    pub(super) commit_ordinal: u64,
    basis: crate::basis::NativeBasisRetirement,
    retired_components: Option<crate::retention::component_obligation::RetiringHistoryComponents>,
    pub(super) complete: bool,
    pins: HistoryRetentionObligation,
}
impl WorldCheckpointOrigin {
    #[cfg(test)]
    pub(crate) fn new(
        basis: &crate::basis::AdmittedCompositeRuntimeWorldBasis,
        pins: HistoryRetentionObligation,
    ) -> Self {
        Self::with_commit(0, basis, pins)
    }
    pub(crate) fn with_commit(
        commit_ordinal: u64,
        basis: &crate::basis::AdmittedCompositeRuntimeWorldBasis,
        pins: HistoryRetentionObligation,
    ) -> Self {
        assert!(
            pins.matches_basis(basis),
            "the recipe retains its exact component pair"
        );
        Self {
            commit_ordinal,
            basis: crate::basis::NativeBasisRetirement::new(basis.relational_basis().clone()),
            retired_components: None,
            complete: false,
            pins,
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
        self.complete = true;
        Ok(true)
    }
}

impl WorldDurableMediumBinding {
    /// Move the SAME prepared obligation after actual root installation.
    pub(crate) fn retain_checkpoint_origin(&self, retained: PreparedCheckpointOccurrence) {
        let mut origin = self
            .checkpoint_origin
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match &mut *origin {
            WorldCheckpointOriginState::Uninstalled => {
                *origin = WorldCheckpointOriginState::Held(
                    retained.into_origin(),
                    CheckpointOccurrences::default(),
                );
            }
            WorldCheckpointOriginState::Held(_, occurrences)
            | WorldCheckpointOriginState::HeldTail(occurrences) => occurrences.retain(retained),
            WorldCheckpointOriginState::Retired => {
                panic!("a retired Home cannot install a branch recipe")
            }
        }
    }

    /// Ensure an open Home with all recovered commits already disposed still
    /// registers an installed (empty) checkpoint origin set for close.
    pub(crate) fn mark_checkpoint_origin_installed(&self) {
        let mut origin = self
            .checkpoint_origin
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if matches!(*origin, WorldCheckpointOriginState::Uninstalled) {
            *origin = WorldCheckpointOriginState::HeldTail(CheckpointOccurrences::default());
        }
    }

    /// Move the original request-funded successor pair into the recipe owner.
    pub(crate) fn retain_checkpoint_occurrence(&self, occurrence: PreparedCheckpointOccurrence) {
        let mut origin = self
            .checkpoint_origin
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match &mut *origin {
            WorldCheckpointOriginState::Held(_, occurrences)
            | WorldCheckpointOriginState::HeldTail(occurrences) => occurrences.retain(occurrence),
            WorldCheckpointOriginState::Uninstalled | WorldCheckpointOriginState::Retired => {
                panic!("a durable occurrence belongs to an installed live recipe");
            }
        }
    }

    /// Release checkpoint origin holdings for durably disposed commits.
    /// Drops all released basis/pin obligations outside the origin lock.
    pub(crate) fn dispose_checkpoint_commits(&self, ordinals: &std::collections::BTreeSet<u64>) {
        if ordinals.is_empty() {
            return;
        }
        let (removed_root, removed) = {
            let mut origin = self
                .checkpoint_origin
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            match &mut *origin {
                WorldCheckpointOriginState::Held(root, occurrences) => {
                    let removed = occurrences.dispose_commits(ordinals);
                    if ordinals.contains(&root.commit_ordinal) {
                        let WorldCheckpointOriginState::Held(root, occurrences) = std::mem::replace(
                            &mut *origin,
                            WorldCheckpointOriginState::HeldTail(CheckpointOccurrences::default()),
                        ) else {
                            unreachable!("same held state")
                        };
                        *origin = WorldCheckpointOriginState::HeldTail(occurrences);
                        (Some(root), removed)
                    } else {
                        (None, removed)
                    }
                }
                WorldCheckpointOriginState::HeldTail(occurrences) => {
                    let removed = occurrences.dispose_commits(ordinals);
                    (None, removed)
                }
                WorldCheckpointOriginState::Uninstalled | WorldCheckpointOriginState::Retired => {
                    (None, Vec::new())
                }
            }
        };
        drop((removed_root, removed));
    }

    pub(crate) fn retire_checkpoint_origin(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial> {
        use WorldCheckpointOriginRetirementDenial as Denial;
        let mut state = try_lock_retirement(&self.checkpoint_origin).map_err(|_| Denial::Busy)?;
        if let WorldCheckpointOriginState::Held(root, _) = &mut *state {
            drain_origin(request, |request| root.advance_retirement(request, native))?;
        }
        match &mut *state {
            WorldCheckpointOriginState::Held(_, occurrences)
            | WorldCheckpointOriginState::HeldTail(occurrences) => {
                drain_origin(request, |request| {
                    occurrences.advance_retirement(request, native)
                })?;
            }
            WorldCheckpointOriginState::Uninstalled => return Err(Denial::NotInstalled),
            WorldCheckpointOriginState::Retired => return Err(Denial::AlreadyRetired),
        }
        let (_root, _occurrences) =
            match std::mem::replace(&mut *state, WorldCheckpointOriginState::Retired) {
                WorldCheckpointOriginState::Held(root, occurrences) => (Some(root), occurrences),
                WorldCheckpointOriginState::HeldTail(occurrences) => (None, occurrences),
                _ => unreachable!("same completed origin"),
            };
        Ok(WorldCheckpointOriginRetirement {
            _root,
            _occurrences,
        })
    }
}

fn drain_origin(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    mut advance: impl FnMut(
        &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, crate::lifecycle::RuntimeWorldCloseDenial>,
) -> Result<(), WorldCheckpointOriginRetirementDenial> {
    use crate::lifecycle::RuntimeWorldCloseDenial as Close;
    use WorldCheckpointOriginRetirementDenial as Denial;
    drain_cooperative_retirement(
        request,
        |request| match advance(request) {
            Ok(true) => Ok(CooperativeStep::Complete(())),
            Ok(false) => Ok(CooperativeStep::Continue),
            Err(Close::Busy) => Ok(CooperativeStep::Busy),
            Err(Close::Execution(stop)) => Ok(CooperativeStep::Stopped(stop)),
            Err(error) => Err(Denial::RecoveredCustody(error)),
        },
        || Denial::RecoveredCustody(Close::Busy),
        |stop| Denial::RecoveredCustody(Close::Execution(stop)),
    )
}

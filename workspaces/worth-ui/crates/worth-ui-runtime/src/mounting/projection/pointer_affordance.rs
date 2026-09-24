use crate::runtime::pointer_affordance::{
    UiPointerAffordanceObservationIdentity, UiPointerAffordanceSnapshot,
};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use worth_ui_host_contract::*;

mod fragment;
use fragment::lower_fragment;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiMountedPointerAffordanceWork {
    pub(crate) observations_examined: usize,
    pub(crate) targets_examined: usize,
    pub(crate) membership_key_probes: usize,
    pub(crate) surfaces_changed: usize,
}

#[derive(Clone, Copy)]
struct RetainedPointer {
    frame: UiMountedFrameIdentity,
    binding: UiSurfaceBindingGeneration,
    mechanic: UiMountedPointerAffordanceMechanic,
    reconstruction: bool,
}

/// What the next lowering is asked to publish.
#[derive(Clone)]
enum UiStagedPointerOutput {
    /// Nothing staged: lowering republishes the retained rows.
    Unstaged,
    /// An empty owner staged nothing to publish and nothing to admit.
    Empty,
    Staged(StagedPointers),
}

#[derive(Clone)]
struct StagedPointers {
    rows: Rc<[UiMountedPointerAffordanceMechanic]>,
    surfaces: Rc<BTreeSet<UiSemanticSurfaceIdentity>>,
    observation: Option<UiPointerAffordanceObservationIdentity>,
    admitted_surfaces: Box<[UiSemanticSurfaceIdentity]>,
}

/// Independent pointer output rows, with at most one row per bound surface.
/// Candidate copies share their predecessor until aggregate output admission.
#[derive(Clone)]
pub(crate) struct UiMountedPointerAffordanceState {
    rows: Rc<BTreeMap<UiSemanticSurfaceIdentity, RetainedPointer>>,
    admitted: Rc<BTreeMap<UiSemanticSurfaceIdentity, UiPointerAffordanceObservationIdentity>>,
    staged: UiStagedPointerOutput,
    work: UiMountedPointerAffordanceWork,
}

impl UiMountedPointerAffordanceState {
    /// An owner with no predecessor: nothing retained, admitted or staged.
    pub(in crate::mounting) fn empty() -> Self {
        Self {
            rows: Rc::default(),
            admitted: Rc::default(),
            staged: UiStagedPointerOutput::Unstaged,
            work: UiMountedPointerAffordanceWork::default(),
        }
    }

    pub(in crate::mounting::projection) fn settle_without_output(&self) -> Option<Self> {
        if !self.rows.is_empty() || !self.admitted.is_empty() {
            return None;
        }
        match &self.staged {
            UiStagedPointerOutput::Unstaged => return None,
            UiStagedPointerOutput::Empty => {}
            UiStagedPointerOutput::Staged(staged) => {
                if !staged.rows.is_empty()
                    || staged.observation.is_some()
                    || !staged.admitted_surfaces.is_empty()
                {
                    return None;
                }
            }
        }
        let mut next = self.clone();
        next.staged = UiStagedPointerOutput::Unstaged;
        next.work.surfaces_changed = 0;
        Some(next)
    }

    #[cfg(test)]
    pub(in crate::mounting) fn retained_mechanic_for_test(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<UiMountedPointerAffordanceMechanic> {
        self.rows.get(&surface).map(|row| row.mechanic)
    }

    pub(in crate::mounting) fn matches_snapshot(
        &self,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        snapshot: Option<&UiPointerAffordanceSnapshot>,
    ) -> bool {
        let desired = snapshot
            .into_iter()
            .flat_map(UiPointerAffordanceSnapshot::projections)
            .find(|row| row.surface() == surface && row.target().is_some());
        match (self.rows.get(&surface), desired) {
            (None, None) => true,
            (Some(retained), Some(desired)) => {
                let same_target = Some(retained.mechanic.target()) == desired.target();
                retained.binding == binding
                    && !retained.reconstruction
                    && retained.mechanic.pointer() == desired.pointer()
                    && same_target
                    && desired.decided_family() == Some(retained.mechanic.family())
            }
            _ => false,
        }
    }

    /// The affordance this owner holds for `target` on `surface`, if its row
    /// names that target.
    pub(in crate::mounting) fn published_family(
        &self,
        surface: UiSemanticSurfaceIdentity,
        target: UiMountedInstanceIdentity,
    ) -> Option<UiPointerAffordanceFamily> {
        let row = self.rows.get(&surface)?;
        (row.mechanic.target() == target).then(|| row.mechanic.family())
    }

    pub(in crate::mounting) fn stage(
        &mut self,
        desired: Vec<UiMountedPointerAffordanceMechanic>,
        work: UiMountedPointerAffordanceWork,
        surfaces: impl IntoIterator<Item = UiSemanticSurfaceIdentity>,
        observation: Option<&UiPointerAffordanceSnapshot>,
        admitted_surfaces: Vec<UiSemanticSurfaceIdentity>,
    ) {
        if self.rows.is_empty()
            && self.admitted.is_empty()
            && desired.is_empty()
            && observation.is_none()
            && admitted_surfaces.is_empty()
        {
            self.staged = UiStagedPointerOutput::Empty;
            self.work = work;
            return;
        }
        self.staged = UiStagedPointerOutput::Staged(StagedPointers {
            rows: desired.into(),
            surfaces: Rc::new(surfaces.into_iter().collect()),
            observation: observation.map(UiPointerAffordanceSnapshot::observation_identity),
            admitted_surfaces: admitted_surfaces.into(),
        });
        self.work = work;
    }

    pub(in crate::mounting) fn has_admitted_observation(
        &self,
        surface: UiSemanticSurfaceIdentity,
        observation: &UiPointerAffordanceSnapshot,
    ) -> bool {
        self.admitted
            .get(&surface)
            .is_some_and(|admitted| *admitted == observation.observation_identity())
    }

    pub(in crate::mounting) fn admitted_surface_count(&self) -> usize {
        self.admitted.len()
    }

    pub(in crate::mounting) fn retain_observation_admission(
        &mut self,
        observation: UiPointerAffordanceObservationIdentity,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) {
        // Admission tracks this one coherent observation, never a history that
        // could authorize an older target after a later empty snapshot.
        self.admitted = Rc::new(
            surfaces
                .iter()
                .map(|surface| (*surface, observation.clone()))
                .collect(),
        );
    }

    pub(in crate::mounting) fn require_reconstruction(&mut self) {
        for row in Rc::make_mut(&mut self.rows).values_mut() {
            row.reconstruction = true;
        }
    }

    pub(in crate::mounting) fn after_surface_deregistration(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Self {
        let mut next = self.clone();
        Rc::make_mut(&mut next.rows).remove(&surface);
        Rc::make_mut(&mut next.admitted).remove(&surface);
        if let UiStagedPointerOutput::Staged(staged) = &mut next.staged {
            staged.rows = staged
                .rows
                .iter()
                .copied()
                .filter(|row| row.surface() != surface)
                .collect::<Vec<_>>()
                .into();
            Rc::make_mut(&mut staged.surfaces).remove(&surface);
            staged.admitted_surfaces = staged
                .admitted_surfaces
                .iter()
                .copied()
                .filter(|candidate| *candidate != surface)
                .collect();
        }
        next
    }

    pub(in crate::mounting) const fn work(&self) -> UiMountedPointerAffordanceWork {
        self.work
    }

    pub(in crate::mounting::projection) fn lower(
        &self,
        frame: &super::UiMountedProjectionFrame,
        presentation: UiMountedPresentationAttemptIdentity,
        bindings: &BTreeMap<UiSemanticSurfaceIdentity, UiMountedSurfaceBindingRequirement>,
    ) -> Result<(Self, Vec<UiUnpublishedAppearanceFragment>), super::UiMountedAppearanceOutputDenial>
    {
        let staged = match &self.staged {
            UiStagedPointerOutput::Staged(staged) => Some(staged),
            UiStagedPointerOutput::Unstaged | UiStagedPointerOutput::Empty => None,
        };
        let desired = match &self.staged {
            UiStagedPointerOutput::Unstaged => self.rows.values().map(|row| row.mechanic).collect(),
            UiStagedPointerOutput::Empty => Vec::new(),
            UiStagedPointerOutput::Staged(staged) => staged.rows.to_vec(),
        };
        let mut next = self.clone();
        let mut current = BTreeMap::new();
        for mechanic in desired {
            let (receipt, probes) = frame
                .semantic_projection()
                .node_receipt_with_probes(mechanic.target());
            next.work.membership_key_probes += probes;
            if receipt.is_some_and(|receipt| receipt.semantic_surface() == mechanic.surface()) {
                current.insert(mechanic.surface(), mechanic);
            }
        }
        let surfaces = self
            .rows
            .keys()
            .chain(current.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let mut fragments = Vec::new();
        next.work.surfaces_changed = 0;
        for surface in surfaces {
            if staged.is_some_and(|staged| !staged.surfaces.contains(&surface)) {
                continue;
            }
            let Some(binding) = bindings.get(&surface).copied() else {
                if staged.is_some() {
                    return Err(super::UiMountedAppearanceOutputDenial::Transport(
                        UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch,
                    ));
                }
                continue;
            };
            let previous = self
                .rows
                .get(&surface)
                .filter(|row| row.binding == binding.binding());
            let successor = current.get(&surface).copied();
            let changed = previous.map(|row| row.mechanic) != successor;
            let reconstruction = previous.is_some_and(|row| row.reconstruction);
            if (changed || reconstruction && successor.is_some())
                && (previous.is_some() || successor.is_some())
            {
                fragments.push(lower_fragment(
                    frame,
                    presentation,
                    binding,
                    previous,
                    successor,
                    reconstruction,
                )?);
                next.work.surfaces_changed += 1;
            }
            match successor {
                Some(mechanic) if changed || reconstruction => {
                    Rc::make_mut(&mut next.rows).insert(
                        surface,
                        RetainedPointer {
                            frame: frame.frame_identity(),
                            binding: binding.binding(),
                            mechanic,
                            reconstruction: false,
                        },
                    );
                }
                Some(_) => {}
                None => {
                    Rc::make_mut(&mut next.rows).remove(&surface);
                }
            }
        }
        if let Some(staged) = staged {
            if let Some(observation) = &staged.observation {
                next.retain_observation_admission(observation.clone(), &staged.admitted_surfaces);
            } else {
                for surface in staged.surfaces.iter() {
                    Rc::make_mut(&mut next.admitted).remove(surface);
                }
            }
        }
        next.staged = UiStagedPointerOutput::Unstaged;
        Ok((next, fragments))
    }
}

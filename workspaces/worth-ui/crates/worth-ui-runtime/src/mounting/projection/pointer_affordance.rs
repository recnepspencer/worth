use crate::runtime::pointer_affordance::{
    UiPointerAffordanceObservationIdentity, UiPointerAffordanceSnapshot,
};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use worth_ui_host_contract::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiMountedPointerAffordanceWork {
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

#[derive(Clone)]
struct StagedPointers {
    rows: Rc<[UiMountedPointerAffordanceMechanic]>,
    surfaces: Rc<BTreeSet<UiSemanticSurfaceIdentity>>,
    observation: Option<UiPointerAffordanceObservationIdentity>,
    admitted_surfaces: Box<[UiSemanticSurfaceIdentity]>,
}

/// Independent pointer output rows, with at most one row per bound surface.
/// Candidate copies share their predecessor until aggregate output admission.
#[derive(Clone, Default)]
pub(crate) struct UiMountedPointerAffordanceState {
    rows: Rc<BTreeMap<UiSemanticSurfaceIdentity, RetainedPointer>>,
    admitted: Rc<BTreeMap<UiSemanticSurfaceIdentity, UiPointerAffordanceObservationIdentity>>,
    staged: Option<StagedPointers>,
    work: UiMountedPointerAffordanceWork,
}

impl UiMountedPointerAffordanceState {
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
                let family = match desired.family() {
                    crate::declaration::UiPointerAffordance::Default => {
                        UiPointerAffordanceFamily::Default
                    }
                    crate::declaration::UiPointerAffordance::Activation => {
                        UiPointerAffordanceFamily::Activation
                    }
                };
                retained.binding == binding
                    && !retained.reconstruction
                    && retained.mechanic.pointer() == desired.pointer()
                    && Some(retained.mechanic.target()) == desired.target()
                    && retained.mechanic.family() == family
            }
            _ => false,
        }
    }

    pub(in crate::mounting) fn stage(
        &mut self,
        desired: Vec<UiMountedPointerAffordanceMechanic>,
        work: UiMountedPointerAffordanceWork,
        surfaces: impl IntoIterator<Item = UiSemanticSurfaceIdentity>,
        observation: Option<&UiPointerAffordanceSnapshot>,
        admitted_surfaces: Vec<UiSemanticSurfaceIdentity>,
    ) {
        self.staged = Some(StagedPointers {
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
        if let Some(staged) = next.staged.as_mut() {
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
        let desired = self
            .staged
            .as_ref()
            .map(|staged| staged.rows.to_vec())
            .unwrap_or_else(|| self.rows.values().map(|row| row.mechanic).collect());
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
            if self
                .staged
                .as_ref()
                .is_some_and(|staged| !staged.surfaces.contains(&surface))
            {
                continue;
            }
            let Some(binding) = bindings.get(&surface).copied() else {
                if self.staged.is_some() {
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
            if changed || reconstruction && successor.is_some() {
                if previous.is_some() || successor.is_some() {
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
        if let Some(staged) = &self.staged {
            if let Some(observation) = &staged.observation {
                next.retain_observation_admission(observation.clone(), &staged.admitted_surfaces);
            } else {
                for surface in staged.surfaces.iter() {
                    Rc::make_mut(&mut next.admitted).remove(surface);
                }
            }
        }
        next.staged = None;
        Ok((next, fragments))
    }
}

fn lower_fragment(
    frame: &super::UiMountedProjectionFrame,
    presentation: UiMountedPresentationAttemptIdentity,
    binding: UiMountedSurfaceBindingRequirement,
    previous: Option<&RetainedPointer>,
    successor: Option<UiMountedPointerAffordanceMechanic>,
    reconstruction: bool,
) -> Result<UiUnpublishedAppearanceFragment, super::UiMountedAppearanceOutputDenial> {
    let denial = || super::UiMountedAppearanceOutputDenial::PointerLowering;
    let old = previous.map(|row| UiMountedAppearanceMechanic::Pointer(row.mechanic));
    let new = successor.map(UiMountedAppearanceMechanic::Pointer);
    let mut changes = Vec::new();
    match (&old, &new) {
        (Some(old), Some(new)) if old.identity() == new.identity() => {
            changes.push(
                UiMountedAppearanceMechanicChange::replacement(old.identity(), new.clone())
                    .ok_or_else(denial)?,
            );
        }
        _ => {
            if let Some(old) = &old {
                changes.push(UiMountedAppearanceMechanicChange::Remove(old.identity()));
            }
            if let Some(new) = &new {
                changes.push(UiMountedAppearanceMechanicChange::Insert(new.clone()));
            }
        }
    }
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        binding.semantic_surface(),
        presentation,
        0,
        0,
        [],
    )
    .map_err(|_| denial())?;
    let output = UiMountedAppearanceFrame::from_runtime_mounting(
        frame.frame_identity(),
        binding.semantic_surface(),
        new,
        order,
    )
    .map_err(|_| denial())?;
    let manifest = previous
        .map(|_| {
            UiMountedAppearancePredecessorManifest::from_runtime_mounting(
                old.iter().map(UiMountedAppearanceMechanic::identity),
                [],
            )
            .ok_or_else(denial)
        })
        .transpose()?;
    let posture = if previous.is_none() {
        UiMountedAppearanceWorkPosture::Initial
    } else if reconstruction {
        UiMountedAppearanceWorkPosture::Reconstruction
    } else {
        UiMountedAppearanceWorkPosture::Delta
    };
    let predecessor = previous.map(|row| row.frame);
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        predecessor,
        manifest,
        output,
        changes,
        [],
        previous.is_none(),
    )
    .ok_or_else(denial)?;
    let affinity = UiMountedPresentationAffinity::from_runtime_mounting(
        predecessor,
        frame.frame_identity(),
        binding,
        frame.content_generation(),
        None,
    );
    let pointer = successor
        .or(previous.map(|row| row.mechanic))
        .ok_or_else(denial)?
        .pointer();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: binding.semantic_surface(),
            pointer,
        },
        work,
        [],
        binding,
        affinity,
    )
    .map_err(super::UiMountedAppearanceOutputDenial::Transport)
}

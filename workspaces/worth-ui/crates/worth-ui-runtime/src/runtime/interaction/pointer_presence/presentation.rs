#![allow(
    dead_code,
    reason = "Gate 1 retains pointer-presence presentation mechanics for later mounted cutover"
)]

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostSurfacePosition, UiMountedCanonicalBox,
    UiMountedCoordinateSpace, UiMountedInstanceIdentity,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

pub(crate) const UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY: usize = 2_048;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPointerPresenceGeometry {
    bounds: UiMountedCanonicalBox,
    clip_bounds: UiMountedCanonicalBox,
}

// Canonical geometry rejects non-finite components, so equality is reflexive.
impl Eq for UiPointerPresenceGeometry {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPointerPresenceGeometryCandidate {
    instance: UiMountedInstanceIdentity,
    old: Option<UiPointerPresenceGeometry>,
    new: Option<UiPointerPresenceGeometry>,
}

// Canonical geometry rejects non-finite components, so equality is reflexive.
impl Eq for UiPointerPresenceGeometryCandidate {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPointerPresencePresentationTrigger {
    presentation: UiHostObservationPresentationBasis,
    changed_instances: Box<[UiMountedInstanceIdentity]>,
    geometry_candidates: Box<[UiPointerPresenceGeometryCandidate]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPointerPresencePresentationTriggerDenial {
    EmptyChangedNeighborhood,
    ChangedNeighborhoodCapacityExceeded { observed: usize, maximum: usize },
}

impl UiPointerPresencePresentationTrigger {
    pub(crate) fn new(
        presentation: UiHostObservationPresentationBasis,
        changed_input: &[UiMountedInstanceIdentity],
    ) -> Result<Self, UiPointerPresencePresentationTriggerDenial> {
        let mut changed_instances = Vec::new();
        for instance in changed_input.iter().copied() {
            if changed_instances.contains(&instance) {
                continue;
            }
            admit_changed_instance(&mut changed_instances, instance)?;
        }
        changed_instances.sort_unstable();
        if changed_instances.is_empty() {
            return Err(UiPointerPresencePresentationTriggerDenial::EmptyChangedNeighborhood);
        }
        let geometry_candidates = changed_instances
            .iter()
            .copied()
            .map(UiPointerPresenceGeometryCandidate::identity_only)
            .collect();
        Ok(Self {
            presentation,
            changed_instances: changed_instances.into_boxed_slice(),
            geometry_candidates,
        })
    }

    pub(crate) fn new_with_geometry(
        presentation: UiHostObservationPresentationBasis,
        candidates: &[UiPointerPresenceGeometryCandidate],
    ) -> Result<Self, UiPointerPresencePresentationTriggerDenial> {
        if candidates.is_empty() {
            return Err(UiPointerPresencePresentationTriggerDenial::EmptyChangedNeighborhood);
        }
        let mut canonical: Vec<UiPointerPresenceGeometryCandidate> = Vec::new();
        for candidate in candidates.iter().copied() {
            if let Some(previous) = canonical
                .iter_mut()
                .find(|previous| previous.instance == candidate.instance)
            {
                previous.old = previous.old.or(candidate.old);
                previous.new = previous.new.or(candidate.new);
            } else {
                admit_changed_instance(&mut canonical, candidate)?;
            }
        }
        canonical.sort_unstable_by_key(|candidate| candidate.instance);
        let changed_instances = canonical
            .iter()
            .map(|candidate| candidate.instance)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(Self {
            presentation,
            changed_instances,
            geometry_candidates: canonical.into_boxed_slice(),
        })
    }

    pub(crate) const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(crate) fn changed_instances(&self) -> &[UiMountedInstanceIdentity] {
        &self.changed_instances
    }

    pub(crate) fn affects_position(
        &self,
        position: UiHostSurfacePosition,
        previous_target: Option<UiMountedInstanceIdentity>,
    ) -> bool {
        self.geometry_candidates.iter().any(|candidate| {
            Some(candidate.instance) == previous_target
                || candidate
                    .old
                    .is_some_and(|geometry| geometry.contains(position))
                || candidate
                    .new
                    .is_some_and(|geometry| geometry.contains(position))
        })
    }
}

fn admit_changed_instance<T>(
    changed_instances: &mut Vec<T>,
    instance: T,
) -> Result<(), UiPointerPresencePresentationTriggerDenial> {
    if changed_instances.len() == UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY {
        return Err(
            UiPointerPresencePresentationTriggerDenial::ChangedNeighborhoodCapacityExceeded {
                observed: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY + 1,
                maximum: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY,
            },
        );
    }
    changed_instances.push(instance);
    Ok(())
}

impl super::UiPointerPresenceOwner {
    pub(crate) fn retest_committed_presentation(
        &mut self,
        trigger: &UiPointerPresencePresentationTrigger,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> usize {
        if mounted
            .validate_current_frame(trigger.presentation().frame())
            .is_err()
            || mounted
                .validate_binding(trigger.presentation().binding())
                .is_err()
        {
            return 0;
        }
        let pointers = self
            .pointers
            .iter()
            .filter_map(|(pointer, record)| {
                trigger
                    .affects_position(record.position, record.target)
                    .then_some(*pointer)
            })
            .collect::<Vec<_>>();
        let mut changed = 0;
        for pointer in pointers {
            let Some((kind, sequence, position)) = self
                .pointers
                .get(&pointer)
                .map(|record| (record.kind, record.sequence, record.position))
            else {
                continue;
            };
            let resolved = match crate::runtime::interaction::targeting::resolve_presented_target(
                mounted,
                trigger.presentation(),
                position,
            ) {
                Ok(target) => Some((
                    target.surface(),
                    target.binding(),
                    target.mounted_instance(),
                    target.node_receipt(),
                )),
                Err(
                    crate::runtime::interaction::targeting::UiInteractionTargetingDenial::NoTarget {
                        ..
                    },
                ) => None,
                Err(_) => continue,
            };
            if self
                .record_pointer_target(
                    pointer,
                    kind,
                    sequence,
                    position,
                    trigger.presentation(),
                    resolved,
                    generation,
                )
                .is_ok_and(|transition| transition.is_some())
            {
                changed += 1;
            }
        }
        changed
    }
}

impl UiPointerPresenceGeometry {
    pub(crate) const fn new(
        bounds: UiMountedCanonicalBox,
        clip_bounds: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            bounds,
            clip_bounds,
        }
    }

    pub(crate) const fn bounds(self) -> UiMountedCanonicalBox {
        self.bounds
    }

    pub(crate) const fn clip_bounds(self) -> UiMountedCanonicalBox {
        self.clip_bounds
    }

    fn contains(self, position: UiHostSurfacePosition) -> bool {
        if position.basis().coordinate_space()
            != worth_ui_host_contract::UiHostSurfaceCoordinateSpace::Viewport
            || position.basis().coordinate_unit()
                != worth_ui_host_contract::UiHostSurfaceCoordinateUnit::LogicalPoint
        {
            return false;
        }
        let point = [
            position.x_subpixels() as f64 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64,
            position.y_subpixels() as f64 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64,
        ];
        contains(self.bounds, point) && contains(self.clip_bounds, point)
    }
}

impl UiPointerPresenceGeometryCandidate {
    pub(crate) const fn new(
        instance: UiMountedInstanceIdentity,
        old: Option<UiPointerPresenceGeometry>,
        new: Option<UiPointerPresenceGeometry>,
    ) -> Self {
        Self { instance, old, new }
    }

    pub(crate) const fn identity_only(instance: UiMountedInstanceIdentity) -> Self {
        Self::new(instance, None, None)
    }

    pub(crate) const fn instance(self) -> UiMountedInstanceIdentity {
        self.instance
    }

    pub(crate) const fn old(self) -> Option<UiPointerPresenceGeometry> {
        self.old
    }

    pub(crate) const fn new_geometry(self) -> Option<UiPointerPresenceGeometry> {
        self.new
    }
}

fn contains(bounds: UiMountedCanonicalBox, point: [f64; 2]) -> bool {
    bounds.coordinate_space() == UiMountedCoordinateSpace::Viewport
        && point[0] >= f64::from(bounds.x())
        && point[0] < f64::from(bounds.x()) + f64::from(bounds.width())
        && point[1] >= f64::from(bounds.y())
        && point[1] < f64::from(bounds.y()) + f64::from(bounds.height())
}

#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;

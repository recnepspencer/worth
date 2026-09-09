use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostSurfacePosition, UiMountedCanonicalBox,
    UiMountedCoordinateSpace, UiMountedInstanceIdentity,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

#[allow(
    dead_code,
    reason = "The dormant mounted-presentation admission lane uses this raw-input cap."
)]
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
#[allow(
    dead_code,
    reason = "These denials belong to the dormant mounted-presentation admission lane."
)]
pub(crate) enum UiPointerPresencePresentationTriggerDenial {
    EmptyChangedNeighborhood,
    ChangedNeighborhoodCapacityExceeded { observed: usize, maximum: usize },
}

impl UiPointerPresencePresentationTrigger {
    #[allow(
        dead_code,
        reason = "The successor presentation producer will consume this constructor after cutover."
    )]
    pub(crate) fn new(
        presentation: UiHostObservationPresentationBasis,
        changed_input: &[UiMountedInstanceIdentity],
    ) -> Result<Self, UiPointerPresencePresentationTriggerDenial> {
        admit_raw_input(changed_input)?;
        let mut changed_instances = changed_input.to_vec();
        changed_instances.sort_unstable();
        changed_instances.dedup();
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

    #[allow(
        dead_code,
        reason = "The successor presentation producer will consume this geometry constructor after cutover."
    )]
    pub(crate) fn new_with_geometry(
        presentation: UiHostObservationPresentationBasis,
        candidates: &[UiPointerPresenceGeometryCandidate],
    ) -> Result<Self, UiPointerPresencePresentationTriggerDenial> {
        admit_raw_input(candidates)?;
        if candidates.is_empty() {
            return Err(UiPointerPresencePresentationTriggerDenial::EmptyChangedNeighborhood);
        }
        let mut sorted = candidates.to_vec();
        sorted.sort_by_key(|candidate| candidate.instance);
        let mut canonical = Vec::<UiPointerPresenceGeometryCandidate>::with_capacity(sorted.len());
        for candidate in sorted {
            if let Some(previous) = canonical.last_mut() {
                if previous.instance != candidate.instance {
                    canonical.push(candidate);
                    continue;
                }
                previous.old = previous.old.or(candidate.old);
                previous.new = previous.new.or(candidate.new);
            } else {
                canonical.push(candidate);
            }
        }
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

    #[allow(
        dead_code,
        reason = "The successor presentation producer will consume canonical instances after cutover."
    )]
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

#[allow(
    dead_code,
    reason = "Raw-input admission is reserved for the dormant mounted-presentation producer."
)]
fn admit_raw_input<T>(input: &[T]) -> Result<(), UiPointerPresencePresentationTriggerDenial> {
    if input.len() > UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY {
        return Err(
            UiPointerPresencePresentationTriggerDenial::ChangedNeighborhoodCapacityExceeded {
                observed: input.len(),
                maximum: UI_POINTER_PRESENTATION_CHANGED_INSTANCE_CAPACITY,
            },
        );
    }
    Ok(())
}

impl UiPointerPresenceGeometry {
    #[allow(
        dead_code,
        reason = "The successor presentation producer will construct geometry after cutover."
    )]
    pub(crate) const fn new(
        bounds: UiMountedCanonicalBox,
        clip_bounds: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            bounds,
            clip_bounds,
        }
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will read geometry bounds after cutover."
    )]
    pub(crate) const fn bounds(self) -> UiMountedCanonicalBox {
        self.bounds
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will read clip bounds after cutover."
    )]
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
    #[allow(
        dead_code,
        reason = "The successor presentation producer will construct geometry candidates after cutover."
    )]
    pub(crate) const fn new(
        instance: UiMountedInstanceIdentity,
        old: Option<UiPointerPresenceGeometry>,
        new: Option<UiPointerPresenceGeometry>,
    ) -> Self {
        Self { instance, old, new }
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will create identity-only candidates after cutover."
    )]
    pub(crate) const fn identity_only(instance: UiMountedInstanceIdentity) -> Self {
        Self::new(instance, None, None)
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will read candidate identities after cutover."
    )]
    pub(crate) const fn instance(self) -> UiMountedInstanceIdentity {
        self.instance
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will read prior geometry after cutover."
    )]
    pub(crate) const fn old(self) -> Option<UiPointerPresenceGeometry> {
        self.old
    }

    #[allow(
        dead_code,
        reason = "The successor presentation producer will read successor geometry after cutover."
    )]
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

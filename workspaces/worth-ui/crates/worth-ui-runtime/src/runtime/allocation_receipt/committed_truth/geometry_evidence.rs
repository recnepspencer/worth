#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiAllocationAxisAlignedBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: crate::evidence::UiMeasurementCoordinateSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationGeometryKnowledge<T> {
    Known(T),
    NotKnownAtAllocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationAnchorPosture {
    NotAnchored,
    PortalAnchored(crate::runtime::portal::anchored_allocation::UiPortalAnchorIdentity),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiAllocationEdgeReference {
    target: crate::graph::UiGraphNodeIdentity,
    axis: UiAllocationAxis,
    delta: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiCommittedAllocationGeometryEvidence {
    bounds: UiAllocationGeometryKnowledge<UiAllocationAxisAlignedBounds>,
    anchor_posture: UiAllocationAnchorPosture,
    portal_anchor_observation: Option<UiPortalAnchorObservationGeometryEvidence>,
    parent_edges: UiAllocationGeometryKnowledge<Box<[UiAllocationEdgeReference]>>,
    sibling_edges: UiAllocationGeometryKnowledge<Box<[UiAllocationEdgeReference]>>,
    spacing_relationship_ids: UiAllocationGeometryKnowledge<Box<[u64]>>,
    baseline_relationships: UiAllocationGeometryKnowledge<Box<[u64]>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPortalAnchorObservationGeometryEvidence {
    identity: crate::runtime::portal::anchored_allocation::UiPortalAnchorIdentity,
    observed_bounds: UiAllocationAxisAlignedBounds,
}

impl UiCommittedAllocationGeometryEvidence {
    pub(super) fn from_candidate(candidate: &super::UiAllocationCandidate) -> Self {
        let portal_anchor_observation = portal_anchor_observation(candidate);
        let bounds = declared_contract_bounds(candidate)
            .map(UiAllocationGeometryKnowledge::Known)
            .unwrap_or(UiAllocationGeometryKnowledge::NotKnownAtAllocation);
        Self {
            // General layout remains unknown until the later solver. The
            // exact FillViewport contract is the sole current known-bounds
            // case; a portal anchor rectangle remains only an input.
            bounds,
            anchor_posture: portal_anchor_observation
                .map(|observation| UiAllocationAnchorPosture::PortalAnchored(observation.identity))
                .unwrap_or(UiAllocationAnchorPosture::NotAnchored),
            portal_anchor_observation,
            parent_edges: UiAllocationGeometryKnowledge::NotKnownAtAllocation,
            sibling_edges: UiAllocationGeometryKnowledge::NotKnownAtAllocation,
            spacing_relationship_ids: UiAllocationGeometryKnowledge::NotKnownAtAllocation,
            baseline_relationships: UiAllocationGeometryKnowledge::NotKnownAtAllocation,
        }
    }

    pub fn bounds(&self) -> UiAllocationGeometryKnowledge<UiAllocationAxisAlignedBounds> {
        self.bounds
    }

    pub fn anchor_posture(&self) -> UiAllocationAnchorPosture {
        self.anchor_posture
    }

    pub fn portal_anchor_observation(&self) -> Option<UiPortalAnchorObservationGeometryEvidence> {
        self.portal_anchor_observation
    }

    pub fn parent_edges(&self) -> &UiAllocationGeometryKnowledge<Box<[UiAllocationEdgeReference]>> {
        &self.parent_edges
    }

    pub fn sibling_edges(
        &self,
    ) -> &UiAllocationGeometryKnowledge<Box<[UiAllocationEdgeReference]>> {
        &self.sibling_edges
    }

    pub fn spacing_relationship_ids(&self) -> &UiAllocationGeometryKnowledge<Box<[u64]>> {
        &self.spacing_relationship_ids
    }

    pub fn baseline_relationships(&self) -> &UiAllocationGeometryKnowledge<Box<[u64]>> {
        &self.baseline_relationships
    }
}

fn declared_contract_bounds(
    candidate: &super::UiAllocationCandidate,
) -> Option<UiAllocationAxisAlignedBounds> {
    let basis = candidate.measurement_basis();
    let policy = basis.declared_measurement_policy();
    if let Some(crate::declaration::UiDeclaredMeasurementMode::FixedLogicalSize { width, height }) =
        policy.mode()
    {
        return Some(UiAllocationAxisAlignedBounds {
            x: 0.0,
            y: 0.0,
            width: f32::from(width),
            height: f32::from(height),
            coordinate_space: crate::evidence::UiMeasurementCoordinateSpace::GraphNodeLocal,
        });
    }
    if policy.basis_source()
        != Some(crate::declaration::UiDeclaredMeasurementBasisSource::ViewportExtent)
    {
        return None;
    }
    basis.evidence_inputs().iter().find_map(|evidence| {
        let result = evidence.as_host_measurement_result()?;
        let crate::evidence::UiMeasurementValue::ViewportExtent(extent) = result.value() else {
            return None;
        };
        let (x, y, width, height) = match policy.mode()? {
            crate::declaration::UiDeclaredMeasurementMode::FillViewport => {
                (0.0, 0.0, extent.width, extent.height)
            }
            crate::declaration::UiDeclaredMeasurementMode::ViewportInset {
                horizontal_logical_points,
                vertical_logical_points,
            } => {
                let horizontal = f32::from(horizontal_logical_points);
                let vertical = f32::from(vertical_logical_points);
                let width = extent.width - horizontal * 2.0;
                let height = extent.height - vertical * 2.0;
                if width <= 0.0 || height <= 0.0 {
                    return None;
                }
                (horizontal, vertical, width, height)
            }
            crate::declaration::UiDeclaredMeasurementMode::ViewportRegion {
                horizontal,
                vertical,
            } => {
                let (x, width) = resolve_viewport_axis(horizontal, extent.width)?;
                let (y, height) = resolve_viewport_axis(vertical, extent.height)?;
                (x, y, width, height)
            }
            crate::declaration::UiDeclaredMeasurementMode::HugHeight => return None,
            crate::declaration::UiDeclaredMeasurementMode::FixedLogicalSize { .. } => {
                unreachable!("fixed logical bounds do not require host evidence")
            }
        };
        Some(UiAllocationAxisAlignedBounds {
            x,
            y,
            width,
            height,
            coordinate_space: result.coordinate_space(),
        })
    })
}

fn resolve_viewport_axis(
    placement: crate::capability::ComponentViewportAxisPlacement,
    viewport_extent: f32,
) -> Option<(f32, f32)> {
    if !viewport_extent.is_finite() || viewport_extent <= 0.0 {
        return None;
    }
    let span = placement.span(viewport_extent);
    let (origin, extent) = (span.start, span.extent);
    (origin >= 0.0 && extent > 0.0 && origin + extent <= viewport_extent)
        .then_some((origin, extent))
}

impl UiAllocationAxisAlignedBounds {
    pub fn x(self) -> f32 {
        self.x
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn width(self) -> f32 {
        self.width
    }
    pub fn height(self) -> f32 {
        self.height
    }
    pub fn coordinate_space(self) -> crate::evidence::UiMeasurementCoordinateSpace {
        self.coordinate_space
    }
}

impl UiAllocationEdgeReference {
    pub fn target(self) -> crate::graph::UiGraphNodeIdentity {
        self.target
    }
    pub fn axis(self) -> UiAllocationAxis {
        self.axis
    }
    pub fn delta(self) -> f32 {
        self.delta
    }
}

impl UiPortalAnchorObservationGeometryEvidence {
    pub fn identity(self) -> crate::runtime::UiPortalAnchorIdentity {
        self.identity
    }

    pub fn observed_bounds(self) -> UiAllocationAxisAlignedBounds {
        self.observed_bounds
    }
}

fn portal_anchor_observation(
    candidate: &super::UiAllocationCandidate,
) -> Option<UiPortalAnchorObservationGeometryEvidence> {
    if let Some(basis) = candidate.portal_allocation_input() {
        let observation = basis.observation();
        let rect = observation.rect();
        return Some(UiPortalAnchorObservationGeometryEvidence {
            identity: observation.identity(),
            observed_bounds: UiAllocationAxisAlignedBounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                coordinate_space: observation.identity().coordinate_space(),
            },
        });
    }
    let input = candidate
        .allocation_constraint_set()?
        .portal_anchor_planning_input()?;
    if input.posture()
        != crate::evidence::UiPortalAnchorPlanningInputPosture::AdmittedPlanningTimeOnly
        || !input.is_planning_time_only()
    {
        return None;
    }
    candidate.measurement_basis().evidence_inputs().iter().find_map(|evidence| {
        let result = evidence.as_host_measurement_result()?;
        let crate::evidence::UiMeasurementValue::PortalAnchorRect(rect) = result.value() else {
            return None;
        };
        let identity = crate::runtime::portal::anchored_allocation::UiPortalAnchorIdentity::from_measurement_result(result)?;
        Some(UiPortalAnchorObservationGeometryEvidence {
            identity,
            observed_bounds: UiAllocationAxisAlignedBounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                coordinate_space: result.coordinate_space(),
            },
        })
    })
}

#[cfg(test)]
mod viewport_region_tests {
    use super::resolve_viewport_axis;
    use crate::capability::ComponentViewportAxisPlacement as Placement;

    #[test]
    fn viewport_axes_preserve_fixed_rail_and_stretch_stage() {
        assert_eq!(
            resolve_viewport_axis(Placement::fixed_from_start(24, 216).unwrap(), 960.0),
            Some((24.0, 216.0)),
        );
        assert_eq!(
            resolve_viewport_axis(Placement::stretch_between(264, 24), 960.0),
            Some((264.0, 672.0)),
        );
        assert_eq!(
            resolve_viewport_axis(Placement::stretch_between(264, 24), 1_120.0),
            Some((264.0, 832.0)),
        );
        assert_eq!(
            resolve_viewport_axis(Placement::fixed_from_end(24, 24).unwrap(), 600.0),
            Some((552.0, 24.0)),
        );
        assert_eq!(
            resolve_viewport_axis(Placement::bounded_stretch(36, 36, 444).unwrap(), 600.0),
            Some((36.0, 444.0)),
        );
        assert_eq!(
            resolve_viewport_axis(Placement::bounded_stretch(36, 36, 444).unwrap(), 400.0),
            Some((36.0, 328.0)),
        );
    }

    #[test]
    fn viewport_axes_fail_closed_when_constraints_consume_the_viewport() {
        assert_eq!(
            resolve_viewport_axis(Placement::stretch_between(264, 24), 280.0),
            None,
        );
        assert_eq!(
            resolve_viewport_axis(Placement::fixed_from_end(24, 56).unwrap(), 64.0),
            None,
        );
    }
}

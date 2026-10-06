use std::collections::HashMap;

use worth_ui_host_contract::{
    UiHostRealizedGeometry, UiHostRealizedOrdering, UiHostRealizedRegion,
    UiHostRealizedRegionParticipation, UiHostSurfacePresentationDenial,
    UiMountedAllocationProjection, UiMountedCanonicalBox, UiMountedCoordinateSpace,
    UiMountedGeometryPosture, UiMountedHitTestProjection, UiMountedPaintCommand,
    UiMountedPaintCommandChange, UiMountedPaintCommandIdentity, UiMountedParticipationStatus,
    UiMountedPresentationDelta, UiMountedPresentationNodeChange, UiMountedPresentationNodeHitTest,
    UiMountedPresentationNodeState, UiMountedProjectionView,
};

#[derive(Clone)]
pub(super) struct UiNativeRetainedRegions {
    receipt_affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    paint: HashMap<UiMountedPaintCommandIdentity, UiHostRealizedRegion>,
    hit_test: Box<[UiHostRealizedRegion]>,
}

impl UiNativeRetainedRegions {
    pub(super) fn owns_node_receipt(
        &self,
        receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> bool {
        self.hit_test
            .iter()
            .chain(self.paint.values())
            .any(|region| self.current_receipt(region.mounted_receipt()) == receipt)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(super) fn paint_only(commands: &[UiMountedPaintCommand]) -> Self {
        Self {
            receipt_affinity: None,
            paint: commands.iter().map(command_region).collect(),
            hit_test: Box::new([]),
        }
    }

    pub(super) fn prepare(
        projection: &UiMountedProjectionView,
        commands: &[UiMountedPaintCommand],
    ) -> Result<Self, UiHostSurfacePresentationDenial> {
        let paint = commands
            .iter()
            .map(command_region)
            .collect::<HashMap<_, _>>();
        if paint.len() != commands.len() {
            return Err(UiHostSurfacePresentationDenial::MalformedProjection);
        }
        Ok(Self {
            receipt_affinity: projection.node_receipt_affinity(),
            paint,
            hit_test: hit_test_regions(projection)?.into_boxed_slice(),
        })
    }

    pub(super) fn apply_paint_changes(&mut self, changes: &[UiMountedPaintCommandChange]) {
        for change in changes {
            match change {
                UiMountedPaintCommandChange::Insert(command) => {
                    let (identity, region) = command_region(command);
                    self.paint.insert(identity, region);
                }
                UiMountedPaintCommandChange::Replace {
                    predecessor,
                    successor,
                } => {
                    self.paint.remove(predecessor);
                    let (identity, region) = command_region(successor);
                    self.paint.insert(identity, region);
                }
                UiMountedPaintCommandChange::Remove(identity) => {
                    self.paint.remove(identity);
                }
            }
        }
    }

    pub(super) fn apply_node_changes(
        &mut self,
        delta: &UiMountedPresentationDelta,
    ) -> Result<(), UiHostSurfacePresentationDenial> {
        let mut hit_test = self.hit_test.to_vec();
        for change in delta.nodes() {
            let instance = change.mounted_instance();
            hit_test.retain(|region| region.mounted_receipt().mounted_instance() != instance);
            let UiMountedPresentationNodeChange::Upsert(state) = change else {
                continue;
            };
            let UiMountedPresentationNodeHitTest::Region(row) = state.hit_test() else {
                continue;
            };
            hit_test.push(delta_hit_test_region(delta, *state, row)?);
        }
        hit_test.sort_unstable_by_key(|region| region.semantic_order());
        if hit_test
            .windows(2)
            .any(|pair| pair[0].semantic_order() == pair[1].semantic_order())
        {
            return Err(UiHostSurfacePresentationDenial::MalformedProjection);
        }
        self.hit_test = hit_test.into_boxed_slice();
        Ok(())
    }

    pub(super) fn rebind_receipt_affinity(
        &mut self,
        affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    ) {
        self.receipt_affinity = affinity;
    }

    pub(super) fn current_receipt(
        &self,
        receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.receipt_affinity
            .map_or(receipt, |affinity| affinity.rebind_node_receipt(receipt))
    }

    pub(super) fn replace_hit_tests(
        &mut self,
        projection: &UiMountedProjectionView,
    ) -> Result<(), UiHostSurfacePresentationDenial> {
        self.hit_test = hit_test_regions(projection)?.into_boxed_slice();
        Ok(())
    }

    pub(super) fn realized(
        &self,
        order: impl Iterator<Item = worth_ui_host_contract::UiMountedPaintOrderIdentity>,
    ) -> Option<Vec<UiHostRealizedRegion>> {
        let mut realized = order
            .map(|identity| self.paint.get(&identity.command()).copied())
            .collect::<Option<Vec<_>>>()?;
        realized.extend(self.hit_test.iter().copied());
        Some(
            realized
                .into_iter()
                .map(|region| {
                    self.receipt_affinity
                        .map_or(region, |affinity| affinity.rebind_realized_region(region))
                })
                .collect(),
        )
    }
}

fn command_region(
    command: &UiMountedPaintCommand,
) -> (UiMountedPaintCommandIdentity, UiHostRealizedRegion) {
    match command {
        UiMountedPaintCommand::PortalOverlay { identity, mechanic } => (
            *identity,
            UiHostRealizedRegion::observed_by_host(
                mechanic.owner_receipt(),
                UiHostRealizedGeometry::observed_by_host(mechanic.bounds(), mechanic.clip_bounds()),
                UiHostRealizedOrdering::observed_by_host(
                    mechanic.layer_semantic_order(),
                    UiHostRealizedRegionParticipation::Paint,
                ),
            ),
        ),
        UiMountedPaintCommand::SemanticText { identity, mechanic } => (
            *identity,
            UiHostRealizedRegion::observed_by_host(
                mechanic.node_receipt(),
                UiHostRealizedGeometry::observed_by_host(mechanic.bounds(), mechanic.clip_bounds()),
                UiHostRealizedOrdering::observed_by_host(
                    mechanic.layer_semantic_order(),
                    UiHostRealizedRegionParticipation::Paint,
                ),
            ),
        ),
    }
}

fn hit_test_regions(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHostRealizedRegion>, UiHostSurfacePresentationDenial> {
    projection
        .hit_tests()
        .rows()
        .iter()
        .copied()
        .map(|row| hit_test_region(projection, row))
        .collect()
}

fn hit_test_region(
    projection: &UiMountedProjectionView,
    row: worth_ui_host_contract::UiMountedHitTestMechanic,
) -> Result<UiHostRealizedRegion, UiHostSurfacePresentationDenial> {
    if row.frame() != projection.frame()
        || row.surface() != projection.surface()
        || row.binding() != projection.binding()
        || row.bounds().posture() != UiMountedGeometryPosture::Area
        || row.clip_bounds().posture() != UiMountedGeometryPosture::Area
        || row.bounds().coordinate_space() != UiMountedCoordinateSpace::Viewport
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    let node = projection
        .nodes()
        .iter()
        .find(|node| node.mounted_instance() == row.mounted_instance())
        .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?;
    let matching_reference = matches!(
        node.hit_test(),
        UiMountedHitTestProjection::Region(reference)
            if projection.hit_tests().resolve(reference) == Some(&row)
    );
    if node.node_receipt() != row.node_receipt()
        || node.participation().hit_test().status() != UiMountedParticipationStatus::Admitted
        || !matching_reference
        || !paints_hit_row(node.allocation(), row)
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    Ok(realized_hit_test_region(row))
}

fn delta_hit_test_region(
    delta: &UiMountedPresentationDelta,
    state: UiMountedPresentationNodeState,
    row: worth_ui_host_contract::UiMountedHitTestMechanic,
) -> Result<UiHostRealizedRegion, UiHostSurfacePresentationDenial> {
    if row.frame() != delta.affinity().successor()
        || row.surface() != delta.affinity().surface()
        || row.binding() != delta.affinity().binding()
        || row.mounted_instance() != state.mounted_instance()
        || row.node_receipt().mounted_instance() != state.mounted_instance()
        || row.bounds().posture() != UiMountedGeometryPosture::Area
        || row.clip_bounds().posture() != UiMountedGeometryPosture::Area
        || row.bounds().coordinate_space() != UiMountedCoordinateSpace::Viewport
        || state.participation().hit_test().status() != UiMountedParticipationStatus::Admitted
        || !paints_hit_row(state.allocation(), row)
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    Ok(realized_hit_test_region(row))
}

/// Whether `allocation` is the box a node whose hit row is `row` is painted in.
///
/// Hit testing reads the box on record, where the accepted scroll offset put
/// it. Inside a scrolled region at an offset off the device grid, paint reads
/// that box moved onto the grid, and the row carries the moved box it paints.
fn paints_hit_row(
    allocation: UiMountedAllocationProjection,
    row: worth_ui_host_contract::UiMountedHitTestMechanic,
) -> bool {
    paints(allocation, row.painted_bounds().unwrap_or(row.bounds()))
}

fn paints(allocation: UiMountedAllocationProjection, painted: UiMountedCanonicalBox) -> bool {
    matches!(allocation, UiMountedAllocationProjection::Known { bounds, .. } if bounds == painted)
}

fn realized_hit_test_region(
    row: worth_ui_host_contract::UiMountedHitTestMechanic,
) -> UiHostRealizedRegion {
    UiHostRealizedRegion::observed_by_host(
        row.node_receipt(),
        UiHostRealizedGeometry::observed_by_host(row.bounds(), row.clip_bounds()),
        UiHostRealizedOrdering::observed_by_host(
            row.order().rank(),
            UiHostRealizedRegionParticipation::HitTest,
        ),
    )
}

#[cfg(test)]
mod tests {
    use worth_ui_host_contract::{
        UiMountedAllocationBasis, UiMountedAllocationProjection, UiMountedCanonicalBox,
        UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedFrameIdentity,
        UiMountedHitTestCompletionInput, UiMountedHitTestMechanic, UiMountedHitTestOrder,
        UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer, UiMountedOmissionReason,
        UiMountedTransformProjection, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    };

    use super::{paints, paints_hit_row};

    fn area(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap()
    }

    fn painted_at(bounds: UiMountedCanonicalBox) -> UiMountedAllocationProjection {
        UiMountedAllocationProjection::Known {
            bounds,
            basis: UiMountedAllocationBasis::new(
                483,
                1,
                481,
                UiMountedTransformProjection::Identity,
            ),
        }
    }

    fn hit_row(
        bounds: UiMountedCanonicalBox,
        painted_bounds: Option<UiMountedCanonicalBox>,
    ) -> UiMountedHitTestMechanic {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
            frame,
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            mounted_instance: instance,
            node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(instance),
            bounds,
            clip_bounds: bounds,
            painted_bounds,
            order: UiMountedHitTestOrder::from_runtime_plan(0),
        })
        .unwrap()
    }

    #[test]
    fn a_hit_row_scrolled_off_the_device_grid_is_painted_in_the_box_it_carries() {
        // A scroll offset at 1.5x put this row at 269 logical points, half a
        // device pixel off the grid; it is painted at 268.67, on the grid.
        let recorded = area(1398.0, 269.0, 85.0, 25.0);
        let painted = area(1398.0, 268.666_66, 85.0, 25.0);
        let row = hit_row(recorded, Some(painted));
        assert!(paints_hit_row(painted_at(painted), row));
        assert!(
            !paints_hit_row(painted_at(recorded), row),
            "the box hit testing reads is not where the row is painted"
        );
    }

    #[test]
    fn a_hit_row_on_the_device_grid_is_painted_where_it_is_hit() {
        let bounds = area(1398.0, 268.666_66, 85.0, 25.0);
        assert!(paints_hit_row(painted_at(bounds), hit_row(bounds, None)));
    }

    #[test]
    fn an_allocation_anywhere_but_the_painted_box_is_denied() {
        let painted = area(1398.0, 268.666_66, 85.0, 25.0);
        for elsewhere in [
            area(1398.0, 269.0, 85.0, 25.0),
            area(1398.0, 270.666_66, 85.0, 25.0),
            area(1398.0, 268.666_66, 86.0, 25.0),
        ] {
            assert!(!paints(painted_at(elsewhere), painted));
        }
        assert!(!paints(
            UiMountedAllocationProjection::Omitted(
                UiMountedOmissionReason::AllocationBoundsUnknown
            ),
            painted
        ));
    }
}

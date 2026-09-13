use std::collections::HashMap;

use worth_ui_host_contract::{
    UiHostRealizedGeometry, UiHostRealizedOrdering, UiHostRealizedRegion,
    UiHostRealizedRegionParticipation, UiHostSurfacePresentationDenial,
    UiMountedAllocationProjection, UiMountedCoordinateSpace, UiMountedGeometryPosture,
    UiMountedHitTestProjection, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedParticipationStatus, UiMountedPresentationDelta,
    UiMountedPresentationNodeChange, UiMountedPresentationNodeHitTest,
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
    let matching_allocation = matches!(
        node.allocation(),
        UiMountedAllocationProjection::Known { bounds, .. } if bounds == row.bounds()
    );
    if node.node_receipt() != row.node_receipt()
        || node.participation().hit_test().status() != UiMountedParticipationStatus::Admitted
        || !matching_reference
        || !matching_allocation
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
    let matching_allocation = matches!(
        state.allocation(),
        UiMountedAllocationProjection::Known { bounds, .. } if bounds == row.bounds()
    );
    if row.frame() != delta.affinity().successor()
        || row.surface() != delta.affinity().surface()
        || row.binding() != delta.affinity().binding()
        || row.mounted_instance() != state.mounted_instance()
        || row.node_receipt().mounted_instance() != state.mounted_instance()
        || row.bounds().posture() != UiMountedGeometryPosture::Area
        || row.clip_bounds().posture() != UiMountedGeometryPosture::Area
        || row.bounds().coordinate_space() != UiMountedCoordinateSpace::Viewport
        || state.participation().hit_test().status() != UiMountedParticipationStatus::Admitted
        || !matching_allocation
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    Ok(realized_hit_test_region(row))
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

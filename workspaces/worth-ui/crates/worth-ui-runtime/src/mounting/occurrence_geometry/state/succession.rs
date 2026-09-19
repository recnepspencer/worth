use super::{UiMountedOccurrenceGeometryDenial as Denial, UiMountedOccurrenceGeometryState};

impl UiMountedOccurrenceGeometryState {
    pub(crate) fn prepare_generation_succession(
        &mut self,
        identity: &crate::mounting::UiMountedIdentityState,
        previous: &crate::runtime::session::WorthUiApplicationSessionState,
        candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        plan: &crate::runtime::WorthUiActiveExecutionPlan,
        bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Result<(), Denial> {
        let predecessor = previous.generation_identity();
        let successor = candidate.generation_identity();
        let declarations = bindings
            .bound_owners()
            .map(|(declaration, surface, _)| (surface, declaration))
            .collect::<std::collections::BTreeMap<_, _>>();
        let old_declarations = previous
            .authored_overlay_material()
            .overlay_declaration_bindings();
        let new_declarations = candidate
            .authored_overlay_material()
            .overlay_declaration_bindings();
        let mut region_successors = std::collections::BTreeMap::new();
        let mut checked = std::collections::BTreeMap::new();
        let retired = self
            .surfaces
            .values()
            .flat_map(|surface| surface.occurrences.iter())
            .filter_map(|(instance, row)| {
                identity
                    .projection_instance(*instance)
                    .is_none_or(|current| {
                        current.mount_incarnation() != row.incarnation
                            || current.graph_node_identity() != row.graph_node
                    })
                    .then_some(*instance)
            })
            .collect::<std::collections::BTreeSet<_>>();
        for instance in retired {
            self.retire_instance(instance);
        }
        for (surface, geometry) in &self.surfaces {
            if geometry.generation.as_ref() != Some(predecessor) {
                return Err(Denial::StaleLayoutBasis);
            }
            let binding = identity
                .projection_surface(*surface)
                .ok_or(Denial::MissingSurfaceBinding)?
                .0
                .binding_generation();
            if geometry.binding != binding {
                return Err(Denial::StaleLayoutBasis);
            }
            for (instance, row) in &geometry.occurrences {
                let current = identity
                    .projection_instance(*instance)
                    .ok_or(Denial::UnknownMountedInstance)?;
                if current.graph_node_identity() != row.graph_node
                    || current.mount_incarnation() != row.incarnation
                    || current.basis().semantic_surface_identity() != *surface
                {
                    return Err(Denial::StaleOccurrenceGeometry);
                }
                let matches = checked.entry((*surface, row.graph_node)).or_insert_with(|| {
                    let Some(layout) = previous.retains_mounted_layout(
                        plan,
                        crate::graph::UiGraphAuthority::new(candidate.graph_snapshot()),
                        row.graph_node,
                    ) else { return false; };
                    if layout == crate::runtime::session::UiMountedLayoutSuccession::NestedRegionRetirement
                        && row.surface_paint_posture != Default::default()
                    {
                        return false;
                    }
                    let Some(declaration) = declarations.get(surface).copied() else { return true; };
                    if previous.capabilities().mosaic_regions() != candidate.capabilities().mosaic_regions() {
                        return false;
                    }
                    let Some(old_surface) = new_declarations.surface_name(declaration)
                        .and_then(|name| old_declarations.surface_named(name)) else { return false; };
                    let old = previous.mounted_region_declarations(old_surface, row.graph_node).0;
                    let next = crate::runtime::session::WorthUiApplicationSessionState::region_declarations_from_plan(
                        plan, crate::graph::UiGraphAuthority::new(candidate.graph_snapshot()),
                        candidate.authored_overlay_material(), declaration, row.graph_node).0;
                    let old_by_identity = old.iter().map(|region| (region.executed_region(), region))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    if old_by_identity.len() != old.len() { return false; }
                    let by_identity = next.iter().map(|region| (region.executed_region(), region))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    if by_identity.len() != next.len() { return false; }
                    for region in &next {
                        let Some(old) = old_by_identity.get(region.executed_region()) else { return false; };
                        if !old.same_layout_meaning(region) { return false; }
                        if region_successors.insert((*surface, old.declaration()), region.declaration())
                            .is_some_and(|existing| existing != region.declaration()) { return false; }
                    }
                    true
                });
                if !*matches {
                    return Err(Denial::UnknownExecutedRegion);
                }
            }
        }
        // The completed coordinates and seam posture survive only after exact
        // binding, occurrence and executed-region dependency validation.
        for (surface, geometry) in &mut self.surfaces {
            let surviving_occurrences = geometry
                .occurrences
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            for row in geometry.occurrences.values_mut() {
                remap_clip_sources(*surface, row, &surviving_occurrences, &region_successors)?;
            }
            let mut regions = std::collections::BTreeMap::new();
            for (declaration, occurrences) in &geometry.regions {
                let Some(next) = region_successors.get(&(*surface, *declaration)) else {
                    continue;
                };
                if regions.insert(*next, occurrences.clone()).is_some() {
                    return Err(Denial::UnknownExecutedRegion);
                }
            }
            geometry.regions = regions;
            geometry.generation = Some(successor.clone());
        }
        Ok(())
    }
}

fn remap_clip_sources(
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    row: &mut super::UiMountedOccurrenceGeometryRow,
    surviving_occurrences: &std::collections::BTreeSet<
        worth_ui_host_contract::UiMountedInstanceIdentity,
    >,
    region_successors: &std::collections::BTreeMap<
        (
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        ),
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    >,
) -> Result<(), Denial> {
    if row.mosaic_clip_bindings.len() != row.mosaic_clips.len() {
        return Err(Denial::StaleOccurrenceGeometry);
    }
    let mut mosaic_bindings = Vec::with_capacity(row.mosaic_clip_bindings.len());
    let mut mosaic_clips = Vec::with_capacity(row.mosaic_clips.len());
    for (binding, clip) in row
        .mosaic_clip_bindings
        .iter()
        .copied()
        .zip(row.mosaic_clips.iter().copied())
    {
        let successor = match binding {
            crate::mounting::UiMountedMosaicClipBinding::Viewport => Some(binding),
            crate::mounting::UiMountedMosaicClipBinding::Region { owner, declaration } => {
                if !surviving_occurrences.contains(&owner) {
                    return Err(Denial::StaleOccurrenceGeometry);
                }
                region_successors
                    .get(&(surface, declaration))
                    .copied()
                    .map(
                        |declaration| crate::mounting::UiMountedMosaicClipBinding::Region {
                            owner,
                            declaration,
                        },
                    )
            }
        };
        if let Some(successor) = successor {
            mosaic_bindings.push(successor);
            mosaic_clips.push(clip);
        }
    }
    row.mosaic_clip_bindings = mosaic_bindings.into_boxed_slice();
    row.mosaic_clips = mosaic_clips.into_boxed_slice();

    match (&row.scroll_clip_bindings, &row.scroll_clips) {
        (Err(binding), Err(resolved)) if binding == resolved => return Ok(()),
        (Ok(bindings), Ok(clips)) if bindings.len() == clips.len() => {
            let mut successor_bindings = Vec::with_capacity(bindings.len());
            let mut successor_clips = Vec::with_capacity(clips.len());
            for (binding, clip) in bindings.iter().copied().zip(clips.iter().copied()) {
                let successor = match binding {
                    crate::mounting::UiMountedScrollClipBinding::Viewport => Some(binding),
                    crate::mounting::UiMountedScrollClipBinding::Occurrence(owner) => {
                        if !surviving_occurrences.contains(&owner) {
                            return Err(Denial::StaleOccurrenceGeometry);
                        }
                        Some(binding)
                    }
                    crate::mounting::UiMountedScrollClipBinding::Region { owner, declaration } => {
                        if !surviving_occurrences.contains(&owner) {
                            return Err(Denial::StaleOccurrenceGeometry);
                        }
                        region_successors
                            .get(&(surface, declaration))
                            .copied()
                            .map(|declaration| {
                                crate::mounting::UiMountedScrollClipBinding::Region {
                                    owner,
                                    declaration,
                                }
                            })
                    }
                };
                if let Some(successor) = successor {
                    successor_bindings.push(successor);
                    successor_clips.push(clip);
                }
            }
            row.scroll_clip_bindings = Ok(successor_bindings.into_boxed_slice());
            row.scroll_clips = Ok(successor_clips.into_boxed_slice());
        }
        _ => return Err(Denial::StaleOccurrenceGeometry),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mounting::{
        UiMountedMosaicClipBinding as Mosaic, UiMountedScrollClipBinding as Scroll,
    };
    use worth_ui_host_contract::{
        UiMountIncarnation, UiMountedAllocationBasis, UiMountedCanonicalBox,
        UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedInstanceIdentity,
        UiMountedTransformProjection, UiSemanticSurfaceIdentity,
    };

    #[test]
    fn nested_region_retirement_removes_only_the_retired_resolved_clip() {
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let old_outer = worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(1).unwrap();
        let old_inner = worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(2).unwrap();
        let new_outer = worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(3).unwrap();
        let outer = bounds(0.0, 0.0, 80.0, 60.0);
        let inner = bounds(8.0, 10.0, 24.0, 20.0);
        let mut row = row(
            owner,
            [
                Mosaic::Region {
                    owner,
                    declaration: old_outer,
                },
                Mosaic::Region {
                    owner,
                    declaration: old_inner,
                },
            ],
            [
                Scroll::Region {
                    owner,
                    declaration: old_outer,
                },
                Scroll::Region {
                    owner,
                    declaration: old_inner,
                },
            ],
            [outer, inner],
        );
        let surviving = [owner].into_iter().collect();
        let successors = [((surface, old_outer), new_outer)].into_iter().collect();

        remap_clip_sources(surface, &mut row, &surviving, &successors).unwrap();

        assert_eq!(
            row.mosaic_clip_bindings.as_ref(),
            &[Mosaic::Region {
                owner,
                declaration: new_outer,
            }]
        );
        assert_eq!(row.mosaic_clips.as_ref(), &[outer]);
        assert_eq!(
            row.scroll_clip_bindings.as_deref().unwrap(),
            &[Scroll::Region {
                owner,
                declaration: new_outer,
            }]
        );
        assert_eq!(row.scroll_clips.as_deref().unwrap(), &[outer]);
    }

    fn row(
        owner: UiMountedInstanceIdentity,
        mosaic: [Mosaic; 2],
        scroll: [Scroll; 2],
        clips: [UiMountedCanonicalBox; 2],
    ) -> super::super::UiMountedOccurrenceGeometryRow {
        super::super::UiMountedOccurrenceGeometryRow {
            parent: None,
            graph_node: crate::graph::UiGraphNodeIdentity::new(1),
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            bounds: bounds(0.0, 0.0, 100.0, 80.0),
            basis: UiMountedAllocationBasis::new(
                owner.diagnostic_value(),
                1,
                1,
                UiMountedTransformProjection::Identity,
            ),
            mosaic_clip_bindings: mosaic.into(),
            mosaic_clips: clips.into(),
            scroll_clip_bindings: Ok(scroll.into()),
            scroll_clips: Ok(clips.into()),
            surface_paint_posture: Default::default(),
        }
    }

    fn bounds(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap()
    }
}

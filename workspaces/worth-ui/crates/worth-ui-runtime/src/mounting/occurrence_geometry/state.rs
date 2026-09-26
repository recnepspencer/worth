use std::collections::{BTreeMap, BTreeSet};

use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedAllocationBasis, UiMountedCanonicalBox, UiMountedCoordinateSpace,
    UiMountedInstanceIdentity, UiMountedTransformProjection, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration,
};

use super::region::complete_regions;

use super::{UiMountedOccurrenceGeometryDenial, UiMountedSurfaceGeometryBatch};

#[path = "state/presented_grid.rs"]
mod presented_grid;
#[path = "state/projection.rs"]
mod projection;
#[path = "state/resolution.rs"]
mod resolution;
mod retirement;
mod sample_carry;
mod scroll;
#[path = "state/scroll_anchor.rs"]
mod scroll_anchor;
mod scroll_index;
mod scroll_pose;
pub(crate) use scroll_pose::UiPreparedMountedScrollPose;
#[path = "state/scroll_presentation.rs"]
mod scroll_presentation;
mod scroll_rest;
mod succession;

use resolution::{
    changed_instances, exact_region_bounds, resolve_surface_geometry, validate_placement,
};

#[derive(Clone, Debug, PartialEq)]
struct UiMountedOccurrenceGeometryRow {
    parent: Option<UiMountedInstanceIdentity>,
    graph_node: crate::graph::UiGraphNodeIdentity,
    incarnation: UiMountIncarnation,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    basis: UiMountedAllocationBasis,
    mosaic_clip_bindings: Box<[super::UiMountedMosaicClipBinding]>,
    mosaic_clips: Box<[UiMountedCanonicalBox]>,
    scroll_clip_bindings:
        Result<Box<[super::UiMountedScrollClipBinding]>, crate::graph::UiGraphNodeIdentity>,
    scroll_clips: Result<Box<[UiMountedCanonicalBox]>, crate::graph::UiGraphNodeIdentity>,
    surface_paint_posture: super::UiMountedSurfacePaintPosture,
}

#[derive(Clone, Debug, PartialEq)]
struct UiMountedSurfaceGeometry {
    binding: UiSurfaceBindingGeneration,
    /// The grid this surface's pixels land on, kept beside the boxes rather
    /// than asked for at each read, so presentation can place a scrolled box
    /// without reaching back into identity for the binding that placed it.
    device_scale: crate::runtime::scroll::UiScrollPresentationDeviceScale,
    layout_revision: super::UiMountedLayoutRevision,
    viewport: UiMountedCanonicalBox,
    occurrences: BTreeMap<UiMountedInstanceIdentity, UiMountedOccurrenceGeometryRow>,
    children: BTreeMap<UiMountedInstanceIdentity, Vec<UiMountedInstanceIdentity>>,
    scroll_index: std::sync::Arc<scroll_index::UiScrollGeometryIndex>,
    scroll_poses: BTreeMap<UiMountedInstanceIdentity, crate::runtime::scroll::UiScrollOffset>,
    generation: Option<
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    >,
    regions: BTreeMap<
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        Vec<(
            UiMountedInstanceIdentity,
            UiMountIncarnation,
            UiMountedCanonicalBox,
        )>,
    >,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiMountedOccurrenceGeometryState {
    surfaces: BTreeMap<UiSemanticSurfaceIdentity, UiMountedSurfaceGeometry>,
    reservations: scroll_index::UiScrollReservationMemo,
    /// The rows on each surface a settle moved without lowering them anew.
    sample_carried: BTreeMap<UiSemanticSurfaceIdentity, BTreeSet<UiMountedInstanceIdentity>>,
}

impl UiMountedOccurrenceGeometryState {
    pub(crate) fn replace_surface(
        &mut self,
        identity: &crate::mounting::UiMountedIdentityState,
        batch: UiMountedSurfaceGeometryBatch,
    ) -> Result<
        (Box<[UiMountedInstanceIdentity]>, usize, usize, usize, usize),
        UiMountedOccurrenceGeometryDenial,
    > {
        if !identity.validates_layout_basis(batch.basis()) {
            return Err(UiMountedOccurrenceGeometryDenial::StaleLayoutBasis);
        }
        let surface_binding = identity
            .projection_surface(batch.surface())
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?
            .0;
        let binding = surface_binding.binding_generation();
        let device_scale = crate::runtime::scroll::UiScrollPresentationDeviceScale::admit(
            surface_binding.profile().device_scale_milli(),
        )
        .map_err(|_| UiMountedOccurrenceGeometryDenial::UnusableDeviceScale)?;
        if batch.occurrences().is_empty() {
            return Err(UiMountedOccurrenceGeometryDenial::EmptyBatch);
        }
        if batch.viewport().coordinate_space() != UiMountedCoordinateSpace::HostSurface {
            return Err(UiMountedOccurrenceGeometryDenial::SurfaceViewportCoordinateSpaceMismatch);
        }
        if self
            .surfaces
            .get(&batch.surface())
            .is_some_and(|current| batch.layout_revision() <= current.layout_revision)
        {
            return Err(UiMountedOccurrenceGeometryDenial::StaleLayoutRevision);
        }
        let mounted = identity.projection_instances(&[batch.surface()]);
        let expected = mounted
            .iter()
            .map(|instance| instance.identity())
            .collect::<BTreeSet<_>>();
        let mut inputs = BTreeMap::new();
        for occurrence in batch.occurrences().iter().copied() {
            let view = identity
                .projection_instance(occurrence.instance())
                .ok_or(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)?;
            if view.basis().semantic_surface_identity() != batch.surface() {
                return Err(UiMountedOccurrenceGeometryDenial::ForeignSurface);
            }
            validate_placement(identity, batch.surface(), occurrence)?;
            if inputs
                .insert(
                    occurrence.instance(),
                    (occurrence, view.mount_incarnation()),
                )
                .is_some()
            {
                return Err(UiMountedOccurrenceGeometryDenial::DuplicateMountedInstance);
            }
        }
        if inputs.keys().copied().collect::<BTreeSet<_>>() != expected {
            return Err(UiMountedOccurrenceGeometryDenial::MissingMountedInstance);
        }
        let resolved = resolve_surface_geometry(&inputs)?;
        let (
            regions,
            exact_regions,
            surface_paint_postures,
            seam_index_rows,
            seam_adjacencies_visited,
        ) = complete_regions(&batch, &resolved, identity)?;
        let current = self.surfaces.get(&batch.surface());
        let mut rows = BTreeMap::new();
        let mut region_lookup_steps = 0;
        for (instance, (occurrence, incarnation)) in inputs {
            let parent = occurrence.parent();
            let bounds = resolved[&instance];
            let graph_node = identity
                .projection_instance(instance)
                .expect("the completed occurrence set was validated above")
                .graph_node_identity();
            let prior = current.and_then(|surface| surface.occurrences.get(&instance));
            let mosaic_clip_bindings = batch
                .mosaic_clips()
                .get(&instance)
                .cloned()
                .unwrap_or_default();
            let mosaic_clips = mosaic_clip_bindings
                .iter()
                .map(|binding| match binding {
                    super::UiMountedMosaicClipBinding::Viewport => Ok(batch.viewport()),
                    super::UiMountedMosaicClipBinding::Region { owner, declaration } => {
                        region_lookup_steps += 1;
                        exact_region_bounds(&exact_regions, *owner, *declaration)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice();
            let scroll_clip_bindings = batch
                .scroll_clips()
                .get(&instance)
                .cloned()
                .unwrap_or_else(|| Ok(Box::default()));
            let scroll_clips = match &scroll_clip_bindings {
                Err(node) => Err(*node),
                Ok(bindings) => Ok(bindings
                    .iter()
                    .map(|binding| match binding {
                        super::UiMountedScrollClipBinding::Viewport => Ok(batch.viewport()),
                        super::UiMountedScrollClipBinding::Occurrence(owner) => resolved
                            .get(owner)
                            .copied()
                            .ok_or(UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry),
                        super::UiMountedScrollClipBinding::Region { owner, declaration } => {
                            region_lookup_steps += 1;
                            exact_region_bounds(&exact_regions, *owner, *declaration)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Vec::into_boxed_slice)?),
            };
            let surface_paint_posture = surface_paint_postures
                .get(&instance)
                .cloned()
                .unwrap_or_default();
            let basis = prior
                .filter(|row| {
                    current.is_some_and(|surface| surface.binding == binding)
                        && row.parent == parent
                        && row.graph_node == graph_node
                        && row.incarnation == incarnation
                        && row.bounds == bounds
                        && row.mosaic_clip_bindings == mosaic_clip_bindings
                        && row.mosaic_clips == mosaic_clips
                        && row.scroll_clip_bindings == scroll_clip_bindings
                        && row.scroll_clips == scroll_clips
                        && row.surface_paint_posture == surface_paint_posture
                })
                .map(|row| row.basis)
                .unwrap_or_else(|| {
                    UiMountedAllocationBasis::new(
                        instance.diagnostic_value(),
                        batch.layout_revision().get(),
                        binding.diagnostic_value() ^ incarnation.diagnostic_value(),
                        UiMountedTransformProjection::Identity,
                    )
                });
            rows.insert(
                instance,
                UiMountedOccurrenceGeometryRow {
                    parent,
                    graph_node,
                    incarnation,
                    bounds,
                    basis,
                    mosaic_clip_bindings,
                    mosaic_clips,
                    scroll_clip_bindings,
                    scroll_clips,
                    surface_paint_posture,
                },
            );
        }
        let mut changed =
            changed_instances(self.surfaces.get(&batch.surface()), batch.viewport(), &rows);
        // The frame lowering this layout lowers every row a settle carried.
        changed.extend(
            self.sample_carried
                .remove(&batch.surface())
                .into_iter()
                .flatten()
                .filter(|instance| rows.contains_key(instance)),
        );
        let mut children = BTreeMap::<_, Vec<_>>::new();
        for (instance, row) in &rows {
            if let Some(parent) = row.parent {
                children.entry(parent).or_default().push(*instance);
            }
        }
        self.surfaces.insert(
            batch.surface(),
            UiMountedSurfaceGeometry {
                binding,
                device_scale,
                layout_revision: batch.layout_revision(),
                viewport: batch.viewport(),
                occurrences: rows,
                children,
                scroll_index: Default::default(),
                scroll_poses: BTreeMap::new(),
                generation: Some(batch.basis.generation),
                regions,
            }
            .index_scroll_geometry(),
        );
        // Snapshot reservations where the geometry changes, so presenting it
        // reads the snapshot instead of building one.
        let _ = self.scroll_geometry_reservations();
        Ok((
            changed.into_boxed_slice(),
            exact_regions.len(),
            region_lookup_steps,
            seam_index_rows,
            seam_adjacencies_visited,
        ))
    }

    pub(crate) fn surface_viewport(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<(
        UiSurfaceBindingGeneration,
        super::UiMountedLayoutRevision,
        UiMountedCanonicalBox,
    )> {
        self.surfaces.get(&surface).map(|geometry| {
            (
                geometry.binding,
                geometry.layout_revision,
                geometry.viewport,
            )
        })
    }

    pub(crate) fn region_extents(
        &self,
        surface: UiSemanticSurfaceIdentity,
        generation: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        region: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<
        Box<
            [(
                UiMountedInstanceIdentity,
                UiMountIncarnation,
                UiMountedCanonicalBox,
            )],
        >,
    > {
        let geometry = self.surfaces.get(&surface)?;
        if geometry.generation.as_ref() != Some(generation) {
            return None;
        }
        Some(
            geometry
                .regions
                .get(&region)?
                .iter()
                .copied()
                .filter(|row| geometry.occurrences.contains_key(&row.0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    pub(crate) fn rebind_surface(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
    ) -> Box<[UiMountedInstanceIdentity]> {
        let Some(geometry) = self.surfaces.get_mut(&surface) else {
            return Box::default();
        };
        geometry.binding = binding;
        let mut affected = Vec::with_capacity(geometry.occurrences.len());
        for (instance, row) in &mut geometry.occurrences {
            row.basis = UiMountedAllocationBasis::new(
                row.basis.receipt_identity(),
                row.basis.receipt_generation(),
                binding.diagnostic_value() ^ row.incarnation.diagnostic_value(),
                row.basis.transform(),
            );
            debug_assert_eq!(row.basis.receipt_identity(), instance.diagnostic_value());
            affected.push(*instance);
        }
        affected.into_boxed_slice()
    }

    pub(crate) fn validates_binding(
        &self,
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
    ) -> bool {
        self.surfaces
            .get(&surface)
            .is_none_or(|geometry| geometry.binding == binding)
    }

    pub(crate) fn next_layout_revision(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<super::UiMountedLayoutRevision, UiMountedOccurrenceGeometryDenial> {
        let next = self
            .surfaces
            .get(&surface)
            .map_or(Some(1), |geometry| {
                geometry.layout_revision.get().checked_add(1)
            })
            .ok_or(UiMountedOccurrenceGeometryDenial::StateRevisionExhausted)?;
        super::UiMountedLayoutRevision::new(next)
            .ok_or(UiMountedOccurrenceGeometryDenial::StateRevisionExhausted)
    }
}

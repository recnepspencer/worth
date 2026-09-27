use super::{UiMountedProjectionDenial, UiMountedProjectionFrame, UiMountedProjectionNodeRecord};
use crate::mounting::projection::appearance::{
    UiMountedAppearanceClip as Clip, UiMountedAppearanceClipDenial as Denial,
};
use crate::mounting::{UiLaidOut, UiMountedPlacement, UiPresented};

/// Derived geometry retained with its mounted node, separate from raw ancestry.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::mounting::projection) struct UiMountedAppearanceGeometry {
    /// Where the frame shows the occurrence. Portal content the frame
    /// presents nowhere keeps its laid-out allocation here, under a
    /// suppressed clip: its lowering keeps a coordinate space while it paints
    /// nothing.
    pub(super) allocation: UiPresented<worth_ui_host_contract::UiMountedAllocationProjection>,
    pub(super) clip: Clip,
    /// Where the frame presents the occurrence. Portal content whose
    /// placement could not be resolved stays in place, and its clip says so.
    pub(super) placement: UiMountedPlacement,
    pub(super) surface_paint_posture: crate::mounting::UiMountedSurfacePaintPosture,
}

impl UiMountedAppearanceGeometry {
    /// The geometry of an occurrence presented where it is laid out.
    #[expect(
        clippy::disallowed_methods,
        reason = "an occurrence's geometry starts where it is laid out; its placement then moves or suppresses it"
    )]
    pub(in crate::mounting::projection) fn in_place(
        allocation: UiLaidOut<worth_ui_host_contract::UiMountedAllocationProjection>,
        clip: Clip,
    ) -> Self {
        Self {
            allocation: allocation.in_place(),
            clip,
            placement: UiMountedPlacement::InPlace,
            surface_paint_posture: crate::mounting::UiMountedSurfacePaintPosture::ordinary(),
        }
    }

    pub(in crate::mounting::projection) fn with_surface_paint_posture(
        mut self,
        posture: crate::mounting::UiMountedSurfacePaintPosture,
    ) -> Self {
        self.surface_paint_posture = posture;
        self
    }

    pub(in crate::mounting::projection) fn surface_paint_posture(
        &self,
    ) -> crate::mounting::UiMountedSurfacePaintPosture {
        self.surface_paint_posture.clone()
    }

    pub(in crate::mounting::projection) const fn allocation(
        &self,
    ) -> UiPresented<worth_ui_host_contract::UiMountedAllocationProjection> {
        self.allocation
    }

    pub(in crate::mounting::projection) const fn placement(&self) -> UiMountedPlacement {
        self.placement
    }

    /// The Portal owner whose group presents this occurrence.
    pub(in crate::mounting::projection) fn portal_group(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.placement.portal().map(|portal| portal.owner())
    }
}

impl UiMountedProjectionNodeRecord {
    pub(in crate::mounting::projection) fn completed_appearance_geometry(
        &self,
    ) -> UiMountedAppearanceGeometry {
        self.appearance_geometry.clone()
    }
}

impl UiMountedProjectionFrame {
    #[cfg(test)]
    pub(crate) fn appearance_allocation_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedAllocationProjection> {
        self.semantic
            .node(instance)
            .map(UiMountedProjectionNodeRecord::completed_appearance_geometry)
            .map(|geometry| geometry.allocation.into_shown())
    }

    #[cfg(test)]
    pub(crate) fn appearance_clip_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::projection::UiMountedAppearanceClip> {
        self.semantic
            .node(instance)
            .map(UiMountedProjectionNodeRecord::completed_appearance_geometry)
            .map(|geometry| geometry.clip)
    }

    /// Portal owner changes already select predecessor and successor children.
    /// Ordinary appearance attempts and reconstruction only read this completion.
    pub(super) fn complete_appearance_geometry(&mut self) -> Result<(), UiMountedProjectionDenial> {
        for instance in self.changed_instances.clone().iter().copied() {
            let (node, probes) = self.semantic.nodes.get_with_probes(&instance);
            self.counters
                .touch_indexes(probes)
                .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
            let Some(node) = node else {
                continue;
            };
            let (geometry, probes, rows) = self.appearance_geometry_for(node)?;
            self.counters
                .touch_indexes(probes)
                .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
            self.counters
                .consider(rows)
                .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
            if node.appearance_geometry == geometry {
                continue;
            }
            let mut node = node.clone();
            node.appearance_geometry = geometry;
            // Only geometry changed; membership, order, and declaration stay intact.
            let work = self.semantic.nodes.insert_with_work(instance, node);
            self.counters
                .touch_indexes(work.key_probes())
                .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
            self.counters
                .replace_rows::<UiMountedProjectionNodeRecord>(work.node_copies())
                .map_err(|_| UiMountedProjectionDenial::CostCounterOverflow)?;
        }
        Ok(())
    }

    fn appearance_geometry_for(
        &self,
        node: &UiMountedProjectionNodeRecord,
    ) -> Result<(UiMountedAppearanceGeometry, usize, usize), UiMountedProjectionDenial> {
        let mut geometry =
            UiMountedAppearanceGeometry::in_place(node.occurrence_allocation, node.appearance_clip)
                .with_surface_paint_posture(node.appearance_geometry.surface_paint_posture());
        if node.portal_child_owner.is_none() {
            return Ok((geometry, 0, 0));
        }
        // No Portal fact can discharge another geometry owner's requirement.
        if matches!(geometry.clip, Clip::Unresolved(denial)
            if !matches!(denial, Denial::PortalBindingUnavailable(_)))
        {
            return Ok((geometry, 0, 0));
        }
        let (surface, surface_probes) = self
            .semantic
            .surface_for_with_probes(node.receipt.semantic_surface());
        let Some(surface) = surface else {
            geometry.clip = Clip::Unresolved(Denial::MountedGeometryUnavailable);
            return Ok((geometry, surface_probes, 0));
        };
        let (placement, probes, rows) = self.portal_child_placement_with_work(
            node.receipt.mounted_instance(),
            surface.surface,
            surface.binding,
        )?;
        match placement {
            UiMountedPlacement::InPlace => {
                geometry.clip = Clip::Unresolved(Denial::MountedGeometryUnavailable);
            }
            // Its Portal presents none of it: the clip suppresses its paint
            // where it is laid out.
            UiMountedPlacement::Hidden => {
                geometry.placement = placement;
                geometry.clip = Clip::Suppressed;
            }
            UiMountedPlacement::ThroughPortal(_) => {
                geometry.placement = placement;
                geometry.allocation = placement
                    .present(node.occurrence_allocation)
                    .ok()
                    .flatten()
                    .ok_or(UiMountedProjectionDenial::NonFiniteGeometry)?;
                geometry.clip =
                    match placement.present(UiLaidOut::from_layout(node.appearance_clip)) {
                        Ok(clip) => clip.map_or(Clip::Suppressed, UiPresented::into_shown),
                        Err(denial) => Clip::Unresolved(Denial::Geometry(denial)),
                    };
            }
        }
        Ok((
            geometry,
            surface_probes
                .checked_add(probes)
                .ok_or(UiMountedProjectionDenial::CostCounterOverflow)?,
            rows,
        ))
    }
}

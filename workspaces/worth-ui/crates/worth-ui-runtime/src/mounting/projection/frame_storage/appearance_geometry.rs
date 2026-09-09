use super::portal_child_view::UiMountedPortalChildPresentation;
use super::{UiMountedProjectionDenial, UiMountedProjectionFrame, UiMountedProjectionNodeRecord};
use crate::mounting::projection::appearance::{
    UiMountedAppearanceClip as Clip, UiMountedAppearanceClipDenial as Denial,
};

/// Derived geometry retained with its mounted node, separate from raw ancestry.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::mounting::projection) struct UiMountedAppearanceGeometry {
    pub(super) allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    pub(super) clip: Clip,
    pub(super) portal_group: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    pub(super) portal_presentation: Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    pub(super) surface_paint_posture: crate::mounting::UiMountedSurfacePaintPosture,
}

impl UiMountedAppearanceGeometry {
    pub(in crate::mounting::projection) fn from_occurrence(
        allocation: worth_ui_host_contract::UiMountedAllocationProjection,
        clip: Clip,
    ) -> Self {
        Self {
            allocation,
            clip,
            portal_group: None,
            portal_presentation: None,
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
    ) -> worth_ui_host_contract::UiMountedAllocationProjection {
        self.allocation
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
            .map(|geometry| geometry.allocation)
    }

    #[cfg(test)]
    pub(crate) fn appearance_clip_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedAppearanceClip> {
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
        let mut geometry = UiMountedAppearanceGeometry::from_occurrence(
            node.occurrence_allocation,
            node.appearance_clip,
        )
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
        let (presentation, probes, rows) = self.portal_child_presentation_with_work(
            node.receipt.mounted_instance(),
            surface.surface,
            surface.binding,
        )?;
        match presentation {
            UiMountedPortalChildPresentation::Ordinary => {
                geometry.clip = Clip::Unresolved(Denial::MountedGeometryUnavailable);
            }
            UiMountedPortalChildPresentation::Suppressed => geometry.clip = Clip::Suppressed,
            UiMountedPortalChildPresentation::Presented(portal) => {
                geometry.portal_group = Some(portal.owner());
                geometry.portal_presentation = Some(portal);
                geometry.allocation = super::super::appearance::portal_presented_allocation(
                    geometry.allocation,
                    portal,
                )
                .map_err(|_| UiMountedProjectionDenial::NonFiniteGeometry)?;
                geometry.clip =
                    super::super::appearance::portal_ancestor_clip(geometry.clip, portal)
                        .unwrap_or_else(|denial| Clip::Unresolved(Denial::Geometry(denial)));
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

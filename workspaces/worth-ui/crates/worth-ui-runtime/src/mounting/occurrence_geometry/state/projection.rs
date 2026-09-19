//! Exact mounted allocation and Mosaic clip lookup.
use worth_ui_host_contract::{UiMountedAllocationProjection, UiMountedCanonicalBox};

impl super::UiMountedOccurrenceGeometryState {
    pub(crate) fn projection(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> Result<
        Option<UiMountedAllocationProjection>,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        let surface = instance.basis().semantic_surface_identity();
        let Some(surface_geometry) = self.surfaces.get(&surface) else {
            return Ok(None);
        };
        let row = surface_geometry
            .occurrences
            .get(&instance.identity())
            .ok_or(crate::mounting::UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry)?;
        if row.incarnation != instance.mount_incarnation() {
            return Err(
                crate::mounting::UiMountedOccurrenceGeometryDenial::StaleOccurrenceGeometry,
            );
        }
        Ok(Some(UiMountedAllocationProjection::Known {
            bounds: row.bounds,
            basis: row.basis,
        }))
    }

    pub(crate) fn mosaic_clips(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> &[UiMountedCanonicalBox] {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or(&[], |row| row.mosaic_clips.as_ref())
    }

    pub(crate) fn scroll_clips(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> Result<&[UiMountedCanonicalBox], crate::graph::UiGraphNodeIdentity> {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or(Ok(&[]), |row| {
                row.scroll_clips.as_deref().map_err(|node| *node)
            })
    }

    pub(crate) fn surface_paint_posture(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> super::super::UiMountedSurfacePaintPosture {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or_else(Default::default, |row| row.surface_paint_posture.clone())
    }
}

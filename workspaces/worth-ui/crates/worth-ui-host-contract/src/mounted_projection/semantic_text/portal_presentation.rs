use super::{UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextMechanic};

impl UiMountedSemanticTextMechanic {
    #[doc(hidden)]
    pub fn presented_within_portal(
        &self,
        portal: crate::UiMountedPortalOverlayMechanic,
        source_anchor: crate::UiMountedCanonicalBox,
    ) -> Result<Option<Self>, UiMountedSemanticTextCompletionDenial> {
        let Some(geometry) = super::super::portal_child_geometry::project(
            self.bounds,
            self.clip_bounds,
            portal,
            source_anchor,
        )
        .map_err(|_| UiMountedSemanticTextCompletionDenial::NonAreaGeometry)?
        else {
            return Ok(None);
        };
        let bounds = geometry.bounds;
        let mut presented = self.clone();
        presented.bounds = bounds;
        presented.clip_bounds = geometry.clip;
        presented.intrinsic_clip_bounds = super::super::portal_child_geometry::translate(
            self.intrinsic_clip_bounds,
            portal,
            source_anchor,
        )
        .map_err(|_| UiMountedSemanticTextCompletionDenial::NonAreaGeometry)?;
        presented.origin_x = self.origin_x + portal.paint_bounds().x() - source_anchor.x();
        presented.origin_y = self.origin_y + portal.paint_bounds().y() - source_anchor.y();
        presented.layer_semantic_order = portal
            .layer_semantic_order()
            .saturating_add(1 + self.layer_semantic_order.min(1_024));
        let max_x = bounds.x() + bounds.width();
        let max_y = bounds.y() + bounds.height();
        if presented.origin_x < bounds.x()
            || presented.origin_x > max_x
            || presented.origin_y < bounds.y()
            || presented.origin_y > max_y
        {
            return Err(UiMountedSemanticTextCompletionDenial::InvalidTextOrigin);
        }
        presented.semantic_digest = super::validation::semantic_digest_mechanic(&presented);
        Ok(Some(presented))
    }
}

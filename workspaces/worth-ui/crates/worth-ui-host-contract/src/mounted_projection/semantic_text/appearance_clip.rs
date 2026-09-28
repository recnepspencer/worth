use super::{UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextMechanic};

impl UiMountedSemanticTextMechanic {
    /// Narrows visible coverage without retiring the qualified row. Empty
    /// coverage remains retained so a Scroll sample can reveal it later.
    /// This changes neither qualification nor span paint ownership.
    #[doc(hidden)]
    pub fn clipped_to_appearance_ancestor(
        mut self,
        ancestor: crate::UiAppearanceClip,
    ) -> Result<Option<Self>, UiMountedSemanticTextCompletionDenial> {
        let points = crate::appearance_points_f32;
        let x = self.clip_bounds.x().max(points(i64::from(ancestor.x())));
        let y = self.clip_bounds.y().max(points(i64::from(ancestor.y())));
        let right = (self.clip_bounds.x() + self.clip_bounds.width()).min(points(
            i64::from(ancestor.x()) + i64::from(ancestor.width()),
        ));
        let bottom = (self.clip_bounds.y() + self.clip_bounds.height()).min(points(
            i64::from(ancestor.y()) + i64::from(ancestor.height()),
        ));
        self.clip_bounds =
            crate::UiMountedCanonicalBox::canonicalize(crate::UiMountedCanonicalBoxInput {
                x,
                y,
                width: (right - x).max(0.0),
                height: (bottom - y).max(0.0),
                coordinate_space: self.clip_bounds.coordinate_space(),
            })
            .map_err(|_| UiMountedSemanticTextCompletionDenial::NonAreaGeometry)?;
        self.semantic_digest = super::validation::semantic_digest_mechanic(&self);
        Ok(Some(self))
    }
}

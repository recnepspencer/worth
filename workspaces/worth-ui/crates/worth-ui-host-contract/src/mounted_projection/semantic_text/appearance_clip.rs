use super::{UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextMechanic};

impl UiMountedSemanticTextMechanic {
    /// Narrows retained text coverage to the runtime-completed ancestor clip.
    /// This changes neither qualification nor span paint ownership.
    #[doc(hidden)]
    pub fn clipped_to_appearance_ancestor(
        mut self,
        ancestor: crate::UiAppearanceClip,
    ) -> Result<Option<Self>, UiMountedSemanticTextCompletionDenial> {
        let x = self.clip_bounds.x().max(ancestor.x() as f32 / 1_000.0);
        let y = self.clip_bounds.y().max(ancestor.y() as f32 / 1_000.0);
        let right = (self.clip_bounds.x() + self.clip_bounds.width())
            .min((f64::from(ancestor.x()) + f64::from(ancestor.width())) as f32 / 1_000.0);
        let bottom = (self.clip_bounds.y() + self.clip_bounds.height())
            .min((f64::from(ancestor.y()) + f64::from(ancestor.height())) as f32 / 1_000.0);
        if right <= x || bottom <= y {
            return Ok(None);
        }
        self.clip_bounds =
            crate::UiMountedCanonicalBox::canonicalize(crate::UiMountedCanonicalBoxInput {
                x,
                y,
                width: right - x,
                height: bottom - y,
                coordinate_space: self.clip_bounds.coordinate_space(),
            })
            .map_err(|_| UiMountedSemanticTextCompletionDenial::NonAreaGeometry)?;
        self.semantic_digest = super::validation::semantic_digest_mechanic(&self);
        Ok(Some(self))
    }
}

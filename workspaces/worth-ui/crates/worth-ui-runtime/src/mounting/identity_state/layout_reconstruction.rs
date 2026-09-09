use super::UiMountedIdentityState;

impl UiMountedIdentityState {
    pub(crate) const fn peak_qualified_layouts(&self) -> usize {
        self.peak_qualified_layouts
    }

    pub(crate) fn require_current_layout_reconstruction(
        &mut self,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        let current = self.current_projection.as_ref().ok_or(
            crate::mounting::UiMountedProjectionDenial::MissingSemanticTextReconstructionSource,
        )?;
        let mut successor = current.projection().clone();
        let lost = successor.require_qualified_layout_reconstruction()?;
        self.current_projection = Some(std::rc::Rc::new(
            crate::mounting::UiMountedProjectionFrameOwner::new(
                std::rc::Rc::new(successor),
                current.appearance().clone(),
                current.theme_revision(),
                current.pointer.clone(),
            ),
        ));
        Ok(lost)
    }

    pub(crate) fn reconstruct_current_layouts(
        &mut self,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        if !self.current_projection.as_ref().is_some_and(|owner| {
            owner
                .projection()
                .qualified_layout_reconstruction_required()
        }) {
            return Ok(0);
        }
        let current = self.current_projection.as_ref().ok_or(
            crate::mounting::UiMountedProjectionDenial::MissingSemanticTextReconstructionSource,
        )?;
        let mut successor = current.projection().clone();
        let reconstructed = successor.reconstruct_qualified_layouts()?;
        self.current_projection = Some(std::rc::Rc::new(
            crate::mounting::UiMountedProjectionFrameOwner::new(
                std::rc::Rc::new(successor),
                current.appearance().clone(),
                current.theme_revision(),
                current.pointer.clone(),
            ),
        ));
        Ok(reconstructed)
    }
}

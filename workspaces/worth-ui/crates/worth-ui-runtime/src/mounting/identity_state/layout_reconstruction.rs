use super::UiMountedIdentityState;

impl UiMountedIdentityState {
    pub(crate) const fn peak_qualified_layouts(&self) -> usize {
        self.peak_qualified_layouts
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn require_current_layout_reconstruction(
        &mut self,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        let current = self.frame.projection_mut().ok_or(
            crate::mounting::UiMountedProjectionDenial::MissingSemanticTextReconstructionSource,
        )?;
        let mut successor = current.projection().clone();
        let lost = successor.require_qualified_layout_reconstruction()?;
        *current = std::rc::Rc::new(crate::mounting::UiMountedProjectionFrameOwner::new(
            std::rc::Rc::new(successor),
            current.appearance().clone(),
            current.pointer.clone(),
        ));
        Ok(lost)
    }

    pub(crate) fn reconstruct_current_layouts(
        &mut self,
    ) -> Result<usize, crate::mounting::UiMountedProjectionDenial> {
        let Some(current) = self.frame.projection_mut().filter(|owner| {
            owner
                .projection()
                .qualified_layout_reconstruction_required()
        }) else {
            return Ok(0);
        };
        let mut successor = current.projection().clone();
        let reconstructed = successor.reconstruct_qualified_layouts()?;
        *current = std::rc::Rc::new(crate::mounting::UiMountedProjectionFrameOwner::new(
            std::rc::Rc::new(successor),
            current.appearance().clone(),
            current.pointer.clone(),
        ));
        Ok(reconstructed)
    }
}

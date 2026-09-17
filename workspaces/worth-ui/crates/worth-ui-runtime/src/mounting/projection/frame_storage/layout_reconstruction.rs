use super::UiMountedProjectionFrame;

impl UiMountedProjectionFrame {
    pub(crate) fn qualified_layout(
        &self,
        identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&worth_ui_text::UiQualifiedTextLayout> {
        self.mechanics
            .qualified_layout(identity)
            .map(std::sync::Arc::as_ref)
    }

    pub(crate) fn qualified_layout_count(&self) -> usize {
        self.mechanics.qualified_layout_count()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn require_qualified_layout_reconstruction(
        &mut self,
    ) -> Result<usize, super::UiMountedProjectionDenial> {
        self.mechanics.require_qualified_layout_reconstruction()
    }

    pub(crate) fn reconstruct_qualified_layouts(
        &mut self,
    ) -> Result<usize, super::UiMountedProjectionDenial> {
        self.mechanics.reconstruct_qualified_layouts()
    }

    pub(crate) fn qualified_layout_reconstruction_required(&self) -> bool {
        self.mechanics.qualified_layout_reconstruction_required()
    }
}

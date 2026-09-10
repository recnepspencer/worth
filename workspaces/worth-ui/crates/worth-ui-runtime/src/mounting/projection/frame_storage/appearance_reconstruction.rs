use super::{UiMountedProjectionFrame, UiMountedProjectionFrameOwner};

impl UiMountedProjectionFrame {
    pub(crate) fn appearance_node_inputs_for_reconstruction(
        &self,
    ) -> Result<
        Vec<super::UiMountedAppearanceNodeInputContext>,
        super::super::UiMountedProjectionDenial,
    > {
        self.appearance_node_inputs_for_reconstruction_on(None)
    }

    fn appearance_node_inputs_for_reconstruction_on(
        &self,
        surfaces: Option<&[worth_ui_host_contract::UiMountedSurfaceBindingRequirement]>,
    ) -> Result<
        Vec<super::UiMountedAppearanceNodeInputContext>,
        super::super::UiMountedProjectionDenial,
    > {
        self.semantic
            .nodes_in_mounted_order()
            .filter(|node| {
                surfaces.is_none_or(|bindings| {
                    bindings.iter().any(|binding| {
                        binding.semantic_surface() == node.receipt().semantic_surface()
                    })
                })
            })
            .map(|node| self.appearance_node_input(node.receipt().mounted_instance()))
            .collect()
    }
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn prepare_appearance_reconstruction(
        &mut self,
    ) -> Result<(), super::super::UiMountedProjectionDenial> {
        let nodes = self
            .projection
            .appearance_node_inputs_for_reconstruction()?;
        self.pointer.require_reconstruction();
        self.appearance.prepare_reconstruction(nodes);
        Ok(())
    }

    pub(crate) fn prepare_appearance_reconstruction_for(
        &mut self,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
    ) -> Result<(), super::super::UiMountedProjectionDenial> {
        let nodes = self
            .projection
            .appearance_node_inputs_for_reconstruction_on(Some(bindings))?;
        self.pointer.require_reconstruction();
        self.appearance.prepare_surface_reconstruction(nodes);
        Ok(())
    }
}

use super::UiAssembledMountedFrame;

impl UiAssembledMountedFrame {
    pub(crate) fn visual_region_basis(&self) -> crate::mounting::UiMountedVisualRegionBasis {
        let bindings = self
            .manifest
            .surfaces()
            .iter()
            .map(|requirement| (requirement.semantic_surface(), requirement.binding()))
            .collect::<Vec<_>>();
        self.candidate
            .frame()
            .visual_region_basis()
            .with_appearance_paint(
                self.candidate
                    .owner
                    .appearance()
                    .retained_visual_mechanics(),
                &bindings,
            )
    }
}

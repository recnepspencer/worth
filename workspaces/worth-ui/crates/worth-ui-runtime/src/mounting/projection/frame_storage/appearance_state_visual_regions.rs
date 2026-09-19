use super::UiMountedAppearanceFrameState;

impl UiMountedAppearanceFrameState {
    pub(in crate::mounting) fn retained_visual_mechanics(
        &self,
    ) -> Vec<crate::mounting::UiMountedRetainedAppearanceVisualMechanic> {
        self.members
            .retained_sidecars()
            .chain(self.overlay_sidecars.values())
            .filter_map(super::super::super::appearance::UiMountedAppearanceSidecar::current_facts)
            .flat_map(|facts| {
                facts.records().iter().map(|fact| {
                    crate::mounting::UiMountedRetainedAppearanceVisualMechanic::new(
                        fact.semantic_surface(),
                        fact.mechanic().clone(),
                        fact.semantic_digest(),
                    )
                })
            })
            .collect()
    }
}

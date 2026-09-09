use worth_ui_host_contract::UiSurfaceBindingGeneration;

use super::UiMountedProjectionFrame;
use crate::mounting::projection::UiMountedProjectionDenial;

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn rebind_retained_mechanics(
        &mut self,
        replacements: &[(
            UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<(), UiMountedProjectionDenial> {
        self.rebind_semantic_surfaces(replacements)?;
        self.hit_index_work
            .merge(self.mechanics.rebind(replacements)?);
        self.reconstruct_presented_hits()?;
        Ok(())
    }

    pub(crate) fn rebound(
        &self,
        successor: worth_ui_host_contract::UiMountedFrameIdentity,
        replacements: &[(
            UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<Self, UiMountedProjectionDenial> {
        let mut rebound = self.clone();
        rebound.frame = successor;
        rebound.rebind_semantic_surfaces(replacements)?;
        rebound.hit_index_work = rebound.mechanics.rebind(replacements)?;
        rebound.reconstruct_presented_hits()?;
        Ok(rebound)
    }

    fn rebind_semantic_surfaces(
        &mut self,
        replacements: &[(
            UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<(), UiMountedProjectionDenial> {
        for (affected, replacement) in replacements {
            let replacement_binding = replacement.binding_generation();
            let mut surface = self
                .semantic
                .surfaces
                .get(affected)
                .copied()
                .or_else(|| self.semantic.surfaces.get(&replacement_binding).copied())
                .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
            if surface.surface != replacement.semantic_surface_identity() {
                return Err(UiMountedProjectionDenial::MissingSurfaceBinding);
            }
            if surface.binding != replacement_binding {
                surface.binding = replacement_binding;
                self.semantic.replace_surface(surface);
            }
            self.semantic.rebind_surface_allocations(
                replacement.semantic_surface_identity(),
                replacement_binding,
            );
        }
        Ok(())
    }
}

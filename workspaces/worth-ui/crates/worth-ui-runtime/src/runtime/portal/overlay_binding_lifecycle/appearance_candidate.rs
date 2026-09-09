use super::{
    UiPortalOverlayBindingLifecycle, UiPortalOverlayBindingLifecycleDenial,
    UiPortalOverlayBindingOwner, UiPortalOverlayBindingStage, UiPreparedPortalServiceTransition,
};
use worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

impl UiPortalOverlayBindingLifecycle {
    pub(crate) fn candidate_bound_owners(
        &self,
        transition: &UiPreparedPortalServiceTransition,
        stage: Option<&UiPortalOverlayBindingStage>,
        retain_exit: bool,
    ) -> Result<
        Vec<(
            UiSemanticSurfaceDeclarationIdentity,
            UiSemanticSurfaceIdentity,
            UiPortalOverlayBindingOwner,
        )>,
        UiPortalOverlayBindingLifecycleDenial,
    > {
        let mut owners = Vec::with_capacity(self.surface_bindings.len());
        for (declaration, runtime) in &self.surface_bindings {
            let mut owner = self
                .owners
                .get(runtime)
                .cloned()
                .ok_or(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound)?;
            if let Some(stage) = stage.filter(|stage| stage.runtime_surface == *runtime) {
                match owner.binding_for_portal(stage.portal) {
                    Some(current) if current != stage.declaration => {
                        return Err(
                            UiPortalOverlayBindingLifecycleDenial::PortalDeclarationConflict,
                        );
                    }
                    Some(_) => {}
                    None => owner
                        .bind(stage.declaration, stage.portal)
                        .map_err(UiPortalOverlayBindingLifecycleDenial::Owner)?,
                }
            }
            if transition.closes_portal() && !retain_exit {
                owner.remove(transition.portal());
                for descendant in transition.closed_descendants() {
                    owner.remove(*descendant);
                }
            }
            owners.push((*declaration, *runtime, owner));
        }
        Ok(owners)
    }
}

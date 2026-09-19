use worth_ui_host_contract::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

/// Exact live inputs under which a layout batch is prepared. Only the mounted
/// owner can issue this basis; completion rejects any intervening identity or
/// binding mutation, including owner reincarnation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedLayoutBasis {
    pub(in crate::mounting) generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(in crate::mounting) surface: UiSemanticSurfaceIdentity,
    pub(in crate::mounting) binding: UiSurfaceBindingGeneration,
    pub(in crate::mounting) world: crate::mounting::UiMountedGraphWorldIdentity,
    pub(in crate::mounting) semantic_revision: u64,
}

impl UiMountedLayoutBasis {
    pub fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) fn generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        &self.generation
    }
}

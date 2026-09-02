use super::planner::UiOverlayPortalBinding;
use super::snapshot::UiOverlayApplicationGeneration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayPortalBindingExport {
    generation: UiOverlayApplicationGeneration,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    portal_revision: u64,
    rows: Box<[UiOverlayPortalBinding]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiOverlayBindingExportDenial {
    DuplicatePortal,
}

impl UiOverlayPortalBindingExport {
    pub(super) fn from_prepared(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        portal_revision: u64,
        bindings: impl IntoIterator<Item = UiOverlayPortalBinding>,
    ) -> Result<Self, UiOverlayBindingExportDenial> {
        let generation = UiOverlayApplicationGeneration::from_prepared(generation);
        let mut rows = bindings.into_iter().collect::<Vec<_>>();
        rows.sort_unstable_by_key(|binding| (binding.portal(), binding.declaration()));
        if rows
            .windows(2)
            .any(|window| window[0].portal() == window[1].portal())
        {
            return Err(UiOverlayBindingExportDenial::DuplicatePortal);
        }
        Ok(Self {
            generation,
            runtime_surface,
            portal_revision,
            rows: rows.into_boxed_slice(),
        })
    }

    pub(super) fn generation(&self) -> &UiOverlayApplicationGeneration {
        &self.generation
    }

    pub(super) const fn runtime_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.runtime_surface
    }

    pub(super) const fn portal_revision(&self) -> u64 {
        self.portal_revision
    }

    pub(super) fn rows(&self) -> &[UiOverlayPortalBinding] {
        &self.rows
    }
}

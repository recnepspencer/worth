use std::collections::BTreeSet;

use super::planner::UiOverlayPortalBinding;
use super::snapshot::UiOverlayApplicationGeneration;
use crate::runtime::portal::UiPortalOverlayBindingOwnerExport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayPortalBindingExport {
    generation: UiOverlayApplicationGeneration,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    portal_revision: u64,
    rows: Box<[UiOverlayPortalBinding]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiOverlayBindingExportDenial {
    GenerationMismatch,
    DuplicatePortal,
}

impl UiOverlayPortalBindingExport {
    pub(super) fn from_owner(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        source: &UiPortalOverlayBindingOwnerExport,
    ) -> Result<Self, UiOverlayBindingExportDenial> {
        if source.generation() != &generation {
            return Err(UiOverlayBindingExportDenial::GenerationMismatch);
        }
        let rows = source
            .rows()
            .iter()
            .map(|row| UiOverlayPortalBinding::new(row.declaration(), row.portal()))
            .collect::<Vec<_>>();
        reject_duplicate_runtime_portals(&rows)?;
        Ok(Self {
            generation: UiOverlayApplicationGeneration::from_prepared(generation),
            runtime_surface: source.runtime_surface(),
            portal_revision: source.portal_revision(),
            rows: rows.into_boxed_slice(),
        })
    }

    #[cfg(test)]
    pub(super) fn from_prepared(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        portal_revision: u64,
        bindings: impl IntoIterator<Item = UiOverlayPortalBinding>,
    ) -> Result<Self, UiOverlayBindingExportDenial> {
        let generation = UiOverlayApplicationGeneration::from_prepared(generation);
        let rows = bindings.into_iter().collect::<Vec<_>>();
        reject_duplicate_runtime_portals(&rows)?;
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

fn reject_duplicate_runtime_portals(
    rows: &[UiOverlayPortalBinding],
) -> Result<(), UiOverlayBindingExportDenial> {
    let mut portals = BTreeSet::new();
    if rows.iter().any(|row| !portals.insert(row.portal())) {
        return Err(UiOverlayBindingExportDenial::DuplicatePortal);
    }
    Ok(())
}

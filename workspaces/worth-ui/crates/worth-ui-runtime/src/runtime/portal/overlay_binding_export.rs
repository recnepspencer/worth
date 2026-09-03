use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use worth_ui_dsl::UiPortalDeclarationId;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

use super::{UiPortalIdentity, UiPortalStackSnapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalOverlayBindingRow {
    declaration: UiPortalDeclarationId,
    portal: UiPortalIdentity,
}

impl UiPortalOverlayBindingRow {
    pub(crate) const fn new(declaration: UiPortalDeclarationId, portal: UiPortalIdentity) -> Self {
        Self {
            declaration,
            portal,
        }
    }

    pub(crate) const fn declaration(self) -> UiPortalDeclarationId {
        self.declaration
    }

    pub(crate) const fn portal(self) -> UiPortalIdentity {
        self.portal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalOverlayBindingOwnerExport {
    generation: WorthUiPreparedApplicationGenerationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    portal_revision: u64,
    rows: Box<[UiPortalOverlayBindingRow]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalOverlayBindingDenial {
    #[allow(
        dead_code,
        reason = "The binding mutation lane exposes this denial to successor owners and tests."
    )]
    DuplicateBinding,
    #[allow(
        dead_code,
        reason = "The binding mutation lane exposes this denial to successor owners and tests."
    )]
    PortalDeclarationConflict,
    MissingPortal,
    ForeignSurface,
}

pub(crate) struct UiPortalOverlayBindingOwner {
    generation: WorthUiPreparedApplicationGenerationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    portal_declarations: BTreeMap<UiPortalIdentity, UiPortalDeclarationId>,
}

impl UiPortalOverlayBindingOwner {
    #[allow(
        dead_code,
        reason = "Binding-owner construction is consumed by owner tests and successor-facing setup."
    )]
    pub(crate) fn new(
        generation: WorthUiPreparedApplicationGenerationIdentity,
        runtime_surface: UiSemanticSurfaceIdentity,
    ) -> Self {
        Self {
            generation,
            runtime_surface,
            portal_declarations: BTreeMap::new(),
        }
    }

    #[allow(
        dead_code,
        reason = "Binding mutation is consumed by owner tests and successor-facing setup."
    )]
    pub(crate) fn bind(
        &mut self,
        declaration: UiPortalDeclarationId,
        portal: UiPortalIdentity,
    ) -> Result<(), UiPortalOverlayBindingDenial> {
        Self::insert_binding(&mut self.portal_declarations, portal, declaration)
    }

    #[allow(
        dead_code,
        reason = "Binding replacement is consumed by owner tests and successor-facing setup."
    )]
    pub(crate) fn replace(
        &mut self,
        bindings: impl IntoIterator<Item = UiPortalOverlayBindingRow>,
    ) -> Result<(), UiPortalOverlayBindingDenial> {
        let mut candidate = BTreeMap::new();
        for binding in bindings {
            Self::insert_binding(&mut candidate, binding.portal(), binding.declaration())?;
        }
        self.portal_declarations = candidate;
        Ok(())
    }

    /// Seal the binding export with one ordered pass over the current Portal
    /// snapshot, then validate bindings in bound portal-identity order. For
    /// each binding, MissingPortal is checked before ForeignSurface.
    pub(crate) fn export(
        &self,
        snapshot: &UiPortalStackSnapshot,
    ) -> Result<UiPortalOverlayBindingOwnerExport, UiPortalOverlayBindingDenial> {
        let mut seen = BTreeSet::new();
        let mut foreign = BTreeSet::new();
        let mut rows = Vec::new();
        for row in snapshot.rows() {
            let portal = row.portal();
            let Some(declaration) = self.portal_declarations.get(&portal) else {
                continue;
            };
            seen.insert(portal);
            if row.surface() != self.runtime_surface {
                foreign.insert(portal);
            } else {
                rows.push(UiPortalOverlayBindingRow::new(*declaration, portal));
            }
        }
        for portal in self.portal_declarations.keys() {
            if !seen.contains(portal) {
                return Err(UiPortalOverlayBindingDenial::MissingPortal);
            }
            if foreign.contains(portal) {
                return Err(UiPortalOverlayBindingDenial::ForeignSurface);
            }
        }
        Ok(UiPortalOverlayBindingOwnerExport {
            generation: self.generation.clone(),
            runtime_surface: self.runtime_surface,
            portal_revision: snapshot.owner_revision(),
            rows: rows.into_boxed_slice(),
        })
    }

    #[allow(
        dead_code,
        reason = "Binding insertion supports the dormant successor-facing mutation lane."
    )]
    fn insert_binding(
        table: &mut BTreeMap<UiPortalIdentity, UiPortalDeclarationId>,
        portal: UiPortalIdentity,
        declaration: UiPortalDeclarationId,
    ) -> Result<(), UiPortalOverlayBindingDenial> {
        match table.entry(portal) {
            Entry::Vacant(entry) => {
                entry.insert(declaration);
                Ok(())
            }
            Entry::Occupied(entry) if *entry.get() == declaration => {
                Err(UiPortalOverlayBindingDenial::DuplicateBinding)
            }
            Entry::Occupied(_) => Err(UiPortalOverlayBindingDenial::PortalDeclarationConflict),
        }
    }
}

impl UiPortalOverlayBindingOwnerExport {
    pub(crate) fn generation(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn runtime_surface(&self) -> UiSemanticSurfaceIdentity {
        self.runtime_surface
    }

    pub(crate) const fn portal_revision(&self) -> u64 {
        self.portal_revision
    }

    pub(crate) fn rows(&self) -> &[UiPortalOverlayBindingRow] {
        &self.rows
    }
}

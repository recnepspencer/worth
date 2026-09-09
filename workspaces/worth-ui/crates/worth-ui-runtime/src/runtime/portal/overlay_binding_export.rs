use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrdSet};
use std::collections::BTreeSet;

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

#[derive(Clone)]
pub(crate) struct UiPortalOverlayBindingOwner {
    generation: WorthUiPreparedApplicationGenerationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    portal_declarations: UiPersistentOrdMap<UiPortalIdentity, UiPortalDeclarationId>,
    declaration_portals:
        UiPersistentOrdMap<UiPortalDeclarationId, UiPersistentOrdSet<UiPortalIdentity>>,
}

impl UiPortalOverlayBindingOwner {
    pub(super) fn commit_graph_generation_succession(
        &mut self,
        succession: &super::overlay_binding_lifecycle::UiPreparedPortalOverlayGraphSuccession,
    ) {
        self.generation = succession.successor().clone();
    }

    pub(super) fn generation(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.generation
    }

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
            portal_declarations: UiPersistentOrdMap::new(),
            declaration_portals: UiPersistentOrdMap::new(),
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
        match self.portal_declarations.get(&portal) {
            Some(current) if *current == declaration => {
                return Err(UiPortalOverlayBindingDenial::DuplicateBinding);
            }
            Some(_) => return Err(UiPortalOverlayBindingDenial::PortalDeclarationConflict),
            None => {}
        }
        self.portal_declarations.insert(portal, declaration);
        let mut portals = self
            .declaration_portals
            .get(&declaration)
            .cloned()
            .unwrap_or_default();
        portals.insert(portal);
        self.declaration_portals.insert(declaration, portals);
        Ok(())
    }

    pub(crate) fn binding_for_portal(
        &self,
        portal: UiPortalIdentity,
    ) -> Option<UiPortalDeclarationId> {
        self.portal_declarations.get(&portal).copied()
    }

    pub(crate) fn bindings(
        &self,
    ) -> impl Iterator<Item = (UiPortalIdentity, UiPortalDeclarationId)> + '_ {
        self.portal_declarations
            .iter()
            .map(|(portal, declaration)| (*portal, *declaration))
    }

    pub(crate) fn remove(&mut self, portal: UiPortalIdentity) {
        if let Some(declaration) = self.portal_declarations.get(&portal).copied() {
            self.portal_declarations.remove(&portal);
            let mut portals = self
                .declaration_portals
                .get(&declaration)
                .cloned()
                .expect("binding retains its declaration index");
            portals.remove(&portal);
            if portals.is_empty() {
                self.declaration_portals.remove(&declaration);
            } else {
                self.declaration_portals.insert(declaration, portals);
            }
        }
    }

    pub(crate) fn portals_for_declaration(
        &self,
        declaration: UiPortalDeclarationId,
    ) -> impl Iterator<Item = UiPortalIdentity> + '_ {
        self.declaration_portals
            .get(&declaration)
            .into_iter()
            .flat_map(|portals| portals.iter().copied())
    }

    pub(crate) fn changed_portals(&self, previous: &Self) -> (Vec<UiPortalIdentity>, usize) {
        let (changed, work) = self
            .portal_declarations
            .changed_keys_with_work(&previous.portal_declarations);
        (changed, work.cursor_steps())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.portal_declarations.is_empty()
    }

    pub(crate) fn binding_count(&self) -> usize {
        self.portal_declarations.len()
    }

    #[allow(
        dead_code,
        reason = "Binding replacement is consumed by owner tests and successor-facing setup."
    )]
    pub(crate) fn replace(
        &mut self,
        bindings: impl IntoIterator<Item = UiPortalOverlayBindingRow>,
    ) -> Result<(), UiPortalOverlayBindingDenial> {
        let mut candidate = Self::new(self.generation.clone(), self.runtime_surface);
        for binding in bindings {
            candidate.bind(binding.declaration(), binding.portal())?;
        }
        *self = candidate;
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
        for (portal, _) in self.portal_declarations.iter() {
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

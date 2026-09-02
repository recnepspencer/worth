#![allow(
    dead_code,
    reason = "Gate 1 retains the Portal overlay binding owner for later overlay composition"
)]

use std::collections::{btree_map::Entry, BTreeMap};

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
    DuplicateBinding,
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

    pub(crate) fn bind(
        &mut self,
        declaration: UiPortalDeclarationId,
        portal: UiPortalIdentity,
    ) -> Result<(), UiPortalOverlayBindingDenial> {
        Self::insert_binding(&mut self.portal_declarations, portal, declaration)
    }

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

    /// Seal the binding export from one current Portal snapshot. The snapshot
    /// is indexed once; every declared binding then resolves through that
    /// carried index instead of rescanning Portal rows.
    pub(crate) fn export(
        &self,
        snapshot: &UiPortalStackSnapshot,
    ) -> Result<UiPortalOverlayBindingOwnerExport, UiPortalOverlayBindingDenial> {
        let by_portal = snapshot
            .rows()
            .iter()
            .map(|row| (row.portal(), row.surface()))
            .collect::<BTreeMap<_, _>>();
        for portal in self.portal_declarations.keys() {
            let Some(surface) = by_portal.get(portal).copied() else {
                return Err(UiPortalOverlayBindingDenial::MissingPortal);
            };
            if surface != self.runtime_surface {
                return Err(UiPortalOverlayBindingDenial::ForeignSurface);
            }
        }
        let rows = snapshot
            .rows()
            .iter()
            .filter_map(|row| {
                self.portal_declarations
                    .get(&row.portal())
                    .map(|declaration| UiPortalOverlayBindingRow::new(*declaration, row.portal()))
            })
            .collect::<Vec<_>>();
        Ok(UiPortalOverlayBindingOwnerExport {
            generation: self.generation.clone(),
            runtime_surface: self.runtime_surface,
            portal_revision: snapshot.owner_revision(),
            rows: rows.into_boxed_slice(),
        })
    }

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

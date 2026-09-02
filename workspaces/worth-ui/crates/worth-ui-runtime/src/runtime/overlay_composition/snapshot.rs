use std::collections::BTreeSet;

use worth_ui_dsl::{
    UiBackdropIdentity, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};
use worth_ui_host_contract::{UiMountIncarnation, UiSemanticSurfaceIdentity};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiBackdropInstanceIdentity {
    declaration: UiBackdropIdentity,
    portal: Option<crate::runtime::portal::UiPortalIdentity>,
    portal_declaration: Option<UiPortalDeclarationId>,
    incarnation: Option<UiMountIncarnation>,
}

impl UiBackdropInstanceIdentity {
    pub(crate) const fn surface_singleton(declaration: UiBackdropIdentity) -> Self {
        Self {
            declaration,
            portal: None,
            portal_declaration: None,
            incarnation: None,
        }
    }

    pub(crate) const fn per_portal_instance(
        declaration: UiBackdropIdentity,
        portal_declaration: UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
        incarnation: UiMountIncarnation,
    ) -> Self {
        Self {
            declaration,
            portal: Some(portal),
            portal_declaration: Some(portal_declaration),
            incarnation: Some(incarnation),
        }
    }

    pub(crate) const fn declaration(self) -> UiBackdropIdentity {
        self.declaration
    }

    pub(crate) const fn portal(self) -> Option<crate::runtime::portal::UiPortalIdentity> {
        self.portal
    }

    pub(crate) const fn portal_declaration(self) -> Option<UiPortalDeclarationId> {
        self.portal_declaration
    }

    pub(crate) const fn incarnation(self) -> Option<UiMountIncarnation> {
        self.incarnation
    }

    pub(crate) fn semantic_digest(self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.declaration.value());
        if let Some(portal) = self.portal {
            digest = fold(digest, 1);
            digest = fold_portal(digest, portal);
        } else {
            digest = fold(digest, 0);
        }
        if let Some(declaration) = self.portal_declaration {
            digest = fold(fold(digest, 1), declaration.value());
        } else {
            digest = fold(digest, 0);
        }
        if let Some(incarnation) = self.incarnation {
            digest = fold(fold(digest, 1), incarnation.diagnostic_value());
        } else {
            digest = fold(digest, 0);
        }
        digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayStackRow {
    instance: UiBackdropInstanceIdentity,
    declaration: UiBackdropIdentity,
    ordinal: u64,
}

impl UiOverlayStackRow {
    pub(crate) const fn new(instance: UiBackdropInstanceIdentity, ordinal: u64) -> Option<Self> {
        if ordinal == 0 {
            return None;
        }
        Some(Self {
            instance,
            declaration: instance.declaration(),
            ordinal,
        })
    }

    pub(crate) const fn instance(self) -> UiBackdropInstanceIdentity {
        self.instance
    }

    pub(crate) const fn declaration(self) -> UiBackdropIdentity {
        self.declaration
    }

    pub(crate) const fn ordinal(self) -> u64 {
        self.ordinal
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayStackSnapshotDenial {
    ZeroPresentationBasis,
    ZeroBackdropRevision,
    UnorderedRows,
    DuplicateInstance,
    DeclarationMismatch,
    PortalSurfaceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayStackSnapshot {
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    surface: UiSemanticSurfaceIdentity,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    presentation_basis: u64,
    portal_snapshot: crate::runtime::portal::UiPortalStackSnapshot,
    backdrop_declaration_revision: u64,
    rows: Box<[UiOverlayStackRow]>,
}

impl UiOverlayStackSnapshot {
    pub(crate) fn seal(
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        surface: UiSemanticSurfaceIdentity,
        declaration_surface: UiSemanticSurfaceDeclarationIdentity,
        presentation_basis: u64,
        portal_snapshot: crate::runtime::portal::UiPortalStackSnapshot,
        backdrop_declaration_revision: u64,
        rows: impl IntoIterator<Item = UiOverlayStackRow>,
    ) -> Result<Self, UiOverlayStackSnapshotDenial> {
        if presentation_basis == 0 {
            return Err(UiOverlayStackSnapshotDenial::ZeroPresentationBasis);
        }
        if backdrop_declaration_revision == 0 {
            return Err(UiOverlayStackSnapshotDenial::ZeroBackdropRevision);
        }
        if portal_snapshot
            .rows()
            .iter()
            .any(|row| row.surface() != surface)
        {
            return Err(UiOverlayStackSnapshotDenial::PortalSurfaceMismatch);
        }
        let rows = rows.into_iter().collect::<Vec<_>>();
        let mut instances = BTreeSet::new();
        if rows
            .windows(2)
            .any(|pair| pair[0].ordinal() >= pair[1].ordinal())
        {
            return Err(UiOverlayStackSnapshotDenial::UnorderedRows);
        }
        for row in &rows {
            if row.declaration() != row.instance().declaration() {
                return Err(UiOverlayStackSnapshotDenial::DeclarationMismatch);
            }
            if !instances.insert(row.instance()) {
                return Err(UiOverlayStackSnapshotDenial::DuplicateInstance);
            }
        }
        Ok(Self {
            application,
            surface,
            declaration_surface,
            presentation_basis,
            portal_snapshot,
            backdrop_declaration_revision,
            rows: rows.into_boxed_slice(),
        })
    }

    pub(crate) const fn application(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.application
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) const fn declaration_surface(&self) -> UiSemanticSurfaceDeclarationIdentity {
        self.declaration_surface
    }

    pub(crate) const fn presentation_basis(&self) -> u64 {
        self.presentation_basis
    }

    pub(crate) const fn portal_revision(&self) -> u64 {
        self.portal_snapshot.owner_revision()
    }

    pub(crate) const fn backdrop_declaration_revision(&self) -> u64 {
        self.backdrop_declaration_revision
    }

    pub(crate) fn rows(&self) -> &[UiOverlayStackRow] {
        &self.rows
    }

    pub(crate) fn contains(
        &self,
        instance: UiBackdropInstanceIdentity,
        declaration: UiBackdropIdentity,
    ) -> bool {
        self.rows
            .iter()
            .any(|row| row.instance() == instance && row.declaration() == declaration)
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.application.session_identity().as_u64());
        digest = fold(
            digest,
            self.application
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
        );
        digest = fold(digest, self.surface.diagnostic_value());
        digest = fold(digest, self.declaration_surface.value());
        digest = fold(digest, self.presentation_basis);
        digest = fold(digest, self.portal_snapshot.owner_revision());
        digest = fold(digest, self.backdrop_declaration_revision);
        digest = fold(digest, self.portal_snapshot.rows().len() as u64);
        for row in self.portal_snapshot.rows() {
            digest = fold_portal(digest, row.portal());
            if let Some(parent) = row.parent() {
                digest = fold(fold(digest, 1), parent.diagnostic_value());
            } else {
                digest = fold(digest, 0);
            }
            digest = fold(digest, row.surface().diagnostic_value());
            digest = fold(digest, row.ordinal().value());
            digest = fold(digest, row.lifecycle() as u64 + 1);
        }
        digest = fold(digest, self.rows.len() as u64);
        for row in &self.rows {
            digest = fold(digest, row.instance().semantic_digest());
            digest = fold(digest, row.declaration().value());
            digest = fold(digest, row.ordinal());
        }
        digest
    }
}

fn fold_portal(mut digest: u64, portal: crate::runtime::portal::UiPortalIdentity) -> u64 {
    digest = fold(digest, portal.diagnostic_value());
    digest = fold(digest, portal.owner().graph_node().digest());
    fold(
        digest,
        portal
            .owner()
            .mounted_instance_identity()
            .diagnostic_value(),
    )
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

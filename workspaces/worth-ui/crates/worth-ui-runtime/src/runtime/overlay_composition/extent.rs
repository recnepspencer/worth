use worth_ui_dsl::{UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiOverlayRegionExtent {
    identity: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
}

impl UiOverlayRegionExtent {
    pub(in crate::runtime::overlay_composition) const fn new(
        identity: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Self {
        Self { identity, bounds }
    }

    pub(crate) const fn identity(self) -> worth_ui_dsl::UiMosaicRegionDeclarationIdentity {
        self.identity
    }

    pub(crate) const fn bounds(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.bounds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlaySurfaceExtentSnapshot {
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    revision: u64,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    regions: Box<[UiOverlayRegionExtent]>,
}

impl UiOverlaySurfaceExtentSnapshot {
    pub(in crate::runtime::overlay_composition) fn seal(
        declaration_surface: UiSemanticSurfaceDeclarationIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        revision: u64,
        viewport: worth_ui_host_contract::UiMountedCanonicalBox,
        regions: impl IntoIterator<Item = UiOverlayRegionExtent>,
    ) -> Result<Self, ()> {
        let mut regions = regions.into_iter().collect::<Vec<_>>();
        regions.sort_by_key(|region| region.identity());
        if regions
            .windows(2)
            .any(|window| window[0].identity() == window[1].identity())
        {
            return Err(());
        }
        Ok(Self {
            declaration_surface,
            runtime_surface,
            revision,
            viewport,
            regions: regions.into_boxed_slice(),
        })
    }

    pub(crate) const fn declaration_surface(&self) -> UiSemanticSurfaceDeclarationIdentity {
        self.declaration_surface
    }
    pub(crate) const fn runtime_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.runtime_surface
    }
    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
    pub(crate) const fn viewport(&self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.viewport
    }
    pub(crate) fn region(
        &self,
        identity: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<UiOverlayRegionExtent> {
        self.regions
            .binary_search_by_key(&identity, |region| region.identity())
            .ok()
            .map(|index| self.regions[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiOverlayMotionBinding {
    portal_declaration: UiPortalDeclarationId,
    portal: crate::runtime::portal::UiPortalIdentity,
    revision: u64,
}

impl UiOverlayMotionBinding {
    pub(in crate::runtime::overlay_composition) const fn new(
        portal_declaration: UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
        revision: u64,
    ) -> Self {
        Self {
            portal_declaration,
            portal,
            revision,
        }
    }
    pub(crate) const fn portal_declaration(self) -> UiPortalDeclarationId {
        self.portal_declaration
    }
    pub(crate) const fn portal(self) -> crate::runtime::portal::UiPortalIdentity {
        self.portal
    }
    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayMotionSnapshot {
    owner_revision: u64,
    rows: Box<[UiOverlayMotionBinding]>,
}

impl UiOverlayMotionSnapshot {
    pub(in crate::runtime::overlay_composition) fn seal(
        owner_revision: u64,
        rows: impl IntoIterator<Item = UiOverlayMotionBinding>,
    ) -> Result<Self, ()> {
        let mut rows = rows.into_iter().collect::<Vec<_>>();
        rows.sort_by_key(|row| (row.portal_declaration(), row.portal()));
        if rows.windows(2).any(|window| {
            (window[0].portal_declaration(), window[0].portal())
                == (window[1].portal_declaration(), window[1].portal())
        }) {
            return Err(());
        }
        Ok(Self {
            owner_revision,
            rows: rows.into_boxed_slice(),
        })
    }
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn lookup(
        &self,
        portal_declaration: UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
    ) -> Option<UiOverlayMotionBinding> {
        self.rows
            .binary_search_by_key(&(portal_declaration, portal), |row| {
                (row.portal_declaration(), row.portal())
            })
            .ok()
            .map(|index| self.rows[index])
    }
}

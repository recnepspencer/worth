#![allow(
    dead_code,
    reason = "Gate 1 retains the mounted overlay extent owner for later publication"
)]

use worth_ui_dsl::{UiMosaicRegionDeclarationIdentity, UiSemanticSurfaceDeclarationIdentity};
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiCommittedOverlayExtentBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl UiCommittedOverlayExtentBounds {
    pub(crate) fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<Self, UiMountedOverlayExtentDenial> {
        if [x, y, width, height].iter().any(|value| !value.is_finite())
            || width < 0.0
            || height < 0.0
            || !(x + width).is_finite()
            || !(y + height).is_finite()
        {
            return Err(UiMountedOverlayExtentDenial::InvalidBounds);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub(crate) const fn x(self) -> f32 {
        self.x
    }

    pub(crate) const fn y(self) -> f32 {
        self.y
    }

    pub(crate) const fn width(self) -> f32 {
        self.width
    }

    pub(crate) const fn height(self) -> f32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedOverlayRegionExtent {
    identity: UiMosaicRegionDeclarationIdentity,
    bounds: UiCommittedOverlayExtentBounds,
}

impl UiMountedOverlayRegionExtent {
    pub(crate) const fn new(
        identity: UiMosaicRegionDeclarationIdentity,
        bounds: UiCommittedOverlayExtentBounds,
    ) -> Self {
        Self { identity, bounds }
    }

    pub(crate) const fn identity(self) -> UiMosaicRegionDeclarationIdentity {
        self.identity
    }

    pub(crate) const fn bounds(self) -> UiCommittedOverlayExtentBounds {
        self.bounds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedOverlayExtentOwnerExport {
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    revision: u64,
    viewport: UiCommittedOverlayExtentBounds,
    regions: Box<[UiMountedOverlayRegionExtent]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedOverlayExtentDenial {
    InvalidRevision,
    InvalidBounds,
    RevisionDidNotAdvance,
    DuplicateRegion,
}

pub(crate) struct UiMountedOverlayExtentOwner {
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    revision: u64,
    viewport: UiCommittedOverlayExtentBounds,
    regions: Box<[UiMountedOverlayRegionExtent]>,
}

impl UiMountedOverlayExtentOwner {
    pub(crate) fn new(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        declaration_surface: UiSemanticSurfaceDeclarationIdentity,
        runtime_surface: UiSemanticSurfaceIdentity,
        revision: u64,
        viewport: UiCommittedOverlayExtentBounds,
        regions: impl IntoIterator<Item = UiMountedOverlayRegionExtent>,
    ) -> Result<Self, UiMountedOverlayExtentDenial> {
        let regions = canonical_regions(regions)?;
        if revision == 0 {
            return Err(UiMountedOverlayExtentDenial::InvalidRevision);
        }
        Ok(Self {
            generation,
            declaration_surface,
            runtime_surface,
            revision,
            viewport,
            regions,
        })
    }

    pub(crate) fn replace(
        &mut self,
        revision: u64,
        viewport: UiCommittedOverlayExtentBounds,
        regions: impl IntoIterator<Item = UiMountedOverlayRegionExtent>,
    ) -> Result<(), UiMountedOverlayExtentDenial> {
        if revision <= self.revision {
            return Err(UiMountedOverlayExtentDenial::RevisionDidNotAdvance);
        }
        self.regions = canonical_regions(regions)?;
        self.revision = revision;
        self.viewport = viewport;
        Ok(())
    }

    pub(crate) fn export(&self) -> UiMountedOverlayExtentOwnerExport {
        UiMountedOverlayExtentOwnerExport {
            generation: self.generation.clone(),
            declaration_surface: self.declaration_surface,
            runtime_surface: self.runtime_surface,
            revision: self.revision,
            viewport: self.viewport,
            regions: self.regions.clone(),
        }
    }
}

impl UiMountedOverlayExtentOwnerExport {
    pub(crate) fn generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        &self.generation
    }

    pub(crate) const fn declaration_surface(&self) -> UiSemanticSurfaceDeclarationIdentity {
        self.declaration_surface
    }

    pub(crate) const fn runtime_surface(&self) -> UiSemanticSurfaceIdentity {
        self.runtime_surface
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn viewport(&self) -> UiCommittedOverlayExtentBounds {
        self.viewport
    }

    pub(crate) fn regions(&self) -> &[UiMountedOverlayRegionExtent] {
        &self.regions
    }
}

fn canonical_regions(
    regions: impl IntoIterator<Item = UiMountedOverlayRegionExtent>,
) -> Result<Box<[UiMountedOverlayRegionExtent]>, UiMountedOverlayExtentDenial> {
    let mut regions = regions.into_iter().collect::<Vec<_>>();
    regions.sort_by_key(|region| region.identity());
    if regions
        .windows(2)
        .any(|window| window[0].identity() == window[1].identity())
    {
        return Err(UiMountedOverlayExtentDenial::DuplicateRegion);
    }
    Ok(regions.into_boxed_slice())
}

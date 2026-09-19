#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiMosaicRegionDeclarationIdentity(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropExtentBasis {
    SurfaceViewport(super::UiSemanticSurfaceDeclarationIdentity),
    PresentedMosaicRegion {
        surface: super::UiSemanticSurfaceDeclarationIdentity,
        region: UiMosaicRegionDeclarationIdentity,
    },
}

impl UiMosaicRegionDeclarationIdentity {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl UiBackdropExtentBasis {
    pub const fn surface(self) -> super::UiSemanticSurfaceDeclarationIdentity {
        match self {
            Self::SurfaceViewport(surface) | Self::PresentedMosaicRegion { surface, .. } => surface,
        }
    }
}

//! Bounded, allocation-independent vector geometry exchanged across renderers.

mod vector_path;

pub use vector_path::{NormalizedPoint, VectorPath, VectorPathDenial, VectorPathSegment};

mod logical_length;
mod surface_geometry;
pub use logical_length::{
    UiAppearanceLogicalLength, UiAppearanceNegativeLength,
    UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};
pub use surface_geometry::{
    UiSoftShadowGeometry, UiSurfaceGeometry, UiSurfaceGeometryDenial, UiVectorSurfaceGeometry,
};

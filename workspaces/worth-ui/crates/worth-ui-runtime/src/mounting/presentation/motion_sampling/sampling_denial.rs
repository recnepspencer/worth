#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentationGeometrySamplingDenial {
    NonFinite,
    NegativeExtent,
    MissingSemanticBasis,
    PresentationBindingChanged,
    PresentationSurfaceChanged,
}

impl From<super::super::UiTruthGeometryDenial> for UiPresentationGeometrySamplingDenial {
    fn from(denial: super::super::UiTruthGeometryDenial) -> Self {
        match denial {
            super::super::UiTruthGeometryDenial::NonFinite => Self::NonFinite,
            super::super::UiTruthGeometryDenial::NegativeExtent => Self::NegativeExtent,
        }
    }
}

impl From<super::super::UiRebaseDenial> for UiPresentationGeometrySamplingDenial {
    fn from(denial: super::super::UiRebaseDenial) -> Self {
        match denial {
            super::super::UiRebaseDenial::SurfaceChanged => Self::PresentationSurfaceChanged,
            super::super::UiRebaseDenial::BindingChanged => Self::PresentationBindingChanged,
        }
    }
}

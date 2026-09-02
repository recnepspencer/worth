use worth_ui_dsl::UiAppearanceStateAxis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceStateAdapterDenial {
    MissingOwner(UiAppearanceStateAxis),
    MissingSource(UiAppearanceStateAxis),
    AmbiguousSource(UiAppearanceStateAxis),
    StaleSource(UiAppearanceStateAxis),
    ForeignSource(UiAppearanceStateAxis),
}

impl UiAppearanceStateAdapterDenial {
    pub(crate) const fn axis(self) -> UiAppearanceStateAxis {
        match self {
            Self::MissingOwner(axis)
            | Self::MissingSource(axis)
            | Self::AmbiguousSource(axis)
            | Self::StaleSource(axis)
            | Self::ForeignSource(axis) => axis,
        }
    }
}

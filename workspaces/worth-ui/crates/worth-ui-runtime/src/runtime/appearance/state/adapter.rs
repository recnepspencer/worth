use worth_ui_dsl::UiAppearanceStateAxis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceStateAdapterDenial {
    MissingOwner(UiAppearanceStateAxis),
    MissingSource(UiAppearanceStateAxis),
    AmbiguousSource(UiAppearanceStateAxis),
    StaleSource(UiAppearanceStateAxis),
    ForeignSource(UiAppearanceStateAxis),
}

impl UiAppearanceStateAdapterDenial {}

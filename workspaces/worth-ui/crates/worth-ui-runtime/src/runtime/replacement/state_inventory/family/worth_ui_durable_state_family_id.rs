#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthUiDurableStateFamilyId {
    FocusChain,
    ScrollAnchor,
    SelectionRange,
    TextEditBuffer,
    SplitterPosition,
    TabState,
    PanelVisibility,
    Custom(String),
}

impl WorthUiDurableStateFamilyId {
    pub fn custom(id: impl Into<String>) -> Self {
        Self::Custom(id.into())
    }

    pub fn reserved_platform_families() -> &'static [Self] {
        &[
            Self::FocusChain,
            Self::ScrollAnchor,
            Self::SelectionRange,
            Self::TextEditBuffer,
            Self::SplitterPosition,
            Self::TabState,
            Self::PanelVisibility,
        ]
    }

    pub fn is_explicit_custom_family(&self) -> bool {
        match self {
            Self::Custom(id) => !id.trim().is_empty(),
            Self::FocusChain
            | Self::ScrollAnchor
            | Self::SelectionRange
            | Self::TextEditBuffer
            | Self::SplitterPosition
            | Self::TabState
            | Self::PanelVisibility => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionRequest {
    SelectSingle(super::UiSelectionStableKey),
    #[cfg(any(test, feature = "certification-support"))]
    ToggleMultiple(super::UiSelectionStableKey),
    Add(super::UiSelectionStableKey),
    Remove(super::UiSelectionStableKey),
    SelectRange {
        target: super::UiSelectionStableKey,
        extend: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSelectionRequestDenial {
    UnknownOwner,
    StaleOwnerIncarnation,
    CatalogUnavailable,
    CatalogCapacityExceeded,
    DuplicateCatalogKey,
    ForeignItemKeyFamily,
    UnknownKey,
    RangeNotSupported,
    MultipleNotSupported,
    MissingRangeAnchor,
    RevisionExhausted,
    CounterOverflow,
}

impl UiSelectionRequest {
    pub(in crate::runtime) const fn application_item_key(
        self,
    ) -> Option<crate::runtime::UiApplicationItemKey> {
        match self {
            Self::SelectSingle(key)
            | Self::Add(key)
            | Self::Remove(key)
            | Self::SelectRange { target: key, .. } => Some(key.application_key()),
            #[cfg(any(test, feature = "certification-support"))]
            Self::ToggleMultiple(key) => Some(key.application_key()),
        }
    }
}

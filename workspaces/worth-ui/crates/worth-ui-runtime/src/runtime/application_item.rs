use core::num::NonZeroU64;

/// Application-declared key family. It classifies stable item identity without
/// borrowing row, cursor, or operational identity from a data runtime.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiApplicationItemKeyFamily(UiApplicationItemKeyFamilyBasis);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum UiApplicationItemKeyFamilyBasis {
    ProjectionInput(worth_ui_query_binding::UiProjectionInputSlot),
    #[cfg(any(test, feature = "certification-support"))]
    Recorded(NonZeroU64),
}

/// Stable application item identity within one declared key family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiApplicationItemKey {
    family: UiApplicationItemKeyFamily,
    value: UiApplicationItemKeyValue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum UiApplicationItemKeyValue {
    Recorded(NonZeroU64),
}

impl UiApplicationItemKeyFamily {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn new(value: NonZeroU64) -> Self {
        Self(UiApplicationItemKeyFamilyBasis::Recorded(value))
    }

    pub(crate) const fn from_projection_input(
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Self {
        Self(UiApplicationItemKeyFamilyBasis::ProjectionInput(slot))
    }

    pub(crate) const fn projection_input_slot(
        self,
    ) -> Option<worth_ui_query_binding::UiProjectionInputSlot> {
        match self.0 {
            UiApplicationItemKeyFamilyBasis::ProjectionInput(slot) => Some(slot),
            #[cfg(any(test, feature = "certification-support"))]
            UiApplicationItemKeyFamilyBasis::Recorded(_) => None,
        }
    }

    pub(crate) const fn diagnostic_value(self) -> u64 {
        match self.0 {
            UiApplicationItemKeyFamilyBasis::ProjectionInput(slot) => {
                (slot.index() as u64) | (1_u64 << 63)
            }
            #[cfg(any(test, feature = "certification-support"))]
            UiApplicationItemKeyFamilyBasis::Recorded(value) => value.get(),
        }
    }
}

impl UiApplicationItemKey {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn new(family: UiApplicationItemKeyFamily, value: NonZeroU64) -> Self {
        Self {
            family,
            value: UiApplicationItemKeyValue::Recorded(value),
        }
    }

    pub(crate) const fn from_projection_mapping(
        family: UiApplicationItemKeyFamily,
        value: NonZeroU64,
    ) -> Self {
        Self {
            family,
            value: UiApplicationItemKeyValue::Recorded(value),
        }
    }

    pub(crate) const fn family(self) -> UiApplicationItemKeyFamily {
        self.family
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn value(self) -> NonZeroU64 {
        match self.value {
            UiApplicationItemKeyValue::Recorded(value) => value,
        }
    }
}

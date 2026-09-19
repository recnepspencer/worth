#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPrimaryPointerKind {
    Mouse,
    Stylus,
    Touch,
}

impl UiPrimaryPointerKind {
    pub(crate) const fn from_host(kind: worth_ui_host_contract::UiHostPointerDeviceKind) -> Self {
        match kind {
            worth_ui_host_contract::UiHostPointerDeviceKind::Mouse => Self::Mouse,
            worth_ui_host_contract::UiHostPointerDeviceKind::Stylus => Self::Stylus,
            worth_ui_host_contract::UiHostPointerDeviceKind::Touch => Self::Touch,
        }
    }

    pub(crate) const fn host_kind(self) -> worth_ui_host_contract::UiHostPointerDeviceKind {
        match self {
            Self::Mouse => worth_ui_host_contract::UiHostPointerDeviceKind::Mouse,
            Self::Stylus => worth_ui_host_contract::UiHostPointerDeviceKind::Stylus,
            Self::Touch => worth_ui_host_contract::UiHostPointerDeviceKind::Touch,
        }
    }
}

pub(crate) const fn primary_pointer_admitted(kind: UiPrimaryPointerKind) -> bool {
    matches!(
        kind,
        UiPrimaryPointerKind::Mouse | UiPrimaryPointerKind::Stylus
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn touch_cannot_become_primary_pointer() {
        assert!(super::primary_pointer_admitted(
            super::UiPrimaryPointerKind::Mouse
        ));
        assert!(super::primary_pointer_admitted(
            super::UiPrimaryPointerKind::Stylus
        ));
        assert!(!super::primary_pointer_admitted(
            super::UiPrimaryPointerKind::Touch
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiDeclaredPortalSurfaceContract {
    MountedOverlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiDeclaredPortalPlacementGeometry {
    preferred_width: u16,
    maximum_height: u16,
    anchor_gap: u8,
    viewport_margin: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(crate) enum UiDeclaredPortalPlacementGeometryDenial {
    EmptyExtent,
    MarginConsumesExtent,
}

impl UiDeclaredPortalSurfaceContract {
    pub(crate) const fn family(self) -> crate::capability::UiRuntimeServiceFamily {
        crate::capability::UiRuntimeServiceFamily::Portal
    }
}

impl UiDeclaredPortalPlacementGeometry {
    pub(crate) const fn dropdown() -> Self {
        Self {
            preferred_width: 280,
            maximum_height: 320,
            anchor_gap: 8,
            viewport_margin: 16,
        }
    }

    pub(crate) const fn modal_dialog() -> Self {
        Self {
            preferred_width: 280,
            maximum_height: 320,
            anchor_gap: 8,
            viewport_margin: 24,
        }
    }

    #[cfg(test)]
    pub(crate) const fn checked(
        preferred_width: u16,
        maximum_height: u16,
        anchor_gap: u8,
        viewport_margin: u8,
    ) -> Result<Self, UiDeclaredPortalPlacementGeometryDenial> {
        if preferred_width == 0 || maximum_height == 0 {
            return Err(UiDeclaredPortalPlacementGeometryDenial::EmptyExtent);
        }
        if viewport_margin as u16 * 2 >= preferred_width
            || viewport_margin as u16 * 2 >= maximum_height
        {
            return Err(UiDeclaredPortalPlacementGeometryDenial::MarginConsumesExtent);
        }
        Ok(Self {
            preferred_width,
            maximum_height,
            anchor_gap,
            viewport_margin,
        })
    }

    pub(crate) const fn preferred_width(self) -> u16 {
        self.preferred_width
    }

    pub(crate) const fn maximum_height(self) -> u16 {
        self.maximum_height
    }

    pub(crate) const fn anchor_gap(self) -> u8 {
        self.anchor_gap
    }

    pub(crate) const fn viewport_margin(self) -> u8 {
        self.viewport_margin
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiPortalPolicyKind {
    Dropdown,
    Popover,
    ModalDialog,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiPortalPolicy {
    kind: UiPortalPolicyKind,
    restore_focus: bool,
    dismiss_on_escape: bool,
    dismiss_on_outside_press: bool,
    dismiss_on_accepted_selection: bool,
    dismiss_on_anchor_loss: bool,
}

impl UiPortalPolicy {
    pub const fn dropdown() -> Self {
        Self::new(UiPortalPolicyKind::Dropdown, true, true, true)
    }

    pub const fn popover() -> Self {
        Self::new(UiPortalPolicyKind::Popover, true, true, true)
    }

    pub const fn modal_dialog() -> Self {
        Self::new(UiPortalPolicyKind::ModalDialog, true, true, false)
    }

    const fn new(
        kind: UiPortalPolicyKind,
        restore_focus: bool,
        dismiss_on_escape: bool,
        dismiss_on_outside_press: bool,
    ) -> Self {
        Self {
            kind,
            restore_focus,
            dismiss_on_escape,
            dismiss_on_outside_press,
            dismiss_on_accepted_selection: true,
            dismiss_on_anchor_loss: true,
        }
    }

    pub const fn with_focus_restoration(mut self, enabled: bool) -> Self {
        self.restore_focus = enabled;
        self
    }

    pub const fn with_outside_press_dismissal(mut self, enabled: bool) -> Self {
        self.dismiss_on_outside_press = enabled;
        self
    }

    pub const fn with_escape_dismissal(mut self, enabled: bool) -> Self {
        self.dismiss_on_escape = enabled;
        self
    }

    pub const fn with_accepted_selection_dismissal(mut self, enabled: bool) -> Self {
        self.dismiss_on_accepted_selection = enabled;
        self
    }

    pub const fn with_anchor_loss_dismissal(mut self, enabled: bool) -> Self {
        self.dismiss_on_anchor_loss = enabled;
        self
    }

    pub const fn kind(self) -> UiPortalPolicyKind {
        self.kind
    }

    pub const fn restores_focus(self) -> bool {
        self.restore_focus
    }

    pub const fn dismisses_on_escape(self) -> bool {
        self.dismiss_on_escape
    }

    pub const fn dismisses_on_outside_press(self) -> bool {
        self.dismiss_on_outside_press
    }

    pub const fn dismisses_on_accepted_selection(self) -> bool {
        self.dismiss_on_accepted_selection
    }

    pub const fn dismisses_on_anchor_loss(self) -> bool {
        self.dismiss_on_anchor_loss
    }

    pub(crate) const fn digest_basis(self) -> u64 {
        self.kind as u64
            | (self.restore_focus as u64) << 8
            | (self.dismiss_on_escape as u64) << 9
            | (self.dismiss_on_outside_press as u64) << 10
            | (self.dismiss_on_accepted_selection as u64) << 11
            | (self.dismiss_on_anchor_loss as u64) << 12
    }
}

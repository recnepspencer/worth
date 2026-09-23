#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiDeclaredScrollOwnershipContract {
    RuntimeOwnedOffset,
}

impl UiDeclaredScrollOwnershipContract {
    pub(crate) const fn family(self) -> crate::capability::UiRuntimeServiceFamily {
        crate::capability::UiRuntimeServiceFamily::Scroll
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiScrollAnchorBehavior {
    RebaseStableAnchor,
    ClampOffset,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiScrollRevealAlignment {
    Nearest,
    End,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiScrollPolicy {
    bubble_remainder: bool,
    anchor: UiScrollAnchorBehavior,
    reveal_alignment: UiScrollRevealAlignment,
    wheel: super::UiScrollWheelBehavior,
}

impl UiScrollPolicy {
    pub const fn nested_region() -> Self {
        Self {
            bubble_remainder: true,
            anchor: UiScrollAnchorBehavior::RebaseStableAnchor,
            reveal_alignment: UiScrollRevealAlignment::Nearest,
            wheel: super::UiScrollWheelBehavior::immediate(),
        }
    }

    pub const fn with_remainder_bubbling(mut self, enabled: bool) -> Self {
        self.bubble_remainder = enabled;
        self
    }

    pub const fn with_anchor_behavior(mut self, anchor: UiScrollAnchorBehavior) -> Self {
        self.anchor = anchor;
        self
    }

    pub const fn with_reveal_alignment(mut self, alignment: UiScrollRevealAlignment) -> Self {
        self.reveal_alignment = alignment;
        self
    }

    pub const fn with_wheel_behavior(mut self, behavior: super::UiScrollWheelBehavior) -> Self {
        self.wheel = behavior;
        self
    }

    pub const fn bubbles_remainder(self) -> bool {
        self.bubble_remainder
    }

    pub const fn anchor_behavior(self) -> UiScrollAnchorBehavior {
        self.anchor
    }

    pub const fn reveal_alignment(self) -> UiScrollRevealAlignment {
        self.reveal_alignment
    }

    pub const fn wheel_behavior(self) -> super::UiScrollWheelBehavior {
        self.wheel
    }

    pub(crate) const fn digest_basis(self) -> u64 {
        self.bubble_remainder as u64
            | (self.anchor as u64) << 8
            | (self.reveal_alignment as u64) << 16
            | self.wheel.digest_basis() << 24
    }
}

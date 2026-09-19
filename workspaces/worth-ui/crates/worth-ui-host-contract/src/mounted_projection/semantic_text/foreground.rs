#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedTextPaintSpanIdentity([u8; 32]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedTextForegroundSpan {
    original_range: crate::UiTextOriginalRange,
    color: super::super::UiMountedRgba8,
    identity: UiMountedTextPaintSpanIdentity,
}

impl UiMountedTextPaintSpanIdentity {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    pub const fn digest(self) -> [u8; 32] {
        self.0
    }
}

impl UiMountedTextForegroundSpan {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(
        original_range: crate::UiTextOriginalRange,
        color: super::super::UiMountedRgba8,
        identity: UiMountedTextPaintSpanIdentity,
    ) -> Self {
        Self {
            original_range,
            color,
            identity,
        }
    }

    pub const fn original_range(self) -> crate::UiTextOriginalRange {
        self.original_range
    }

    pub const fn color(self) -> super::super::UiMountedRgba8 {
        self.color
    }

    pub const fn identity(self) -> UiMountedTextPaintSpanIdentity {
        self.identity
    }
}

use sha2::{Digest, Sha256};

use crate::capability::ThemeTokenId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSemanticTextSpanContract {
    original_range: worth_ui_host_contract::UiTextOriginalRange,
    foreground_token: ThemeTokenId,
    style: worth_ui_text::UiTextStyle,
    paint_identity: [u8; 32],
    appearance_foreground: bool,
}

impl ComponentSemanticTextSpanContract {
    pub fn new(
        original_range: worth_ui_host_contract::UiTextOriginalRange,
        foreground_token: ThemeTokenId,
        style: worth_ui_text::UiTextStyle,
    ) -> Option<Self> {
        if original_range.is_empty() {
            return None;
        }
        let paint_identity = paint_identity(&foreground_token, original_range);
        Some(Self {
            original_range,
            foreground_token,
            style,
            paint_identity,
            appearance_foreground: false,
        })
    }

    pub const fn original_range(&self) -> worth_ui_host_contract::UiTextOriginalRange {
        self.original_range
    }

    /// Adopts the node's resolved appearance foreground for this original range.
    /// The range, style, and paint identity remain owned by semantic text.
    pub fn with_appearance_foreground(mut self) -> Self {
        self.appearance_foreground = true;
        self
    }

    pub const fn uses_appearance_foreground(&self) -> bool {
        self.appearance_foreground
    }

    pub fn foreground_token(&self) -> &ThemeTokenId {
        &self.foreground_token
    }

    pub const fn style(&self) -> &worth_ui_text::UiTextStyle {
        &self.style
    }

    pub(crate) const fn paint_identity(&self) -> [u8; 32] {
        self.paint_identity
    }
}

pub(crate) fn whole_paragraph_paint_identity(token: &ThemeTokenId) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-ui-mounted-text-whole-paragraph-paint-v1\0");
    digest.update(token.as_str().as_bytes());
    digest.finalize().into()
}

fn paint_identity(
    token: &ThemeTokenId,
    range: worth_ui_host_contract::UiTextOriginalRange,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-ui-mounted-text-authored-paint-span-v1\0");
    digest.update(token.as_str().as_bytes());
    digest.update(range.start().to_le_bytes());
    digest.update(range.end().to_le_bytes());
    digest.finalize().into()
}

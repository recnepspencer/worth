#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedTextForegroundAppearanceMechanic {
    node_receipt: crate::UiMountedNodeReceiptIdentity,
    paint_span: crate::UiMountedTextPaintSpanIdentity,
    foreground: super::UiMountedAppearanceColor,
    opacity: super::UiMountedAppearanceOpacity,
    projection: super::UiMountedNodeAppearanceAttribution,
}

#[doc(hidden)]
pub struct UiMountedTextForegroundAppearanceCompletionInput {
    pub issuer: crate::UiMountedNodeReceiptIssuer,
    pub node_receipt: crate::UiMountedNodeReceiptIdentity,
    pub paint_span: crate::UiMountedTextPaintSpanIdentity,
    pub foreground: super::UiMountedAppearanceColor,
    pub opacity: super::UiMountedAppearanceOpacity,
    pub projection: super::UiMountedNodeAppearanceAttribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedTextForegroundAppearanceCompletionDenial {
    NodeReceiptFrameMismatch,
    ProjectionIssuerMismatch,
}

impl UiMountedTextForegroundAppearanceMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedTextForegroundAppearanceCompletionInput,
    ) -> Result<Self, UiMountedTextForegroundAppearanceCompletionDenial> {
        if input.node_receipt.frame() != input.issuer.frame_identity() {
            return Err(
                UiMountedTextForegroundAppearanceCompletionDenial::NodeReceiptFrameMismatch,
            );
        }
        if !input.projection.matches_issuer(input.issuer) {
            return Err(
                UiMountedTextForegroundAppearanceCompletionDenial::ProjectionIssuerMismatch,
            );
        }
        Ok(Self {
            node_receipt: input.node_receipt,
            paint_span: input.paint_span,
            foreground: input.foreground,
            opacity: input.opacity,
            projection: input.projection,
        })
    }
    pub const fn node_receipt(&self) -> crate::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn paint_span(&self) -> crate::UiMountedTextPaintSpanIdentity {
        self.paint_span
    }
    pub const fn foreground(&self) -> super::UiMountedAppearanceColor {
        self.foreground
    }
    pub const fn opacity(&self) -> super::UiMountedAppearanceOpacity {
        self.opacity
    }
    pub const fn projection(&self) -> super::UiMountedNodeAppearanceAttribution {
        self.projection
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedTextForegroundAppearanceMechanic {
    node_receipt: crate::UiMountedNodeReceiptIdentity,
    command: crate::UiMountedPaintCommandIdentity,
    paint_span: crate::UiMountedTextPaintSpanIdentity,
    foreground: super::UiMountedAppearanceColor,
    opacity: crate::UiMountedPresentationOpacity,
    projection: super::UiMountedNodeAppearanceAttribution,
}

#[doc(hidden)]
pub struct UiMountedTextForegroundAppearanceCompletionInput {
    pub issuer: crate::UiMountedNodeReceiptIssuer,
    pub node_receipt: crate::UiMountedNodeReceiptIdentity,
    pub command: crate::UiMountedPaintCommandIdentity,
    pub paint_span: crate::UiMountedTextPaintSpanIdentity,
    pub foreground: super::UiMountedAppearanceColor,
    pub opacity: crate::UiMountedPresentationOpacity,
    pub projection: super::UiMountedNodeAppearanceAttribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedTextForegroundAppearanceCompletionDenial {
    NodeReceiptFrameMismatch,
    CommandMismatch,
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
        if input.command.mounted_instance() != input.node_receipt.mounted_instance()
            || input.command.semantic_text_identity_parts().is_none()
        {
            return Err(UiMountedTextForegroundAppearanceCompletionDenial::CommandMismatch);
        }
        if !input.projection.matches_issuer(input.issuer) {
            return Err(
                UiMountedTextForegroundAppearanceCompletionDenial::ProjectionIssuerMismatch,
            );
        }
        Ok(Self {
            node_receipt: input.node_receipt,
            command: input.command,
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
    pub const fn command(&self) -> crate::UiMountedPaintCommandIdentity {
        self.command
    }
    pub const fn foreground(&self) -> super::UiMountedAppearanceColor {
        self.foreground
    }
    pub const fn opacity(&self) -> crate::UiMountedPresentationOpacity {
        self.opacity
    }
    pub const fn projection(&self) -> super::UiMountedNodeAppearanceAttribution {
        self.projection
    }
}

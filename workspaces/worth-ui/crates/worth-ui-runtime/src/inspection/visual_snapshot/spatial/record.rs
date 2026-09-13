use super::geometry::UiSpatialRect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiVisibleOpacity {
    Opaque,
    Composited(u8),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiValidatedClipLineage {
    canonical: worth_ui_host_contract::UiMountedCanonicalBox,
    realized: worth_ui_host_contract::UiMountedCanonicalBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiVisibleRegionRecord {
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    region: UiSpatialRect,
    layer_order: u32,
    paint_order: u32,
    opacity: UiVisibleOpacity,
    clip_lineage: UiValidatedClipLineage,
    source_projection_digest: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiHitTestRegionRecord {
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    region: UiSpatialRect,
    total_order: worth_ui_host_contract::UiMountedHitTestOrder,
    source_projection_digest: u64,
}

pub(crate) trait UiSpatialRecord {
    fn region(&self) -> UiSpatialRect;
    fn semantic_digest(&self) -> u64;
}

impl UiVisibleRegionRecord {
    pub(crate) fn appearance(
        mechanic: crate::mounting::UiMountedAppearancePaintBasis,
        realized_clip: worth_ui_host_contract::UiMountedCanonicalBox,
        region: UiSpatialRect,
    ) -> Self {
        Self {
            node_receipt: mechanic.node_receipt(),
            region,
            layer_order: mechanic.semantic_order(),
            paint_order: mechanic.semantic_order(),
            opacity: match mechanic.alpha() {
                Some(u8::MAX) => UiVisibleOpacity::Opaque,
                Some(alpha) => UiVisibleOpacity::Composited(alpha),
                None => UiVisibleOpacity::Unsupported,
            },
            clip_lineage: UiValidatedClipLineage {
                canonical: mechanic.clip(),
                realized: realized_clip,
            },
            source_projection_digest: mechanic.source_digest(),
        }
    }

    pub(crate) fn unsupported(
        mechanic: crate::mounting::UiMountedUnsupportedPaintBasis,
        realized_clip: worth_ui_host_contract::UiMountedCanonicalBox,
        region: UiSpatialRect,
    ) -> Self {
        Self {
            node_receipt: mechanic.node_receipt(),
            region,
            layer_order: mechanic.semantic_order(),
            paint_order: mechanic.semantic_order(),
            opacity: UiVisibleOpacity::Unsupported,
            clip_lineage: UiValidatedClipLineage {
                canonical: mechanic.clip(),
                realized: realized_clip,
            },
            source_projection_digest: mechanic.source_digest(),
        }
    }

    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn layer_order(self) -> u32 {
        self.layer_order
    }

    pub(crate) const fn paint_order(self) -> u32 {
        self.paint_order
    }

    pub(crate) const fn opacity(self) -> UiVisibleOpacity {
        self.opacity
    }

    pub(crate) fn inspection_region(self) -> worth_ui_inspection::UiClientPhysicalRect {
        self.region.inspection_rect()
    }

    #[cfg(test)]
    pub(crate) const fn clip_lineage(self) -> UiValidatedClipLineage {
        self.clip_lineage
    }

    #[cfg(test)]
    pub(crate) const fn source_projection_digest(self) -> u64 {
        self.source_projection_digest
    }
}

impl UiHitTestRegionRecord {
    pub(crate) const fn validated(
        mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
        region: UiSpatialRect,
    ) -> Self {
        Self {
            node_receipt: mechanic.node_receipt(),
            region,
            total_order: mechanic.order(),
            source_projection_digest: mechanic.semantic_digest(),
        }
    }

    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn total_order(self) -> worth_ui_host_contract::UiMountedHitTestOrder {
        self.total_order
    }

    pub(crate) fn inspection_region(self) -> worth_ui_inspection::UiClientPhysicalRect {
        self.region.inspection_rect()
    }

    #[cfg(test)]
    pub(crate) const fn source_projection_digest(self) -> u64 {
        self.source_projection_digest
    }
}

impl UiSpatialRecord for UiVisibleRegionRecord {
    fn region(&self) -> UiSpatialRect {
        self.region
    }

    fn semantic_digest(&self) -> u64 {
        let opacity = match self.opacity {
            UiVisibleOpacity::Opaque => u64::from(u8::MAX),
            UiVisibleOpacity::Composited(alpha) => u64::from(alpha),
            UiVisibleOpacity::Unsupported => u64::MAX,
        };
        [
            self.node_receipt.diagnostic_value(),
            u64::from(self.layer_order),
            u64::from(self.paint_order),
            opacity,
            box_digest(self.clip_lineage.canonical),
            box_digest(self.clip_lineage.realized),
            self.source_projection_digest,
        ]
        .into_iter()
        .fold(0x7669_7369_626c_6501, fold)
    }
}

impl UiSpatialRecord for UiHitTestRegionRecord {
    fn region(&self) -> UiSpatialRect {
        self.region
    }

    fn semantic_digest(&self) -> u64 {
        [
            self.node_receipt.diagnostic_value(),
            u64::from(self.total_order.rank()),
            self.source_projection_digest,
        ]
        .into_iter()
        .fold(0x6869_745f_7465_7301, fold)
    }
}

impl UiValidatedClipLineage {
    #[cfg(test)]
    pub(crate) const fn canonical(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.canonical
    }

    #[cfg(test)]
    pub(crate) const fn realized(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.realized
    }
}

fn box_digest(bounds: worth_ui_host_contract::UiMountedCanonicalBox) -> u64 {
    [
        u64::from(bounds.x().to_bits()),
        u64::from(bounds.y().to_bits()),
        u64::from(bounds.width().to_bits()),
        u64::from(bounds.height().to_bits()),
    ]
    .into_iter()
    .fold(0x636c_6970_6c69_6e65, fold)
}

fn fold(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(0x100000001b3)
}

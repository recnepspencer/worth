#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedPaintCommandIdentity {
    mounted_instance: crate::UiMountedInstanceIdentity,
    family: UiMountedPaintCommandFamily,
    semantic_slot: u16,
    collection_row: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum UiMountedPaintCommandFamily {
    FilledRect,
    PortalOverlay,
    SemanticText,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiMountedPaintCommand {
    FilledRect {
        identity: UiMountedPaintCommandIdentity,
        mechanic: crate::UiMountedFilledRectMechanic,
    },
    PortalOverlay {
        identity: UiMountedPaintCommandIdentity,
        mechanic: crate::UiMountedPortalOverlayMechanic,
    },
    SemanticText {
        identity: UiMountedPaintCommandIdentity,
        mechanic: crate::UiMountedSemanticTextMechanic,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiMountedPaintCommandChange {
    Insert(UiMountedPaintCommand),
    Replace {
        predecessor: UiMountedPaintCommandIdentity,
        successor: UiMountedPaintCommand,
    },
    Remove(UiMountedPaintCommandIdentity),
}

impl UiMountedPaintCommandChange {
    pub fn replacement(
        predecessor: UiMountedPaintCommandIdentity,
        successor: UiMountedPaintCommand,
    ) -> Self {
        Self::Replace {
            predecessor,
            successor,
        }
    }
}

impl UiMountedPaintCommandIdentity {
    #[doc(hidden)]
    pub fn filled_rect_from_correspondence(
        mounted_instance: crate::UiMountedInstanceIdentity,
    ) -> Self {
        Self {
            mounted_instance,
            family: UiMountedPaintCommandFamily::FilledRect,
            semantic_slot: 0,
            collection_row: None,
        }
    }

    #[doc(hidden)]
    pub fn semantic_text_from_correspondence(
        mounted_instance: crate::UiMountedInstanceIdentity,
        semantic_slot: u16,
        collection_row: Option<[u8; 32]>,
    ) -> Self {
        Self {
            mounted_instance,
            family: UiMountedPaintCommandFamily::SemanticText,
            semantic_slot,
            collection_row,
        }
    }

    #[doc(hidden)]
    pub fn filled_rect(mechanic: &crate::UiMountedFilledRectMechanic) -> Self {
        Self::filled_rect_from_correspondence(mechanic.mounted_instance())
    }

    #[doc(hidden)]
    pub fn semantic_text(mechanic: &crate::UiMountedSemanticTextMechanic) -> Self {
        let semantic_slot = match mechanic.slot() {
            crate::UiSemanticTextSlot::Value => 0,
            crate::UiSemanticTextSlot::CollectionValue {
                selected_field_ordinal,
            } => selected_field_ordinal.saturating_add(1),
            crate::UiSemanticTextSlot::Posture => u16::MAX,
        };
        Self {
            mounted_instance: mechanic.mounted_instance(),
            family: UiMountedPaintCommandFamily::SemanticText,
            semantic_slot,
            collection_row: mechanic
                .collection_row()
                .map(|row| row.correlation_digest()),
        }
    }

    #[doc(hidden)]
    pub fn portal_overlay(mechanic: &crate::UiMountedPortalOverlayMechanic) -> Self {
        Self {
            mounted_instance: mechanic.owner(),
            family: UiMountedPaintCommandFamily::PortalOverlay,
            semantic_slot: 0,
            collection_row: Some(portal_identity_digest(mechanic.portal_identity())),
        }
    }

    pub const fn mounted_instance(self) -> crate::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    #[doc(hidden)]
    pub const fn semantic_text_identity_parts(self) -> Option<(u16, Option<[u8; 32]>)> {
        match self.family {
            UiMountedPaintCommandFamily::SemanticText => {
                Some((self.semantic_slot, self.collection_row))
            }
            UiMountedPaintCommandFamily::FilledRect
            | UiMountedPaintCommandFamily::PortalOverlay => None,
        }
    }

    pub(super) fn order_fingerprint(self) -> u64 {
        let family = match self.family {
            UiMountedPaintCommandFamily::FilledRect => 1_u64,
            UiMountedPaintCommandFamily::PortalOverlay => 2_u64,
            UiMountedPaintCommandFamily::SemanticText => 3_u64,
        };
        let mut digest = self
            .mounted_instance
            .diagnostic_value()
            .wrapping_mul(0x9e37_79b1_85eb_ca87)
            ^ family
            ^ u64::from(self.semantic_slot);
        if let Some(row) = self.collection_row {
            for chunk in row.chunks_exact(8) {
                digest ^= u64::from_le_bytes(chunk.try_into().expect("eight-byte chunk"));
                digest = digest.rotate_left(13).wrapping_mul(0xff51_afd7_ed55_8ccd);
            }
        }
        digest
    }
}

impl UiMountedPaintCommand {
    pub fn identity(&self) -> UiMountedPaintCommandIdentity {
        match self {
            Self::FilledRect { identity, .. }
            | Self::PortalOverlay { identity, .. }
            | Self::SemanticText { identity, .. } => *identity,
        }
    }

    pub fn layer_semantic_order(&self) -> u32 {
        match self {
            Self::FilledRect { mechanic, .. } => mechanic.layer_semantic_order(),
            Self::PortalOverlay { mechanic, .. } => mechanic.layer_semantic_order(),
            Self::SemanticText { mechanic, .. } => mechanic.layer_semantic_order(),
        }
    }

    pub fn bounds(&self) -> crate::UiMountedCanonicalBox {
        match self {
            Self::FilledRect { mechanic, .. } => mechanic.bounds(),
            Self::PortalOverlay { mechanic, .. } => mechanic.bounds(),
            Self::SemanticText { mechanic, .. } => mechanic.bounds(),
        }
    }

    pub fn clip_bounds(&self) -> crate::UiMountedCanonicalBox {
        match self {
            Self::FilledRect { mechanic, .. } => mechanic.clip_bounds(),
            Self::PortalOverlay { mechanic, .. } => mechanic.clip_bounds(),
            Self::SemanticText { mechanic, .. } => mechanic.clip_bounds(),
        }
    }

    pub fn semantic_digest(&self) -> u64 {
        match self {
            Self::FilledRect { mechanic, .. } => mechanic.semantic_digest(),
            Self::PortalOverlay { mechanic, .. } => mechanic.semantic_digest(),
            Self::SemanticText { mechanic, .. } => mechanic.semantic_digest(),
        }
    }
}

fn portal_identity_digest(identity: u64) -> [u8; 32] {
    let mut digest = [0_u8; 32];
    for (index, chunk) in digest.chunks_exact_mut(8).enumerate() {
        chunk.copy_from_slice(&identity.rotate_left((index * 13) as u32).to_le_bytes());
    }
    digest
}

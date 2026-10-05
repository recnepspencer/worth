use crate::BlobRecordKind;

/// The admission class of selected record bytes, independent of their mutable header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedRecordContentClass {
    UnknownLegacy,
    Opaque,
    Blob(BlobRecordKind),
    DerivedDirectory,
    BTreeNode { family_code: u16 },
}

/// Physical residency declared by the selected route. Primary is the only
/// currently allocatable class; other classes require tier-owned arenas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalTierClass {
    Primary,
    Hot,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedRecordRouteMetadata {
    content_class: SelectedRecordContentClass,
    tier_class: PhysicalTierClass,
}

impl SelectedRecordRouteMetadata {
    pub const fn new(
        content_class: SelectedRecordContentClass,
        tier_class: PhysicalTierClass,
    ) -> Option<Self> {
        if matches!(content_class, SelectedRecordContentClass::UnknownLegacy)
            && !matches!(tier_class, PhysicalTierClass::Primary)
        {
            return None;
        }
        if matches!(
            content_class,
            SelectedRecordContentClass::BTreeNode { family_code: 0 }
        ) {
            return None;
        }
        Some(Self {
            content_class,
            tier_class,
        })
    }

    pub const fn legacy_primary() -> Self {
        Self {
            content_class: SelectedRecordContentClass::UnknownLegacy,
            tier_class: PhysicalTierClass::Primary,
        }
    }

    pub const fn primary(content_class: SelectedRecordContentClass) -> Option<Self> {
        Self::new(content_class, PhysicalTierClass::Primary)
    }

    pub const fn content_class(self) -> SelectedRecordContentClass {
        self.content_class
    }

    pub const fn tier_class(self) -> PhysicalTierClass {
        self.tier_class
    }

    pub const fn is_legacy_unknown(self) -> bool {
        matches!(
            self.content_class,
            SelectedRecordContentClass::UnknownLegacy
        )
    }

    pub(crate) const fn encode(self) -> [u8; 7] {
        let mut encoded = [0; 7];
        match self.content_class {
            SelectedRecordContentClass::UnknownLegacy => {}
            SelectedRecordContentClass::Opaque => encoded[0] = 1,
            SelectedRecordContentClass::Blob(kind) => {
                encoded[0] = 2;
                encoded[1] = kind as u8;
            }
            SelectedRecordContentClass::DerivedDirectory => encoded[0] = 4,
            SelectedRecordContentClass::BTreeNode { family_code } => {
                encoded[0] = 3;
                let family = family_code.to_le_bytes();
                encoded[2] = family[0];
                encoded[3] = family[1];
            }
        }
        encoded[4] = match self.tier_class {
            PhysicalTierClass::Primary => 0,
            PhysicalTierClass::Hot => 1,
            PhysicalTierClass::Cold => 2,
        };
        encoded
    }

    pub(crate) fn decode(encoded: [u8; 7]) -> Option<Self> {
        if encoded[5..] != [0; 2] {
            return None;
        }
        let family = u16::from_le_bytes([encoded[2], encoded[3]]);
        let content_class = match (encoded[0], encoded[1], family) {
            (0, 0, 0) => SelectedRecordContentClass::UnknownLegacy,
            (1, 0, 0) => SelectedRecordContentClass::Opaque,
            (2, code, 0) => SelectedRecordContentClass::Blob(BlobRecordKind::from_code(code)?),
            (3, 0, family_code @ 1..) => SelectedRecordContentClass::BTreeNode { family_code },
            (4, 0, 0) => SelectedRecordContentClass::DerivedDirectory,
            _ => return None,
        };
        let tier_class = match encoded[4] {
            0 => PhysicalTierClass::Primary,
            1 => PhysicalTierClass::Hot,
            2 => PhysicalTierClass::Cold,
            _ => return None,
        };
        Self::new(content_class, tier_class)
    }
}

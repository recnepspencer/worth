#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCapability {
    Media,
    PageRecord,
    WalCheckpoint,
    Recovery,
    Maintenance,
    Layout,
    Blob,
}

impl PhysicalCapability {
    pub(crate) const FAMILY_COUNT: u64 = 7;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityAvailability {
    Absent,
    Present,
}

/// Immutable status derived from the runtime construction stage, not a caller claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstalledCapabilityStatus {
    serving: bool,
}

impl InstalledCapabilityStatus {
    pub(crate) const fn c3() -> Self {
        Self { serving: false }
    }

    pub(in crate::physical_runtime) const fn record_serving_with_layouts() -> Self {
        Self { serving: true }
    }

    pub const fn availability(self, capability: PhysicalCapability) -> CapabilityAvailability {
        match capability {
            PhysicalCapability::Media
            | PhysicalCapability::PageRecord
            | PhysicalCapability::WalCheckpoint
            | PhysicalCapability::Maintenance
            | PhysicalCapability::Layout
            | PhysicalCapability::Blob
                if self.serving =>
            {
                CapabilityAvailability::Present
            }
            PhysicalCapability::Media
            | PhysicalCapability::PageRecord
            | PhysicalCapability::WalCheckpoint
            | PhysicalCapability::Recovery
            | PhysicalCapability::Maintenance
            | PhysicalCapability::Layout
            | PhysicalCapability::Blob => CapabilityAvailability::Absent,
        }
    }

    pub const fn physical_media(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::Media)
    }

    pub const fn page_records(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::PageRecord)
    }

    pub const fn wal_and_checkpoint(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::WalCheckpoint)
    }

    pub const fn recovery(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::Recovery)
    }

    pub const fn maintenance(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::Maintenance)
    }

    pub const fn layout(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::Layout)
    }

    pub const fn blobs(self) -> CapabilityAvailability {
        self.availability(PhysicalCapability::Blob)
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilityAvailability, InstalledCapabilityStatus, PhysicalCapability};

    #[test]
    fn installed_status_requires_constructed_serving_owner() {
        let admitted = InstalledCapabilityStatus::c3();
        let serving = InstalledCapabilityStatus::record_serving_with_layouts();
        for family in [
            PhysicalCapability::Media,
            PhysicalCapability::PageRecord,
            PhysicalCapability::WalCheckpoint,
            PhysicalCapability::Maintenance,
            PhysicalCapability::Layout,
            PhysicalCapability::Blob,
        ] {
            assert_eq!(
                admitted.availability(family),
                CapabilityAvailability::Absent
            );
            assert_eq!(
                serving.availability(family),
                CapabilityAvailability::Present
            );
        }
        for unowned in [PhysicalCapability::Recovery] {
            assert_eq!(
                admitted.availability(unowned),
                CapabilityAvailability::Absent
            );
            assert_eq!(
                serving.availability(unowned),
                CapabilityAvailability::Absent
            );
        }
    }
}

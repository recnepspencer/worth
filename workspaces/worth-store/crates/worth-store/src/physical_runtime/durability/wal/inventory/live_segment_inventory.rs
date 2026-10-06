use worth_store_wal::{WalLsnRange, WalSegmentArtifactIdentity, WalSegmentInspection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability) struct PhysicalWalSegmentInventoryEntry {
    identity: WalSegmentArtifactIdentity,
    lsn_range: WalLsnRange,
    byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::wal) struct PhysicalWalSegmentInventory {
    pub(super) entries: Vec<PhysicalWalSegmentInventoryEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::wal) enum PhysicalWalSegmentInventoryUpdateDenial {
    ArtifactOrder,
    GenerationMismatch,
    LsnDiscontinuity,
    ByteCountOverflow,
}

impl PhysicalWalSegmentInventoryEntry {
    const fn from_inspection(inspection: WalSegmentInspection) -> Self {
        Self {
            identity: inspection.identity(),
            lsn_range: inspection.lsn_range(),
            byte_count: inspection.byte_count(),
        }
    }

    const fn from_completed_append(
        identity: WalSegmentArtifactIdentity,
        lsn_range: WalLsnRange,
        byte_count: u64,
    ) -> Self {
        Self {
            identity,
            lsn_range,
            byte_count,
        }
    }

    pub(in crate::physical_runtime::durability) const fn identity(
        self,
    ) -> WalSegmentArtifactIdentity {
        self.identity
    }

    pub(in crate::physical_runtime::durability) const fn lsn_range(self) -> WalLsnRange {
        self.lsn_range
    }

    pub(in crate::physical_runtime::durability) const fn byte_count(self) -> u64 {
        self.byte_count
    }
}

impl PhysicalWalSegmentInventory {
    pub(super) const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::durability::wal) const fn empty_for_runtime_test() -> Self {
        Self::empty()
    }

    pub(super) fn from_reopened(
        inspections: Vec<WalSegmentInspection>,
    ) -> Result<Self, PhysicalWalSegmentInventoryUpdateDenial> {
        let mut inventory = Self::empty();
        for inspection in inspections {
            inventory.push_reopened(PhysicalWalSegmentInventoryEntry::from_inspection(
                inspection,
            ))?;
        }
        Ok(inventory)
    }

    pub(in crate::physical_runtime::durability::wal) fn record_completed_append(
        &mut self,
        identity: WalSegmentArtifactIdentity,
        lsn_range: WalLsnRange,
        byte_count: u64,
    ) -> Result<(), PhysicalWalSegmentInventoryUpdateDenial> {
        let entry = PhysicalWalSegmentInventoryEntry::from_completed_append(
            identity, lsn_range, byte_count,
        );
        let Some(active) = self.entries.last_mut() else {
            self.entries.push(entry);
            return Ok(());
        };
        if active.identity == identity {
            if active.lsn_range.end_exclusive() != lsn_range.start() {
                return Err(PhysicalWalSegmentInventoryUpdateDenial::LsnDiscontinuity);
            }
            active.lsn_range =
                WalLsnRange::new(active.lsn_range.start(), lsn_range.end_exclusive())
                    .expect("two adjacent nonempty WAL ranges compose into one nonempty range");
            active.byte_count = active
                .byte_count
                .checked_add(byte_count)
                .ok_or(PhysicalWalSegmentInventoryUpdateDenial::ByteCountOverflow)?;
            return Ok(());
        }
        self.push_after_active(entry)
    }

    pub(in crate::physical_runtime::durability::wal) fn first_lsn_start(
        &self,
    ) -> Option<worth_store_wal::LogSequenceNumber> {
        self.entries.first().map(|entry| entry.lsn_range.start())
    }

    pub(in crate::physical_runtime::durability::wal) fn retains_canonical_wal_origin(
        &self,
    ) -> bool {
        let Some(first) = self.entries.first() else {
            return false;
        };
        worth_store_wal::WAL_ORIGIN.begins(first.identity, first.lsn_range.start())
    }

    pub(in crate::physical_runtime::durability::wal) fn entries(
        &self,
    ) -> &[PhysicalWalSegmentInventoryEntry] {
        &self.entries
    }

    pub(in crate::physical_runtime::durability::wal) fn consume_reclaimed_head(
        &mut self,
        expected: PhysicalWalSegmentInventoryEntry,
    ) -> Result<PhysicalWalSegmentInventoryEntry, PhysicalWalSegmentInventoryUpdateDenial> {
        if self.entries.first().copied() != Some(expected) {
            return Err(PhysicalWalSegmentInventoryUpdateDenial::ArtifactOrder);
        }
        Ok(self.entries.remove(0))
    }

    fn push_reopened(
        &mut self,
        entry: PhysicalWalSegmentInventoryEntry,
    ) -> Result<(), PhysicalWalSegmentInventoryUpdateDenial> {
        if let Some(active) = self.entries.last() {
            require_successor(*active, entry)?;
        }
        self.entries.push(entry);
        Ok(())
    }

    fn push_after_active(
        &mut self,
        entry: PhysicalWalSegmentInventoryEntry,
    ) -> Result<(), PhysicalWalSegmentInventoryUpdateDenial> {
        let active = *self
            .entries
            .last()
            .expect("a successor is appended only after an active segment");
        require_successor(active, entry)?;
        self.entries.push(entry);
        Ok(())
    }
}

fn require_successor(
    active: PhysicalWalSegmentInventoryEntry,
    successor: PhysicalWalSegmentInventoryEntry,
) -> Result<(), PhysicalWalSegmentInventoryUpdateDenial> {
    if successor.identity.generation() != active.identity.generation() {
        return Err(PhysicalWalSegmentInventoryUpdateDenial::GenerationMismatch);
    }
    if successor.identity <= active.identity {
        return Err(PhysicalWalSegmentInventoryUpdateDenial::ArtifactOrder);
    }
    if successor.lsn_range.start() != active.lsn_range.end_exclusive() {
        return Err(PhysicalWalSegmentInventoryUpdateDenial::LsnDiscontinuity);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

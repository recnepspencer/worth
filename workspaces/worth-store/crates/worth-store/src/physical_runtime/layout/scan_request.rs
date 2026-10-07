use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::BlobObjectId;

use super::PhysicalIndexPointKey;

const MAX_FOREGROUND_SCAN_PAGES: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIndexScanRequestDenial {
    ForeignFamily,
    ForeignStore,
    EmptyRange,
    ZeroPageBudget,
    PageBudgetTooWide,
}

/// A bounded request starting at a canonical family key. There is no
/// foreground full-scan constructor; the caller must name a lower bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIndexRange {
    family: DurableArtifactFamilyId,
    store: StableStoreIdentity,
    lower: [u8; 24],
    upper: Option<[u8; 24]>,
}

impl PhysicalIndexRange {
    pub fn from(start: PhysicalIndexPointKey) -> Self {
        Self {
            family: start.family(),
            store: start.store(),
            lower: start.canonical_bytes(),
            upper: None,
        }
    }

    pub fn between(
        start: PhysicalIndexPointKey,
        end_exclusive: PhysicalIndexPointKey,
    ) -> Result<Self, PhysicalIndexScanRequestDenial> {
        if start.family() != end_exclusive.family() {
            return Err(PhysicalIndexScanRequestDenial::ForeignFamily);
        }
        if start.store() != end_exclusive.store() {
            return Err(PhysicalIndexScanRequestDenial::ForeignStore);
        }
        let lower = start.canonical_bytes();
        let upper = end_exclusive.canonical_bytes();
        if lower >= upper {
            return Err(PhysicalIndexScanRequestDenial::EmptyRange);
        }
        Ok(Self {
            family: start.family(),
            store: start.store(),
            lower,
            upper: Some(upper),
        })
    }

    pub const fn family(self) -> DurableArtifactFamilyId {
        self.family
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn lower(self) -> [u8; 24] {
        self.lower
    }

    pub const fn upper(self) -> Option<[u8; 24]> {
        self.upper
    }
}

/// Blob-catalog prefix selects every generation of one Store-issued object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIndexPrefix {
    store: StableStoreIdentity,
    object: [u8; 16],
}

impl PhysicalIndexPrefix {
    pub fn blob_object(object: BlobObjectId) -> Self {
        Self {
            store: object.store(),
            object: object.bytes(),
        }
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn family(self) -> DurableArtifactFamilyId {
        DurableArtifactFamilyId::BlobCatalog
    }

    pub const fn object(self) -> [u8; 16] {
        self.object
    }

    pub(in crate::physical_runtime) fn bounds(self) -> ([u8; 24], Option<[u8; 24]>) {
        let mut lower = [0; 24];
        lower[..16].copy_from_slice(&self.object);
        let mut next = self.object;
        for byte in next.iter_mut().rev() {
            if *byte != u8::MAX {
                *byte += 1;
                let mut upper = [0; 24];
                upper[..16].copy_from_slice(&next);
                return (lower, Some(upper));
            }
            *byte = 0;
        }
        (lower, None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIndexScanBudget {
    maximum_pages: u64,
}

impl PhysicalIndexScanBudget {
    pub fn pages(maximum_pages: u64) -> Result<Self, PhysicalIndexScanRequestDenial> {
        if maximum_pages == 0 {
            return Err(PhysicalIndexScanRequestDenial::ZeroPageBudget);
        }
        if maximum_pages > MAX_FOREGROUND_SCAN_PAGES {
            return Err(PhysicalIndexScanRequestDenial::PageBudgetTooWide);
        }
        Ok(Self { maximum_pages })
    }

    pub const fn maximum_pages(self) -> u64 {
        self.maximum_pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_budget_is_explicit_and_bounded() {
        assert_eq!(
            PhysicalIndexScanBudget::pages(0),
            Err(PhysicalIndexScanRequestDenial::ZeroPageBudget)
        );
        assert!(PhysicalIndexScanBudget::pages(64).is_ok());
        assert_eq!(
            PhysicalIndexScanBudget::pages(65),
            Err(PhysicalIndexScanRequestDenial::PageBudgetTooWide)
        );
    }
}

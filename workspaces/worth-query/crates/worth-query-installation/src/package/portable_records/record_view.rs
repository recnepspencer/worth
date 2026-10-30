//! Installation-facing projections of retained portable operation meaning.

use super::{
    WorthQueryPortableApplicationOperationContractRecord, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily,
};

/// Borrowed canonical-position view over one exported logical record.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryPortablePackageRecordView<'a> {
    canonical_index: u32,
    record: &'a WorthQueryPortablePackageRecord,
}

impl<'a> WorthQueryPortablePackageRecordView<'a> {
    pub(crate) const fn new(
        canonical_index: u32,
        record: &'a WorthQueryPortablePackageRecord,
    ) -> Self {
        Self {
            canonical_index,
            record,
        }
    }

    pub const fn canonical_index(self) -> u32 {
        self.canonical_index
    }

    pub const fn family(self) -> WorthQueryPortablePackageRecordFamily {
        self.record.family()
    }

    pub const fn record(self) -> &'a WorthQueryPortablePackageRecord {
        self.record
    }
}

impl WorthQueryPortableApplicationOperationContractRecord {
    pub(crate) fn authored_program_width(&self) -> Option<usize> {
        self.touches().len().checked_add(self.emissions().len())
    }
}

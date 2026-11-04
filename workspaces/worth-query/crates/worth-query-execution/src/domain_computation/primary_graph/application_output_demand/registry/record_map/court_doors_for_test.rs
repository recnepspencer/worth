//! Sealed test doors over this module's private members.
use super::*;

impl DemandRecords {
    #[doc(hidden)]
    pub fn insert_for_test(
        &mut self,
        key: WorthQueryOutputDemandKey,
        row: DemandRecord,
    ) -> Option<DemandRecord> {
        Self::insert(self, key, row)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn values_for_test(&self) -> impl DoubleEndedIterator<Item = &DemandRecord> {
        Self::values(self)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

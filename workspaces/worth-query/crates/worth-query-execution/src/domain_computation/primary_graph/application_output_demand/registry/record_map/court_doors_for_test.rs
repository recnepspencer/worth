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

impl DemandRecords {
    #[doc(hidden)]
    pub fn contains_key_for_test(&self, key: &WorthQueryOutputDemandKey) -> bool {
        Self::contains_key(self, key)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn iter_for_test(
        &self,
    ) -> impl DoubleEndedIterator<Item = (&WorthQueryOutputDemandKey, &DemandRecord)> {
        Self::iter(self)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn source_occurrence_rows_for_test<'a>(
        &'a self,
        source: &super::super::SourceEpoch,
    ) -> impl Iterator<Item = (&'a WorthQueryOutputDemandKey, &'a DemandRecord)> + use<'a> {
        Self::source_occurrence_rows(self, source)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn values_mut_for_test(&mut self) -> impl DoubleEndedIterator<Item = RecordMutation<'_>> {
        Self::values_mut(self)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn extend_for_test(
        &mut self,
        rows: impl IntoIterator<Item = (WorthQueryOutputDemandKey, DemandRecord)>,
    ) {
        Self::extend(self, rows)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn get_for_test(&self, key: &WorthQueryOutputDemandKey) -> Option<&DemandRecord> {
        Self::get(self, key)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn is_empty_for_test(&self) -> bool {
        Self::is_empty(self)
    }
}

impl DemandRecords {
    #[doc(hidden)]
    pub fn remove_for_test(&mut self, key: &WorthQueryOutputDemandKey) -> Option<DemandRecord> {
        Self::remove(self, key)
    }
}

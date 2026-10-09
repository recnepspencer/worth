use std::sync::Arc;

use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DerivedFamilyRootDirectoryBinding,
    IndexedThroughBlobPublication, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    layout::AdmittedDirectoryRetirement, LifecycleGeneration, MaintenanceRetainedDirectoryCharge,
    RuntimeIdentity,
};

#[derive(Debug)]
struct ChargedDirectoryRetirement {
    records: Vec<PersistedRecordIdentity>,
    _charge: MaintenanceRetainedDirectoryCharge,
}

// The retirement owner's 256-byte retained control allowance must cover
// this allocation, including Arc's two reference-count words. Keep growth
// of the shared prepared owner visible to the compiler, not just a comment.
const _: () = assert!(
    std::mem::size_of::<ChargedDirectoryRetirement>() + 2 * std::mem::size_of::<usize>() <= 256
);

#[derive(Debug, Clone)]
enum DirectoryRetirementRecords {
    Empty,
    Charged(Arc<ChargedDirectoryRetirement>),
}

#[derive(Debug, Clone)]
pub(in crate::physical_runtime) struct PreparedDerivedDirectoryBasis {
    pub(in crate::physical_runtime) expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
    pub(in crate::physical_runtime) indexed_through: Option<IndexedThroughBlobPublication>,
    pub(in crate::physical_runtime) indexed_through_quarantine: Option<PersistedRecordIdentity>,
    replaced_nodes: DirectoryRetirementRecords,
}

impl PreparedDerivedDirectoryBasis {
    pub(in crate::physical_runtime) fn from_admitted(
        expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
        indexed_through: Option<IndexedThroughBlobPublication>,
        indexed_through_quarantine: Option<PersistedRecordIdentity>,
        admitted: AdmittedDirectoryRetirement<'_>,
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
    ) -> Option<Self> {
        let (records, charge) = admitted.into_charged_parts();
        let replaced_nodes = match charge {
            Some(charge) if charge.matches(store, runtime, generation) => {
                DirectoryRetirementRecords::Charged(Arc::new(ChargedDirectoryRetirement {
                    records,
                    _charge: charge,
                }))
            }
            None if records.is_empty() => DirectoryRetirementRecords::Empty,
            _ => return None,
        };
        Some(Self {
            expected_previous,
            indexed_through,
            indexed_through_quarantine,
            replaced_nodes,
        })
    }

    pub(in crate::physical_runtime) fn replaced_nodes(&self) -> &[PersistedRecordIdentity] {
        match &self.replaced_nodes {
            DirectoryRetirementRecords::Empty => &[],
            DirectoryRetirementRecords::Charged(retirement) => &retirement.records,
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn empty_for_validation(
        expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
        indexed_through: Option<IndexedThroughBlobPublication>,
        indexed_through_quarantine: Option<PersistedRecordIdentity>,
    ) -> Self {
        Self {
            expected_previous,
            indexed_through,
            indexed_through_quarantine,
            replaced_nodes: DirectoryRetirementRecords::Empty,
        }
    }
}

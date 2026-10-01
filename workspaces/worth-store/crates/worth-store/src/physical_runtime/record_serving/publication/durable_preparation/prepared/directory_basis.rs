use std::sync::Arc;

use worth_store_physical_format::{
    DerivedFamilyRootDirectoryBinding, IndexedThroughBlobPublication, PersistedRecordIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PreparedDerivedDirectoryBasis {
    pub(in crate::physical_runtime) expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
    pub(in crate::physical_runtime) indexed_through: Option<IndexedThroughBlobPublication>,
    pub(in crate::physical_runtime) indexed_through_quarantine: Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime) replaced_nodes: Arc<[PersistedRecordIdentity]>,
}

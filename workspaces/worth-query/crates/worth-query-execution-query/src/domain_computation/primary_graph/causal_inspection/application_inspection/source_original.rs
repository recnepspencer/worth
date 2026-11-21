//! A performed source shares only immutable canonical/disclosure custody.
use crate::domain_computation::primary_graph::{
    source_disclosure::OriginalSourceInspectionDisclosure, WorthQueryApplicationCommittedChanges,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SourceOriginal {
    pub runtime: u64,
    pub operation: [u8; 32],
    pub publication: crate::basis::WorthQueryProductBranchReadIdentity,
    pub changes: WorthQueryApplicationCommittedChanges,
    pub disclosure: Arc<OriginalSourceInspectionDisclosure>,
}

impl SourceOriginal {
    pub fn same_original(&self, other: &Self) -> bool {
        self.runtime == other.runtime
            && self.operation == other.operation
            && self.publication == other.publication
            && self.changes.shares_original(&other.changes)
            && Arc::ptr_eq(&self.disclosure, &other.disclosure)
    }
}

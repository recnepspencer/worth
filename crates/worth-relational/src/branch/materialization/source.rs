//! Original materialization source custody independent of the live proposal.
use super::RelationalMaterializationRecord;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use std::sync::Arc;

/// Minted from the real suspension read. Retains its actual basis and manifest;
/// neither this backing nor its identifiers grant an installed Source audience.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NativeMaterializationSource {
    basis: AdmittedRelationalBranchBasis,
    records: Arc<Vec<RelationalMaterializationRecord>>,
}

impl NativeMaterializationSource {
    pub(crate) fn from_suspension(
        basis: &AdmittedRelationalBranchBasis,
        records: Arc<Vec<RelationalMaterializationRecord>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            basis: basis.clone(),
            records,
        })
    }

    pub(crate) fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub(crate) fn matches_records(
        &self,
        records: &Arc<Vec<RelationalMaterializationRecord>>,
    ) -> bool {
        Arc::ptr_eq(&self.records, records)
    }

    pub(crate) fn records(&self) -> &[RelationalMaterializationRecord] {
        &self.records
    }
}

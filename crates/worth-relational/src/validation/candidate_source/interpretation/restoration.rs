//! Only the real materialization custody can supply a restoration origin.
use super::*;
use crate::branch::RelationalMaterializationCustody;
use crate::history::data::CommitId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeSourceRestorationOrigin {
    selected: AdmittedRelationalBranchBasis,
    source_commit: CommitId,
    suspension_commit: CommitId,
    source: Arc<NativeMaterializationSource>,
    records: Arc<Vec<RelationalMaterializationRecord>>,
}

impl NativeSourceRestorationOrigin {
    pub(crate) fn from_custody(
        selected: &AdmittedRelationalBranchBasis,
        custody: &RelationalMaterializationCustody,
    ) -> Self {
        Self {
            selected: selected.clone(),
            source_commit: custody.source_commit(),
            suspension_commit: custody.suspension_commit(),
            source: Arc::clone(&custody.data().source),
            records: Arc::clone(&custody.data().records),
        }
    }

    pub(super) fn admits(&self, owner: &NativeManagedSourceInterpretation) -> bool {
        owner.posture == Some(RelationalMaterializationTransactionMode::Rematerialize)
            && self.selected == owner.physical_basis
            && self.selected.observation().commit_id() == Some(self.suspension_commit)
            && self.source.basis().observation().commit_id() == Some(self.source_commit)
            && self.selected.identity() == self.source.basis().identity()
            && !self.selected.materialization_is_complete()
            && self.source.basis().materialization_is_complete()
            && Arc::ptr_eq(&self.records, &self.source.records)
            && owner
                .materialization
                .as_ref()
                .is_some_and(|source| Arc::ptr_eq(source, &self.source))
    }
}

impl NativeManagedSourceInterpretation {
    pub(in crate::validation::candidate_source) fn has_restoration_origin(&self) -> bool {
        self.restoration
            .as_ref()
            .is_some_and(|origin| origin.admits(self))
    }
}

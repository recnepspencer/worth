//! Exact runtime, branch and commit association for the admitted durable append.
pub(crate) struct CommitDurableAppendAdmission {
    runtime_instance_id: u64,
    commit_id: crate::history::data::CommitId,
    branch_id: crate::history::data::BranchId,
}

impl CommitDurableAppendAdmission {
    pub(crate) fn new(
        runtime_instance_id: u64,
        commit_id: crate::history::data::CommitId,
        branch_id: &crate::history::data::BranchId,
    ) -> Self {
        Self {
            runtime_instance_id,
            commit_id,
            branch_id: branch_id.clone(),
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        u64,
        crate::history::data::CommitId,
        crate::history::data::BranchId,
    ) {
        (self.runtime_instance_id, self.commit_id, self.branch_id)
    }
}

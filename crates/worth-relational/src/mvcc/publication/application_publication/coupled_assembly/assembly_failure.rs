//! A refused assembly retains the Native apply failure.
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

/// Retains Native cold-symbol discovery so the ordinary round can retry it.
#[derive(Debug)]
pub struct RelationalCoupledAssemblyFailure {
    pub(crate) cause: crate::authority::mutation::MutationApplyDenial,
    pub(super) stage_conflict: Option<(
        crate::transactions::data::primitives::RecordRef,
        usize,
        usize,
    )>,
}
impl From<TransactionCommitError> for RelationalCoupledAssemblyFailure {
    fn from(error: TransactionCommitError) -> Self {
        Self {
            cause: error.into(),
            stage_conflict: None,
        }
    }
}
impl RelationalCoupledAssemblyFailure {
    /// Exact conflicting record and physical authored-stage positions (prefix
    /// is zero). Descriptive evidence for Query's typed stage refusal only.
    pub fn authored_stage_conflict(
        &self,
    ) -> Option<(
        &crate::transactions::data::primitives::RecordRef,
        usize,
        usize,
    )> {
        self.stage_conflict
            .as_ref()
            .map(|(record, earlier, later)| (record, *earlier, *later))
    }
    pub(crate) fn from_apply(cause: crate::authority::mutation::MutationApplyDenial) -> Self {
        Self {
            cause,
            stage_conflict: None,
        }
    }
}

//! A refused assembly retains the Native apply failure.
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;

/// Retains Native cold-symbol discovery so the ordinary round can retry it.
#[derive(Debug)]
pub struct RelationalCoupledAssemblyFailure {
    pub(crate) cause: worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial,
    pub(super) stage_conflict: Option<(
        contracts::transactions::data::primitives::RecordRef,
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
        &contracts::transactions::data::primitives::RecordRef,
        usize,
        usize,
    )> {
        self.stage_conflict
            .as_ref()
            .map(|(record, earlier, later)| (record, *earlier, *later))
    }
    pub(crate) fn from_apply(
        cause: worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial,
    ) -> Self {
        Self {
            cause,
            stage_conflict: None,
        }
    }
}

#[doc(hidden)]
pub trait RelationalCoupledAssemblyFailureOwnerPort: Sized {
    fn from_apply(
        cause: worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial,
    ) -> Self;
}

impl RelationalCoupledAssemblyFailureOwnerPort for RelationalCoupledAssemblyFailure {
    fn from_apply(
        cause: worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial,
    ) -> Self {
        Self::from_apply(cause)
    }
}

#[doc(hidden)]
pub trait RelationalCoupledAssemblyFailureFieldsPort: Sized {
    fn into_cause(self) -> worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial;
}

impl RelationalCoupledAssemblyFailureFieldsPort for RelationalCoupledAssemblyFailure {
    fn into_cause(self) -> worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial{
        self.cause
    }
}

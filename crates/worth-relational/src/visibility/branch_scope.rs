//! The exact historical owner and configured genesis branch of the same runtime.
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::history::HistorySubsystem;

#[derive(Clone, Copy)]
pub(crate) struct HistoricalBasisInputs<'a> {
    pub(crate) history: &'a HistorySubsystem,
    pub(crate) genesis_branch: &'a BranchId,
}

pub(crate) fn branch_for_version<'a>(
    inputs: impl Into<HistoricalBasisInputs<'a>>,
    version_id: VersionId,
) -> Option<BranchId> {
    let inputs = inputs.into();
    inputs
        .history
        .immutable_commit_receipt_for_version(version_id)
        .map(|receipt| receipt.branch_id)
        .or_else(|| version_id.is_zero().then(|| inputs.genesis_branch.clone()))
}

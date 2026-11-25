//! The exact historical owner and configured genesis branch of the same runtime.
use crate::history::data::BranchId;
use crate::history::envelope_demand::CanonicalHistoryRead;
use crate::identity::data::VersionId;

#[derive(Clone, Copy)]
pub struct HistoricalBasisInputs<'a> {
    pub history: CanonicalHistoryRead<'a>,
    pub genesis_branch: &'a BranchId,
}

pub fn branch_for_version<'a>(
    inputs: impl Into<HistoricalBasisInputs<'a>>,
    version_id: VersionId,
) -> Option<BranchId> {
    let inputs = inputs.into();
    inputs
        .history
        .receipt_for_version(version_id)
        .map(|receipt| receipt.branch_id)
        .or_else(|| version_id.is_zero().then(|| inputs.genesis_branch.clone()))
}

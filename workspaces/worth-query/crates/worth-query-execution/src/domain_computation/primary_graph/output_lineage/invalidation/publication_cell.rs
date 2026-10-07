//! A writer can reserve an admitted cell or its own unpublished candidate only.
use super::source_alignment::BranchMarkRoot;
use worth_relational::facade::mvcc::{
    CompanionBranchCell, CompanionPreflightStop, PreparedCompanionBranchCell,
    PublicationCompanionPreflight, ReservedCompanionBranchCell,
};
pub(super) enum SelectedPublicationCell {
    Admitted(CompanionBranchCell<BranchMarkRoot>),
    Prepared(PreparedCompanionBranchCell<BranchMarkRoot>),
}
impl SelectedPublicationCell {
    pub(super) fn reserve_preflight(
        &self,
        context: &PublicationCompanionPreflight<'_>,
    ) -> Result<ReservedCompanionBranchCell<BranchMarkRoot>, CompanionPreflightStop> {
        match self {
            Self::Admitted(cell) => cell.reserve_preflight(context),
            Self::Prepared(cell) => cell.reserve_preflight(context),
        }
    }
}

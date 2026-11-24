//! Canonical membership for a selected producer publication.
use super::{
    CanonicalRequiredOutputOrigin, Deserialize, RelationalBranchBasisDescriptor, Serialize,
};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

mod absence;
mod header;
pub(super) mod install;
mod records;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalHistoricalOutputSelection {
    pub(crate) selected_basis: RelationalBranchBasisDescriptor,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) absences: Vec<absence::CanonicalHistoricalOutputAbsence>,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<Arc<ExecutionMemoryReservation>>,
}

impl CanonicalHistoricalOutputSelection {
    pub(crate) fn absence_decode_storage(capacity: usize) -> Option<u64> {
        u64::try_from(
            std::alloc::Layout::array::<absence::CanonicalHistoricalOutputAbsence>(capacity)
                .ok()?
                .size(),
        )
        .ok()
    }
}
impl PartialEq for CanonicalHistoricalOutputSelection {
    fn eq(&self, other: &Self) -> bool {
        self.selected_basis == other.selected_basis
            && self.publications == other.publications
            && self.absences == other.absences
    }
}
impl Eq for CanonicalHistoricalOutputSelection {}

//! Canonical membership for a selected producer publication.
use super::{
    CanonicalRequiredOutputOrigin, Deserialize, RelationalBranchBasisDescriptor, Serialize,
};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

pub mod absence;
pub mod header;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalHistoricalOutputSelection {
    pub selected_basis: RelationalBranchBasisDescriptor,
    pub publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absences: Vec<absence::CanonicalHistoricalOutputAbsence>,
    #[serde(skip)]
    pub _decoded_memory: Option<Arc<ExecutionMemoryReservation>>,
}

impl CanonicalHistoricalOutputSelection {
    pub fn absence_decode_storage(capacity: usize) -> Option<u64> {
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

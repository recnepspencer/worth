//! A concrete cell weak share retains its actual allocation charge, not data.
use super::RelationalRegionResidency;
use std::sync::{Arc, Weak};
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, Debug)]
pub(in crate::storage::residency) struct RegionCellWeak {
    raw: Weak<RelationalRegionResidency>,
    _control_memory: Option<Arc<ExecutionMemoryReservation>>,
}

impl RegionCellWeak {
    pub(in crate::storage::residency) fn upgrade(&self) -> Option<Arc<RelationalRegionResidency>> {
        self.raw.upgrade()
    }
    pub(in crate::storage::residency) fn as_ptr(&self) -> *const RelationalRegionResidency {
        self.raw.as_ptr()
    }
}

impl RelationalRegionResidency {
    pub(in crate::storage::residency) fn downgrade(cell: &Arc<Self>) -> RegionCellWeak {
        RegionCellWeak {
            raw: Arc::downgrade(cell),
            _control_memory: cell._control_memory.clone(),
        }
    }
}

//! A finite installed-query schedule. Each read still requires fresh authorization.
use std::sync::{Arc, Mutex};
mod planning;
pub(in crate::domain_computation::primary_graph) use planning::PlannedBatchItem;
pub use planning::WorthQueryApplicationQueryBatchReadPlan;

/// A scheduled read cannot be admitted to its exact installed batch plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBatchResourceDenial {
    ItemLimit { required: usize, maximum: usize },
    CounterOverflow,
    AllocationFailed,
    ForeignPlan,
    UnplannedRead,
    PlanningFrozen,
}

/// Exact scheduled slots issued from installed queries; not disclosure authority.
#[derive(Clone)]
pub struct WorthQueryApplicationQueryBatchAdmission {
    plan: Arc<Mutex<planning::InstalledBatchPlan>>,
}
impl WorthQueryApplicationQueryBatchAdmission {
    pub fn admit_items(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let maximum = self
            .plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .items;
        if count > maximum {
            Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: count,
                maximum,
            })
        } else {
            Ok(())
        }
    }
}

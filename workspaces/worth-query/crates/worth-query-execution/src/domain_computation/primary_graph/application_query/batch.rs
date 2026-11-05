//! A finite installed-query schedule. Each read still requires fresh authorization.
use std::sync::{Arc, Mutex};
pub(crate) mod planning;
pub(in crate::domain_computation::primary_graph) use planning::PlannedBatchItem;
pub use planning::WorthQueryApplicationQueryBatchReadPlan;

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
use crate::domain_computation::primary_graph::vocabulary::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial;
pub use planning::PlanningOps;

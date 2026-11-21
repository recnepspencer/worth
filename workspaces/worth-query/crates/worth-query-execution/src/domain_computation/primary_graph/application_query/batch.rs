//! A finite installed-query schedule. Each read still requires fresh authorization.
use std::sync::{Arc, Mutex};
pub(crate) mod planning;
pub(in crate::domain_computation::primary_graph) use planning::PlannedBatchItem;
pub use planning::WorthQueryApplicationQueryBatchReadPlan;

pub use worth_query_execution_lineage::facade::primary_graph::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial;

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

mod cause_aggregation;
mod dependency_admission;
mod revalidation;
pub(super) mod source_seed;
#[cfg(test)]
mod tests;

pub(crate) use cause_aggregation::{
    changed_scopes_for_edge, reconcile_edge_cause, CauseAdmissionContext,
};
pub(crate) use dependency_admission::{
    PreparedDirectCauseNodes, PreparedDirectCausePublication, PreparedDirectCauseStores,
    PreparedRetainedDirectCauseStores,
};

mod preparation_work;
mod scope_normalization;

pub(crate) use preparation_work::admit_delta_copy;

pub(crate) use scope_normalization::normalize as normalize_changed_scopes;

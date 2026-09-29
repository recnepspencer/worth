use crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile;

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Resolves optional declared caps against this runtime's finite work policy.
    pub fn resolve_application_query_limits(
        &self,
        limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits,
    ) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
        limits.resolve(
            self.runtime
                .application_query_resource_profile()
                .maximum_work(),
        )
    }
}

use super::super::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls,
};

pub(super) fn validate_work_limit<Schema>(
    plan: &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview,
    controls: &WorthQueryApplicationQueryControls<'_, Schema>,
    subject: &str,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    let intrinsic = plan.cost_estimate().intrinsic();
    let estimated_work = intrinsic
        .candidate_roots()
        .saturating_add(intrinsic.edge_touches())
        .saturating_add(intrinsic.intermediate_set_size());
    if estimated_work > controls.maximum_work().get() {
        return Err(WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
            subject,
        ));
    }
    Ok(())
}

pub(super) fn application_query_graph_read_budget<Schema>(
    profile: WorthQueryApplicationQueryResourceProfile,
    controls: &WorthQueryApplicationQueryControls<'_, Schema>,
) -> worth_query_admission::facade::graph_read_access::WorthQueryGraphReadBudget {
    profile.admission_budget(controls.maximum_result_count(), controls.maximum_work())
}

use super::request::SignalRuntimePolicyRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedSignalRuntimePolicy {
    request: SignalRuntimePolicyRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalRuntimePolicyAdmissionDenial {
    InvalidPolicy,
}

impl AdmittedSignalRuntimePolicy {
    pub(crate) fn admit(
        request: SignalRuntimePolicyRequest,
    ) -> Result<Self, SignalRuntimePolicyAdmissionDenial> {
        let policy = request.policy();
        if policy.maximum_waiter_resolution_visits == 0
            || policy.maximum_upstream_dependency_visits == 0
            || !policy.conditional_evaluation_budget.is_valid()
            || !policy.conditional_temporal_budget.is_valid()
            || policy.parallel_admission.throughput_min_parallel_tasks == 0
            || policy.parallel_admission.balanced_min_parallel_tasks == 0
            || policy.parallel_admission.latency_bounded_min_parallel_tasks == 0
            || policy.parallel_admission.full_parallel_min_tasks == 0
        {
            return Err(SignalRuntimePolicyAdmissionDenial::InvalidPolicy);
        }
        Ok(Self { request })
    }

    pub(crate) const fn request(&self) -> SignalRuntimePolicyRequest {
        self.request
    }
}

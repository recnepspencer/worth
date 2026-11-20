//! Bootstrap advancement phase wrapper carrying primary-graph capture policy admission.

use std::ops::Deref;

use worth_execution::ExecutionRequest;
use worth_query_execution_runtime::facade::request_execution as rt_req;

use super::WorthQueryAdvancementDenial;

#[repr(transparent)]
pub struct WorthQueryBootstrapAdvancementPhase<'scope>(
    pub(crate) rt_req::WorthQueryBootstrapAdvancementPhase<'scope>,
);

impl<'scope> Deref for WorthQueryBootstrapAdvancementPhase<'scope> {
    type Target = rt_req::WorthQueryBootstrapAdvancementPhase<'scope>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WorthQueryBootstrapAdvancementPhase<'_> {
    pub(in crate::domain_computation::primary_graph) fn funded_capture_policy(
        &self,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'_, '_>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::primary_graph::{
            WorthQueryPrimaryGraphInstallationDenial as D,
            WorthQueryPrimaryGraphInstallationDenialKind as K,
        };
        let request = self.0.execution_request();
        if !request.is_leased() {
            return Err(D::new(
                K::ExecutionDenied {
                    kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::ResourceEnvelopeMismatch,
                },
                "original-request checkpoint capture requires leased backing; serial system allocation is unsupported",
            ));
        }
        Ok(request.allocation_policy())
    }

    pub(in crate::domain_computation::primary_graph) fn admit_capture_policy<
        'policy,
        'authority,
    >(
        &self,
        capture: worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::primary_graph::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryPrimaryGraphInstallationDenial as D,
            WorthQueryPrimaryGraphInstallationDenialKind as K,
        };
        use worth_execution::ExecutionAllocationPolicy as Policy;
        let funding = self.funded_capture_policy()?;
        match (funding, capture) {
            (Policy::Execution(funding), Policy::Execution(lease))
                if lease.descends_from(funding) =>
            {
                Ok(capture)
            }
            (Policy::Execution(_), Policy::Execution(_)) => Err(D::new(
                K::ExecutionDenied {
                    kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: Resource::NestedLeaseMisuse,
                        partition_identity: None,
                        policy_ancestor: None,
                    },
                },
                "checkpoint capture requires a lease descending from the transition's funding lease",
            )),
            (Policy::Execution(_) | Policy::SystemAllocation, Policy::SystemAllocation)
            | (Policy::SystemAllocation, Policy::Execution(_)) => Err(D::new(
                K::ExecutionDenied {
                    kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::ResourceEnvelopeMismatch,
                },
                "checkpoint capture requires leased backing; system allocation is unsupported",
            )),
        }
    }
}

pub fn with_bootstrap_advancement<R>(
    policy: worth_foundational::ExecutionRequestPolicy,
    body: impl for<'scope> FnOnce(WorthQueryBootstrapAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    rt_req::with_bootstrap_advancement(policy, |phase| {
        body(WorthQueryBootstrapAdvancementPhase(phase))
    })
    .map_err(Into::into)
}

pub fn with_bootstrap_advancement_from_request<R>(
    request: ExecutionRequest<'_, '_>,
    body: impl for<'scope> FnOnce(WorthQueryBootstrapAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    rt_req::with_bootstrap_advancement_from_request(request, |phase| {
        body(WorthQueryBootstrapAdvancementPhase(phase))
    })
    .map_err(Into::into)
}

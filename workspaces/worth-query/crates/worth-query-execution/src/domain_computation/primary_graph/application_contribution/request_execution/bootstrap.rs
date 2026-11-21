//! Primary-graph capture policy admission extension on `WorthQueryBootstrapAdvancementPhase`.

use worth_query_execution_runtime::facade::request_execution::WorthQueryBootstrapAdvancementPhase;

pub(in crate::domain_computation::primary_graph) trait BootstrapCapturePolicyExt {
    fn funded_capture_policy(
        &self,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'_, '_>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >;

    fn admit_capture_policy<'policy, 'authority>(
        &self,
        capture: worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >;
}

impl BootstrapCapturePolicyExt for WorthQueryBootstrapAdvancementPhase<'_> {
    fn funded_capture_policy(
        &self,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'_, '_>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::primary_graph::{
            WorthQueryPrimaryGraphInstallationDenial as D,
            WorthQueryPrimaryGraphInstallationDenialKind as K,
        };
        let request = self.execution_request();
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

    fn admit_capture_policy<'policy, 'authority>(
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

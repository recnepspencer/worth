use super::*;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind;

pub(in crate::domain_computation::authorization) trait InstalledBasisScopedOps<Schema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn installed_capability_plan(
        &self,
        request: &WorthQueryRetainedCapabilityRequest,
    ) -> Result<
        &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.authorization()
            .capability_plan_by_identity(&request.capability_identity())
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::PolicyNotInstalled,
                    ),
                    "retained-capability-request",
                )
            })
    }

    fn sample_capability_time(
        &self,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.authorization_clock()
            .sample(installed.request().timeline)
            .map_err(|_| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::TrustedTimeUnavailable,
                    ),
                    installed.contract().name(),
                )
            })
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > InstalledBasisScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

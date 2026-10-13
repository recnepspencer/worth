use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::authorization) fn installed_capability_plan(
        &self,
        request: &WorthQueryRetainedCapabilityRequest,
    ) -> Result<
        &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.authorization
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

    pub(in crate::domain_computation::authorization) fn sample_capability_time(
        &self,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.authorization_clock
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

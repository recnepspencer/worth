//! Fresh observation of the exact governed support carried by a request receipt.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRuntimeTimeSample,
};
use super::super::WorthQueryElevationRequestBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryRequestedElevation,
};

pub(super) fn refresh_exact_support<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    requested: &mut WorthQueryRequestedElevation,
    access: &mut WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        Capability,
        Operation,
        Input,
    >,
) -> Result<WorthQueryRuntimeTimeSample, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let installed =
        runtime.installed_capability_plan(requested.binding().supporting().request())?;
    validate_support_owner(requested.binding(), installed)?;
    let current = observe_current_support(runtime, requested.binding(), access, installed)?;
    requested.apply_current_support(current, access, installed.contract().name())
}

fn observe_current_support<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    requested: &WorthQueryElevationRequestBinding,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Result<super::super::WorthQueryCurrentElevationSupport, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    access
        .with_exact_observation(runtime, |observation| {
            observation.observe_current_elevation_support(installed, requested.supporting())
        })?
        .ok_or_else(|| stale_support(installed.contract().name()))?
        .map_err(|denial| support_observation_denial(installed.contract().name(), denial))
}

fn validate_support_owner(
    requested: &WorthQueryElevationRequestBinding,
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if requested.capability_identity() == requested.supporting().request().capability_identity()
        && requested.capability_authority_identity()
            == requested.supporting().capability_authority_identity()
        && requested.supporting().capability_authority_identity()
            == installed.capability_authority_identity().as_ref()
        && requested.grant() == requested.supporting().grant()
    {
        Ok(())
    } else {
        Err(inconsistent_support(installed.contract().name()))
    }
}

fn support_observation_denial(
    subject: &str,
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryOperationAuthorizationDenial {
    match denial.kind() {
        WorthQueryOperationAuthorizationDenialKind::PermissionDenied
        | WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected => {
            stale_support(subject)
        }
        _ => denial,
    }
}

fn stale_support(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
        subject,
    )
}

fn inconsistent_support(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}

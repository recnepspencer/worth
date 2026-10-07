use std::sync::Arc;

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationDefinition, ApplicationCapabilityElevationRequest,
    ApplicationCapabilityElevationRequestProjection, ApplicationCapabilityRequest,
};
use worth_query_installation::facade::{ApplicationOperationProgramTarget, ApplicationSchema};

use super::super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use super::super::request::{denial, projection_denial};
use super::WorthQueryElevationRequestBinding;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation::authorization::elevation_progression) fn bind_request<
    Schema,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    capability_identity: [u8; 32],
    installed: &WorthQueryInstalledCapabilityPlan,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    proposed: &ApplicationCapabilityElevationRequestProjection<
        Schema,
        <Input as ApplicationCapabilityElevationRequest<Schema, Operation>>::Scope,
        <Input as ApplicationCapabilityElevationRequest<Schema, Operation>>::Context,
    >,
) -> Result<WorthQueryElevationRequestBinding, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>
        + ApplicationCapabilityElevationRequest<Schema, Operation>
        + 'static,
{
    let elevation = installed
        .elevation()
        .as_ref()
        .ok_or_else(|| projection_denial(installed.contract().name()))?;
    let interval = runtime
        .authorization_clock
        .sample_interval(elevation.temporal.timeline, proposed.duration())
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable,
                installed.contract().name(),
            )
        })?;
    let observed = access
        .with_exact_observation(runtime, |observation| {
            observation.resolve_elevation_upper_bound(
                capability_identity,
                installed,
                proposed,
                &interval.issued,
            )
        })?
        .ok_or_else(|| projection_denial(access.operation()))??;
    let lifecycle = &elevation.lifecycle;
    Ok(WorthQueryElevationRequestBinding {
        runtime_authority: runtime.runtime.authority_identity(),
        branch: access.graph_work_branch().clone(),
        capability_identity,
        capability_authority_identity: Arc::clone(installed.capability_authority_identity()),
        observed: observed.retained_for_operation(),
        elevation_kind: elevation.elevation_kind,
        review_kind: lifecycle.review_kind,
        elevation_key: proposed.elevation_key().to_string(),
        elevation_identity_field: lifecycle.identity.clone(),
        elevation_identity: proposed.elevation_identity().value().clone(),
        reason_field: lifecycle.reason.clone(),
        reason: proposed.reason().value().clone(),
        status_field: lifecycle.status.clone(),
        requested_status: lifecycle.requested.clone(),
        not_before_field: elevation.temporal.not_before.clone(),
        issued_at: interval.issued.value().clone(),
        not_after_field: elevation.temporal.not_after.clone(),
        expires_at: interval.expires,
        review_key: proposed.review_key().to_string(),
        review_identity_field: lifecycle.review_identity.clone(),
        review_identity: proposed.review_identity().value().clone(),
        review_type_field: lifecycle.review_type.clone(),
        review_type: lifecycle.review_type_value.clone(),
        review_status_field: lifecycle.review_status.clone(),
        review_required_status: lifecycle.review_required.clone(),
        requester_relation: lifecycle.requester_relation,
        grant_relation: lifecycle.grant_relation,
        resource_relation: lifecycle.resource_relation,
        review_relation: lifecycle.review_relation,
        review_scope_relation: lifecycle.review_scope_relation,
        required_program_targets: required_program_targets(
            installed
                .contract()
                .elevation()
                .definition()
                .expect("installed elevation plan retains its governed declaration"),
        ),
        lifecycle_effect: super::super::lifecycle_effect::derive_lifecycle_effect(
            installed
                .contract()
                .elevation()
                .definition()
                .unwrap()
                .lifecycle()
                .request(),
            access.capability_input(),
            installed.contract().name(),
        )?,
    })
}

fn required_program_targets(
    elevation: &ApplicationCapabilityElevationDefinition,
) -> Vec<ApplicationOperationProgramTarget> {
    let review = elevation.review();
    let mut targets = vec![
        ApplicationOperationProgramTarget::Create {
            entity: elevation.identity().entity().to_string(),
        },
        ApplicationOperationProgramTarget::Create {
            entity: review.identity().entity().to_string(),
        },
    ];
    targets.extend(
        [
            elevation.identity(),
            elevation.reason(),
            elevation.status(),
            elevation.validity().not_before(),
            elevation.validity().not_after(),
            review.identity(),
            review.kind().field(),
            review.status(),
        ]
        .into_iter()
        .map(|field| ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        }),
    );
    targets.extend(
        [
            elevation.requester(),
            elevation.grant(),
            review.relation(),
            review.scope(),
        ]
        .into_iter()
        .map(|relation| ApplicationOperationProgramTarget::Link {
            relation: relation.relation().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        }),
    );
    targets.extend(elevation.resource_relation().map(|relation| {
        ApplicationOperationProgramTarget::Link {
            relation: relation.relation().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        }
    }));
    targets.extend(
        elevation
            .lifecycle()
            .request()
            .lifecycle_effect()
            .map(|effect| ApplicationOperationProgramTarget::Emit {
                effect: effect.effect().to_string(),
            }),
    );
    targets
}

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
    WorthQueryInstalledApplicationOperation,
};
use worth_relational::facade::identity::EntityId;

use super::super::capability_registry::{
    WorthQueryElevationLifecycleOperationRole, WorthQueryInstalledCapabilityPlan,
};
use super::super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use super::context_identity::selected_elevation_entity;
use super::operation_role::installed_lifecycle_owner;
use super::transition_contract::{close_program_targets, lifecycle_decision_reads};
use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod binding;
pub(in crate::domain_computation) use binding::WorthQueryElevationCloseBinding;

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryElevationCloseDraft {
    elevation: EntityId,
    review: EntityId,
    closer: EntityId,
    closure_kind: WorthQueryElevationClosureKind,
    closed_at: AspectValue,
    closed_status: AspectValue,
    approved_status: AspectValue,
    elevation_entity: String,
    status_field: AspectFieldLocator,
    approver_relation: worth_relational::facade::identity::KindId,
    reviewer_relation: worth_relational::facade::identity::KindId,
    required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    required_program_targets: Vec<ApplicationOperationProgramTarget>,
    lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

#[derive(Debug)]
pub struct WorthQueryElevationCloseAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    approved: WorthQueryApprovedElevation,
}

impl WorthQueryElevationCloseAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_approved(self) -> WorthQueryApprovedElevation {
        self.approved
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn authorize_elevation_close<Capability, Operation, Input>(
        &self,
        approved: WorthQueryApprovedElevation,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryElevationCloseAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        let draft = match bind_close(self, &approved, &access, operation) {
            Ok(draft) => draft,
            Err(denial) => return Err(close_denial(approved, denial)),
        };
        let admission = match progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        ) {
            Ok(admission) => admission,
            Err(denial) => return Err(close_denial(approved, denial)),
        };
        admission
            .bind_elevation_close(draft.bind(approved))
            .map_err(|(denial, binding)| close_denial(binding.into_approved(), denial))
    }
}

fn bind_close<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    approved: &WorthQueryApprovedElevation,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<WorthQueryElevationCloseDraft, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
    Input: 'static,
{
    let (capability_identity, installed) = installed_lifecycle_owner(
        runtime,
        access.installed_capability_identity(),
        operation,
        WorthQueryElevationLifecycleOperationRole::Revoke,
    )?;
    if !approved.belongs_to_lifecycle(
        runtime.runtime.authority_identity(),
        access.graph_work_branch(),
        capability_identity,
        installed.capability_authority_identity().as_ref(),
    ) {
        return Err(close_rejected(installed.contract().name()));
    }
    let elevation = selected_elevation_entity(access, installed)
        .ok_or_else(|| close_rejected(installed.contract().name()))?;
    if elevation != approved.elevation() {
        return Err(close_rejected(installed.contract().name()));
    }
    let review = approved.review();
    let lifecycle = installed.elevation().as_ref().unwrap();
    let (closure_kind, closed_at, closed_status) = derive_closure(runtime, installed, approved)?;
    let definition = installed.contract().elevation().definition().unwrap();
    Ok(WorthQueryElevationCloseDraft {
        elevation,
        review,
        closer: access.principal_entity_id(),
        closure_kind,
        closed_at,
        closed_status,
        approved_status: lifecycle.lifecycle.approved.clone(),
        elevation_entity: definition.status().entity().to_string(),
        status_field: lifecycle.lifecycle.status.clone(),
        approver_relation: lifecycle.lifecycle.approver_relation,
        reviewer_relation: lifecycle.lifecycle.reviewer_relation,
        required_decision_reads: lifecycle_decision_reads(installed),
        required_program_targets: close_program_targets(installed),
        lifecycle_effect: super::lifecycle_effect::derive_lifecycle_effect(
            definition.lifecycle().revoke(),
            access.capability_input(),
            installed.contract().name(),
        )?,
    })
}

fn derive_closure<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledCapabilityPlan,
    approved: &WorthQueryApprovedElevation,
) -> Result<
    (WorthQueryElevationClosureKind, AspectValue, AspectValue),
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
{
    let lifecycle = installed.elevation().as_ref().unwrap();
    let sample = runtime
        .authorization_clock
        .sample(lifecycle.temporal.timeline)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable,
                installed.contract().name(),
            )
        })?;
    let (closure_kind, closed_status) = match (sample.value(), approved.expires_at()) {
        (AspectValue::UInt64(now), AspectValue::UInt64(expires)) if now >= expires => (
            WorthQueryElevationClosureKind::Expired,
            lifecycle.lifecycle.expired.clone(),
        ),
        (AspectValue::UInt64(_), AspectValue::UInt64(_)) => (
            WorthQueryElevationClosureKind::Revoked,
            lifecycle.lifecycle.revoked.clone(),
        ),
        _ => return Err(close_rejected(installed.contract().name())),
    };
    Ok((closure_kind, sample.value().clone(), closed_status))
}

fn close_denial(
    approved: WorthQueryApprovedElevation,
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryElevationCloseAuthorizationDenial {
    WorthQueryElevationCloseAuthorizationDenial { denial, approved }
}

fn close_rejected(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::ElevationCloseRejected,
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}

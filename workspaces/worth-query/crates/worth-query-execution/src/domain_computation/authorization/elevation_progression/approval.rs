use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
mod replay;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
    WorthQueryInstalledApplicationOperation,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::super::capability_registry::{
    WorthQueryElevationLifecycleOperationRole, WorthQueryInstalledCapabilityPlan,
};
use super::super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::context_identity::{
    resolve_elevation_identity, resolve_review_identity, selected_elevation_entity,
};
use super::operation_role::installed_lifecycle_owner;
use super::transition_contract::{approval_program_targets, lifecycle_decision_reads};
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding, crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial, crate::domain_computation::primary_graph::WorthQueryElevationApprovalOutcome, crate::domain_computation::primary_graph::WorthQueryRequestedElevation};
use {
    super::super::WorthQueryAdmittedApplicationCapabilityAccess,
    super::super::WorthQueryAdmittedApplicationOperation,
    super::super::WorthQueryOperationAuthorizationDenial,
    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
};

mod support_currentness;
use support_currentness::refresh_exact_support;
mod binding;
pub(in crate::domain_computation) use binding::WorthQueryElevationApprovalBinding;

pub(in crate::domain_computation) struct WorthQueryElevationApprovalBindingPermit(());

impl WorthQueryElevationApprovalBindingPermit {
    fn mint() -> Self {
        Self(())
    }
}

pub(in crate::domain_computation) struct WorthQueryElevationApprovalDraft {
    elevation: EntityId,
    review: EntityId,
    approver: EntityId,
    approved_status: AspectValue,
    elevation_entity: String,
    status_field: AspectFieldLocator,
    approver_relation: KindId,
    reviewer_relation: KindId,
    required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    required_program_targets: Vec<ApplicationOperationProgramTarget>,
    lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

/// Refusal to authorize approving a requested elevation.
///
/// Nothing was approved. [`Self::denial`] says why, and
/// [`Self::into_requested`] returns the requested elevation so the caller can
/// try again or replay a recorded approval.
#[derive(Debug)]
pub struct WorthQueryElevationApprovalAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    requested: WorthQueryRequestedElevation,
}

impl WorthQueryElevationApprovalAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_requested(self) -> WorthQueryRequestedElevation {
        self.requested
    }
}

pub trait ApprovalOps<Schema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn authorize_elevation_approval<Capability, Operation, Input>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        requested: WorthQueryRequestedElevation,
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
        WorthQueryElevationApprovalAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        let execution = match phase.execution_request_for(self.product_runtime()) {
            Ok(execution) => execution,
            Err(_) => return Err(approval_denial(requested, crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody"))),
        };
        self.authorize_elevation_approval_with_request(
            &execution,
            requested,
            access,
            operation,
            preconditions,
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > ApprovalOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

pub(in crate::domain_computation) trait ApprovalScopedOps<Schema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn authorize_elevation_approval_with_request<Capability, Operation, Input>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut requested: WorthQueryRequestedElevation,
        mut access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
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
        WorthQueryElevationApprovalAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        let draft = match bind_approval(
            execution,
            self.authorization_host(),
            &requested,
            &access,
            operation,
        ) {
            Ok(draft) => draft,
            Err(denial) => return Err(approval_denial(requested, denial)),
        };
        let support_sample = match refresh_exact_support(
            execution,
            self.authorization_host(),
            &mut requested,
            &mut access,
        ) {
            Ok(sample) => sample,
            Err(denial) => return Err(approval_denial(requested, denial)),
        };
        if let Err(denial) =
            validate_approval_time(requested.binding(), &support_sample, operation.operation())
        {
            return Err(approval_denial(requested, denial));
        }
        let admission = match progress_capability_operation(
            execution,
            self.authorization_host(),
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        ) {
            Ok(admission) => admission,
            Err(denial) => return Err(approval_denial(requested, denial)),
        };
        admission
            .bind_elevation_approval(draft.bind(requested))
            .map_err(|(denial, binding)| approval_denial(binding.into_requested(), denial))
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > ApprovalScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

fn bind_approval<Schema, Capability, Operation, Input>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    requested: &WorthQueryRequestedElevation,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<WorthQueryElevationApprovalDraft, WorthQueryOperationAuthorizationDenial>
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
        WorthQueryElevationLifecycleOperationRole::Approve,
    )?;
    let requested_binding = requested.binding();
    validate_receipt_authority(runtime, capability_identity, installed, requested, access)?;
    let lifecycle = installed
        .elevation()
        .as_ref()
        .ok_or_else(|| approval_rejected(installed.contract().name()))?;
    let elevation_definition = installed
        .contract()
        .elevation()
        .definition()
        .ok_or_else(|| approval_rejected(installed.contract().name()))?;
    let (elevation, review) =
        resolve_approval_entities(execution, runtime, access, installed, requested_binding)?;
    Ok(WorthQueryElevationApprovalDraft {
        elevation,
        review,
        approver: access.principal_entity_id(),
        approved_status: lifecycle.lifecycle.approved.clone(),
        elevation_entity: elevation_definition.status().entity().to_string(),
        status_field: lifecycle.lifecycle.status.clone(),
        approver_relation: lifecycle.lifecycle.approver_relation,
        reviewer_relation: lifecycle.lifecycle.reviewer_relation,
        required_decision_reads: lifecycle_decision_reads(installed),
        required_program_targets: approval_program_targets(installed),
        lifecycle_effect: super::lifecycle_effect::derive_lifecycle_effect(
            elevation_definition.lifecycle().approve(),
            access.capability_input(),
            installed.contract().name(),
        )?,
    })
}

fn resolve_approval_entities<Schema, Capability, Operation, Input>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    installed: &WorthQueryInstalledCapabilityPlan,
    requested: &super::WorthQueryElevationRequestBinding,
) -> Result<(EntityId, EntityId), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let elevation = selected_elevation_entity(access, installed)
        .ok_or_else(|| approval_rejected(installed.contract().name()))?;
    let resolved_elevation = resolve_elevation_identity(
        execution,
        runtime,
        access,
        requested.capability_identity(),
        installed,
        requested.elevation_identity(),
    )
    .ok_or_else(|| approval_rejected(installed.contract().name()))?;
    let review = resolve_review_identity(
        execution,
        runtime,
        access,
        requested.capability_identity(),
        installed,
        requested.review_identity(),
    )
    .ok_or_else(|| approval_rejected(installed.contract().name()))?;
    if resolved_elevation == elevation {
        Ok((elevation, review))
    } else {
        Err(approval_rejected(installed.contract().name()))
    }
}

fn validate_receipt_authority<Schema, Capability, Operation, Input>(
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    capability_identity: [u8; 32],
    installed: &WorthQueryInstalledCapabilityPlan,
    requested: &WorthQueryRequestedElevation,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let binding = requested.binding();
    if binding.runtime_authority() != &runtime.authority_identity()
        || binding.branch() != access.graph_work_branch()
        || requested.commit_receipt().terminal().branch() != binding.branch()
        || binding.capability_identity() != capability_identity
        || binding.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
    {
        return Err(approval_rejected(installed.contract().name()));
    }
    Ok(())
}

fn validate_approval_time(
    requested: &super::WorthQueryElevationRequestBinding,
    sample: &super::super::WorthQueryRuntimeTimeSample,
    subject: &str,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    match (
        sample.value(),
        requested.issued_at(),
        requested.expires_at(),
    ) {
        (AspectValue::UInt64(now), AspectValue::UInt64(start), AspectValue::UInt64(end))
            if start <= now && now < end =>
        {
            Ok(())
        }
        _ => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationExpired,
            ),
            subject,
        )),
    }
}

fn approval_denial(
    requested: WorthQueryRequestedElevation,
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryElevationApprovalAuthorizationDenial {
    WorthQueryElevationApprovalAuthorizationDenial { denial, requested }
}

fn approval_rejected(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApprovalRejected,
        ),
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
pub use replay::ApprovalReplayOps;
pub(in crate::domain_computation) use replay::ApprovalReplayScopedOps;

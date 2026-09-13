use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
    WorthQueryInstalledApplicationOperation,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::super::capability_registry::WorthQueryElevationLifecycleOperationRole;
use super::super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use super::context_identity::{selected_elevation_entity, selected_review_entity};
use super::operation_role::installed_lifecycle_owner;
use super::transition_contract::{lifecycle_decision_reads, review_program_targets};
use crate::domain_computation::primary_graph::{
    WorthQueryElevationClosureKind, WorthQueryMandatoryReview,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod binding;
pub(in crate::domain_computation) use binding::WorthQueryMandatoryReviewBinding;

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryMandatoryReviewDraft {
    elevation: EntityId,
    review: EntityId,
    reviewer: EntityId,
    reviewed_at: AspectValue,
    terminal_status: AspectValue,
    completed_status: AspectValue,
    review_entity: String,
    review_status_field: AspectFieldLocator,
    approver_relation: KindId,
    reviewer_relation: KindId,
    required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    required_program_targets: Vec<ApplicationOperationProgramTarget>,
    lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

#[derive(Debug)]
pub struct WorthQueryMandatoryReviewAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    mandatory: WorthQueryMandatoryReview,
}

impl WorthQueryMandatoryReviewAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_mandatory_review(self) -> WorthQueryMandatoryReview {
        self.mandatory
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn authorize_mandatory_review<Capability, Operation, Input>(
        &self,
        mandatory: WorthQueryMandatoryReview,
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
        WorthQueryMandatoryReviewAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>,
        Input: 'static,
    {
        let draft = match bind_review(self, &mandatory, &access, operation) {
            Ok(draft) => draft,
            Err(denial) => return Err(review_denial(mandatory, denial)),
        };
        let admission = match progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        ) {
            Ok(admission) => admission,
            Err(denial) => return Err(review_denial(mandatory, denial)),
        };
        admission
            .bind_mandatory_review(draft.bind(mandatory))
            .map_err(|(denial, binding)| review_denial(binding.into_mandatory(), denial))
    }
}

fn bind_review<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mandatory: &WorthQueryMandatoryReview,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<WorthQueryMandatoryReviewDraft, WorthQueryOperationAuthorizationDenial>
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
        WorthQueryElevationLifecycleOperationRole::CompleteReview,
    )?;
    if !mandatory.belongs_to_lifecycle(
        runtime.runtime.authority_identity(),
        access.graph_work_branch(),
        capability_identity,
        installed.capability_authority_identity().as_ref(),
    ) {
        return Err(review_rejected(installed.contract().name()));
    }
    let elevation = selected_elevation_entity(access, installed)
        .ok_or_else(|| review_rejected(installed.contract().name()))?;
    if elevation != mandatory.elevation() {
        return Err(review_rejected(installed.contract().name()));
    }
    let review = mandatory.review();
    if selected_review_entity(access, installed) != Some(review) {
        return Err(review_rejected(installed.contract().name()));
    }
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
    let terminal_status = match mandatory.closure_kind() {
        WorthQueryElevationClosureKind::Revoked => lifecycle.lifecycle.revoked.clone(),
        WorthQueryElevationClosureKind::Expired => lifecycle.lifecycle.expired.clone(),
    };
    let definition = installed.contract().elevation().definition().unwrap();
    Ok(WorthQueryMandatoryReviewDraft {
        elevation,
        review,
        reviewer: access.principal_entity_id(),
        reviewed_at: sample.value().clone(),
        terminal_status,
        completed_status: lifecycle.lifecycle.review_completed.clone(),
        review_entity: definition.review().status().entity().to_string(),
        review_status_field: lifecycle.lifecycle.review_status.clone(),
        approver_relation: lifecycle.lifecycle.approver_relation,
        reviewer_relation: lifecycle.lifecycle.reviewer_relation,
        required_decision_reads: lifecycle_decision_reads(installed),
        required_program_targets: review_program_targets(installed),
        lifecycle_effect: super::lifecycle_effect::derive_lifecycle_effect(
            definition.lifecycle().complete_review(),
            access.capability_input(),
            installed.contract().name(),
        )?,
    })
}

fn review_denial(
    mandatory: WorthQueryMandatoryReview,
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryMandatoryReviewAuthorizationDenial {
    WorthQueryMandatoryReviewAuthorizationDenial { denial, mandatory }
}

fn review_rejected(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::MandatoryReviewRejected,
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

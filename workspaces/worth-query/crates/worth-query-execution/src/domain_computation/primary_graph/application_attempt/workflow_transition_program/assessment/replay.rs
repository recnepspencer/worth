use worth_query_installation::facade::ApplicationSchema;

use super::{decode_hex_identity, evidence_meaning, projection_with_locator};
use crate::domain_computation::primary_graph::{
    PreparedWorkflowAdvance, RequiredWorkflowAssessment, WorkflowProgressOutcome,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryObservedSource, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryWorkflowAssessmentPosture,
};

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn requested_instance(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        match self {
            Self::Transition { instance, .. }
            | Self::AwaitingEvidence { instance, .. }
            | Self::AwaitingApproval { instance, .. }
            | Self::ReplayOnly { instance, .. } => *instance,
            Self::AwaitingAssessment(prepared) => prepared.required.instance(),
            Self::AwaitingCondition(prepared) => prepared.required.instance(),
            Self::AwaitingOperation(prepared) => prepared.required.instance(),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn resolve_assessment_replay<
        Query,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        required: &RequiredWorkflowAssessment,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        let (admission, transition_locator, evidence_locator) = match self {
            Self::Transition {
                program,
                transition_identity_locator,
                assessment_identity_locator,
                ..
            } => (
                &program.read_set.admission,
                transition_identity_locator,
                assessment_identity_locator,
            ),
            Self::AwaitingAssessment(prepared) => (
                &prepared.admitted.read_set().admission,
                &prepared.layout.transition.identity,
                &prepared.layout.assessment_evidence.identity,
            ),
            Self::AwaitingCondition(prepared) => (
                &prepared.admitted.read_set().admission,
                &prepared.layout.transition.identity,
                &prepared.layout.assessment_evidence.identity,
            ),
            Self::AwaitingOperation(prepared) => (
                &prepared.admitted.read_set().admission,
                &prepared.layout.transition.identity,
                &prepared.layout.assessment_evidence.identity,
            ),
            Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                ..
            } => (
                &read_set.admission,
                transition_identity_locator,
                assessment_identity_locator,
            ),
            Self::AwaitingEvidence {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                ..
            } => (
                &read_set.admission,
                transition_identity_locator,
                assessment_identity_locator,
            ),
            Self::AwaitingApproval {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                ..
            } => (
                &read_set.admission,
                transition_identity_locator,
                assessment_identity_locator,
            ),
        };
        resolve_replay(
            runtime,
            admission,
            transition_locator,
            evidence_locator,
            required,
            settlement,
            source,
            posture,
            idempotency,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_replay<Schema, Operation, Input, Scope, Query>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    transition_locator: &worth_foundational::facade::AspectFieldLocator,
    evidence_locator: &worth_foundational::facade::AspectFieldLocator,
    required: &RequiredWorkflowAssessment,
    settlement: &WorthQueryOutputDemandSettlement,
    source: &WorthQueryObservedSource<Query>,
    posture: WorthQueryWorkflowAssessmentPosture,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let meaning = evidence_meaning(
        required,
        source.source_root(),
        settlement,
        source,
        posture,
        std::sync::Arc::from([]),
    );
    let assessment = projection_with_locator(evidence_locator, meaning);
    let transition_identity = decode_hex_identity(required.transition_identity())
        .expect("workflow transition identity is an encoded SHA-256 digest");
    let resolution = runtime
        .resolve_admitted_application_idempotency(
            admission,
            idempotency
                .bind_workflow_transition(&transition_identity)
                .bind_workflow_assessment(&assessment.intent_identity),
        )?
        .into_resolution();
    Ok(match resolution {
        WorthQueryApplicationIdempotencyResolution::Unseen => None,
        WorthQueryApplicationIdempotencyResolution::IntentDrift => Some(
            WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
                super::super::super::WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            )),
        ),
        WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
            Some(runtime.primary_provider.graph.with_runtime(|relational| {
                super::super::publication::project(
                    relational,
                    receipt,
                    required.transition_identity().to_owned(),
                    transition_locator.clone(),
                    required.node_path().to_owned(),
                    Some(assessment),
                    None,
                    None,
                    true,
                )
            }))
        }
    })
}
